use crate::config::DEFAULT_IR_FILE;
use crate::domain::audio_processor::AudioProcessor;
use crate::domain::dto::effect::cabinet_dto::CabinetDto;
use crate::domain::dto::effect::effect_dto::EffectDto;
use crate::domain::effect::Effect;
use crate::domain::validation::sanitize_wav_file_name;
use crate::infrastructure::file_loader::{FileLoader, FileLoaderTrait};
use crate::services::processors::resampler::resampler::ResamplerImpl;
use rustfft::num_complex::Complex;
use rustfft::{Fft, FftPlanner};
use std::collections::VecDeque;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tracing::{info, warn};

const CUSTOM_IR_ENV_KEY: &str = "RUSTRIFF_CUSTOM_IR_DIR";
/// Chunk size used by IR resampling during initialization.
const IR_RESAMPLER_CHUNK_SIZE: usize = 256;
/// Number of input samples collected before one FFT convolution pass.
const CONV_BLOCK_SIZE: usize = 256;
/// Upper bound for IR length to keep real-time CPU usage predictable.
const MAX_IR_SAMPLES: usize = 2048;
/// Safety clamp applied to processed output to reduce hard digital clipping.
const OUTPUT_CLAMP: f32 = 0.98;

/// FFT-based cabinet simulator that convolves input audio with a loaded IR.
///
/// The effect uses block convolution:
/// - gather `CONV_BLOCK_SIZE` input samples,
/// - forward FFT,
/// - multiply by precomputed IR FFT kernel,
/// - inverse FFT,
/// - overlap-add into an output queue.
///
/// `overlap-add` means each processed block contributes some samples that belong
/// to the same time positions as the next block. Instead of overwriting those
/// positions, we add them together in `output_queue`.
/// This reconstructs the same linear-convolution result you would get from a
/// full sample-by-sample convolution, but in a block-friendly way.
///
/// Public-facing behavior is sample-by-sample through [`AudioProcessor::process`],
/// while internally processing is block-based for efficiency.
pub struct Cabinet {
    id: u32,
    name: String,
    is_active: Arc<AtomicBool>,
    color: String,
    /// Filename of the currently loaded IR (e.g. `"vintage-4x12.wav"`).
    ///
    /// Stored so it can be serialized into [`CabinetDto`] for persistence and
    /// used to re-initialize the cabinet after a configuration reload.
    ir_file_path: String,
    /// Time-domain IR samples after optional resampling and truncation.
    /// Retained in memory because `ir_buffer.len()` is needed on every block
    /// to compute the correct overlap-add region length.
    ir_buffer: Vec<f32>,
    /// Frequency-domain FFT of the IR — the convolution kernel `H[k]`.
    /// Pre-computed once in `new()` so the hot audio path only multiplies
    /// rather than re-computing the IR FFT every block.
    ir_fft_kernel: Vec<Complex<f32>>,
    ir_fft_size: usize,
    fft_forward: Arc<dyn Fft<f32>>,
    fft_inverse: Arc<dyn Fft<f32>>,
    /// Reusable working buffer of length `ir_fft_size` for in-place FFT ops.
    /// Preallocated to avoid heap allocation on the real-time audio thread.
    fft_scratch: Vec<Complex<f32>>,
    /// Attenuation factor derived from the IR peak; prevents output clipping
    /// caused by high-amplitude IR files.
    cabinet_gain: f32,
    /// Guards against flooding the log with repeated "IR unavailable" warnings.
    has_logged_ir_unavailable: bool,
    /// Accumulates incoming samples until a full `CONV_BLOCK_SIZE` block is
    /// ready for FFT convolution.
    input_block: Vec<f32>,
    /// Ring buffer of ready-to-deliver processed samples.
    /// Overlap-add writes ahead into positions that represent future blocks,
    /// and [`AudioProcessor::process`] pops one sample per call.
    output_queue: VecDeque<f32>,
    /// Sample rate of the DSP pipeline; used to resample the IR if needed.
    dsp_sample_rate: u32,
}

impl Cabinet {
    /// Creates a new cabinet effect instance and prepares FFT convolution state.
    ///
    /// Initialization steps:
    /// - load default IR file,
    /// - optionally resample IR to the DSP sample rate,
    /// - truncate very long IRs,
    /// - precompute IR FFT kernel,
    /// - preallocate buffers used by the audio thread.
    pub fn new(
        id: u32,
        name: String,
        is_active: bool,
        color: String,
        ir_file_path: String,
        dsp_sample_rate: u32,
    ) -> Self {
        info!("init cabinet simulation");
        let file_loader = FileLoader::new();

        let selected_ir_file = if ir_file_path.trim().is_empty() {
            DEFAULT_IR_FILE.to_string()
        } else {
            match sanitize_wav_file_name(&ir_file_path) {
                Ok(sanitized) => sanitized,
                Err(err) => {
                    warn!(
                        "Invalid cabinet IR file '{}': {}. Falling back to default '{}'.",
                        ir_file_path, err, DEFAULT_IR_FILE
                    );
                    DEFAULT_IR_FILE.to_string()
                }
            }
        };

        let temp_file_path = Self::resolve_ir_file_path(&selected_ir_file).unwrap_or_else(|| {
            warn!(
                "Could not resolve IR '{}' in known directories. Falling back to default location.",
                selected_ir_file
            );
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("resources")
                .join("default_ir")
                .join(&selected_ir_file)
        });

        let ir_buffer = file_loader.read_wav_to_buffer(&temp_file_path);
        let ir_sample_rate = file_loader
            .read_wav_sample_rate(&temp_file_path)
            .unwrap_or(dsp_sample_rate);
        let (mut ir_buffer, resampling_applied) =
            Self::resample_if_needed(ir_buffer, ir_sample_rate, dsp_sample_rate);
        if ir_buffer.is_empty() {
            warn!(
                "Cabinet IR buffer is empty. IR file may be missing, unreadable, unsupported, or corrupt. Falling back to passthrough."
            );
        }
        if ir_buffer.len() > MAX_IR_SAMPLES {
            info!(
                "Cabinet IR too long ({} samples). Truncating to {} to keep real-time CPU stable.",
                ir_buffer.len(),
                MAX_IR_SAMPLES
            );
            ir_buffer.truncate(MAX_IR_SAMPLES);
        }
        let ir_fft_size = (CONV_BLOCK_SIZE + ir_buffer.len().saturating_sub(1))
            .next_power_of_two()
            .max(2);
        let output_queue_capacity = CONV_BLOCK_SIZE + ir_buffer.len();
        let (fft_forward, fft_inverse) = Self::build_fft_plans(ir_fft_size);
        let ir_fft_kernel = Self::convert_ir_to_fft_kernel(&ir_buffer, ir_fft_size, &fft_forward);
        let cabinet_gain = Self::compute_cabinet_gain(&ir_buffer);

        info!(
      "Cabinet rates -> ir_sample_rate={}, dsp_sample_rate={}, resampling_applied={}, ir_len={}, fft_size={}, cabinet_gain={}",
			ir_sample_rate,
			dsp_sample_rate,
			resampling_applied,
      ir_buffer.len(),
      ir_fft_size,
      cabinet_gain
		);

        Self {
            id,
            name,
            is_active: Arc::new(AtomicBool::new(is_active)),
            color,
            ir_file_path: selected_ir_file,
            ir_buffer,
            ir_fft_kernel,
            ir_fft_size,
            fft_forward,
            fft_inverse,
            fft_scratch: vec![Complex::new(0.0_f32, 0.0_f32); ir_fft_size],
            cabinet_gain,
            has_logged_ir_unavailable: false,
            input_block: Vec::with_capacity(CONV_BLOCK_SIZE),
            output_queue: VecDeque::with_capacity(output_queue_capacity),
            dsp_sample_rate,
        }
    }

    /// Computes a conservative gain factor from IR peak amplitude.
    ///
    /// If IR peak is above unity, this returns an attenuation factor `1.0 / peak`.
    /// Otherwise returns `1.0`.
    fn compute_cabinet_gain(ir_buffer: &[f32]) -> f32 {
        let peak = ir_buffer
            .iter()
            .fold(0.0_f32, |acc, sample| acc.max(sample.abs()));

        if peak > 1.0 {
            1.0 / peak
        } else {
            1.0
        }
    }

    /// Builds forward and inverse FFT plans for a fixed FFT size.
    fn build_fft_plans(fft_size: usize) -> (Arc<dyn Fft<f32>>, Arc<dyn Fft<f32>>) {
        let mut planner = FftPlanner::<f32>::new();
        let forward = planner.plan_fft_forward(fft_size);
        let inverse = planner.plan_fft_inverse(fft_size);
        (forward, inverse)
    }

    /// Searches several well-known directories for an IR file and returns the
    /// first path that resolves to an existing regular file.
    ///
    /// Search order:
    /// 1. `$RUSTRIFF_CUSTOM_IR_DIR/<file_name>` — user-defined override via the
    ///    [`CUSTOM_IR_ENV_KEY`] environment variable (useful during development
    ///    or when testing a custom cabinet).
    /// 2. `CARGO_MANIFEST_DIR/resources/default_ir/<file_name>` — bundled
    ///    default IRs when running under `cargo run` / `cargo tauri dev`.
    /// 3. `<exe_dir>/resources/default_ir/<file_name>` — bundled resources
    ///    relative to the installed executable in release builds.
    ///
    /// Returns `None` when no candidate exists; the caller is responsible for
    /// logging a warning and supplying a fallback path.
    fn resolve_ir_file_path(file_name: &str) -> Option<PathBuf> {
        let sanitized_file_name = match sanitize_wav_file_name(file_name) {
            Ok(name) => name,
            Err(err) => {
                warn!(
                    "Refusing to resolve invalid IR file name '{}': {}",
                    file_name, err
                );
                return None;
            }
        };

        let mut candidates = Vec::new();

        if let Ok(custom_dir) = std::env::var(CUSTOM_IR_ENV_KEY) {
            candidates.push(PathBuf::from(custom_dir).join(&sanitized_file_name));
        }

        candidates.push(
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("resources")
                .join("default_ir")
                .join(&sanitized_file_name),
        );

        if let Ok(exe_path) = std::env::current_exe() {
            if let Some(exe_dir) = exe_path.parent() {
                candidates.push(
                    exe_dir
                        .join("resources")
                        .join("default_ir")
                        .join(&sanitized_file_name),
                );
            }
        }

        candidates.into_iter().find(|path| path.is_file())
    }

    /// Converts the time-domain IR into a frequency-domain convolution kernel.
    ///
    /// The returned vector has length `fft_size` and is zero-padded when IR is shorter.
    fn convert_ir_to_fft_kernel(
        ir_buffer: &[f32],
        fft_size: usize,
        fft_forward: &Arc<dyn Fft<f32>>,
    ) -> Vec<Complex<f32>> {
        if ir_buffer.is_empty() {
            return Vec::new();
        }

        let mut buffer = vec![Complex::new(0.0_f32, 0.0_f32); fft_size];
        for (idx, sample) in ir_buffer.iter().enumerate() {
            buffer[idx].re = *sample;
        }

        fft_forward.process(&mut buffer);
        buffer
    }

    /// Pushes dry samples when IR data is unavailable (missing/unreadable/corrupt).
    fn push_passthrough_block_for_ir_unavailable(&mut self) {
        for &sample in &self.input_block {
            self.output_queue.push_back(sample);
        }
    }

    /// Copies the current input block into the reusable FFT scratch buffer.
    ///
    /// The remaining tail is zero-filled to represent block convolution padding.
    fn prepare_fft_input_from_block(&mut self) {
        self.fft_scratch.fill(Complex::new(0.0_f32, 0.0_f32));
        for (sample_index, sample) in self.input_block.iter().enumerate() {
            self.fft_scratch[sample_index].re = *sample;
        }
    }

    /// Applies point-wise complex multiplication `X[k] *= H[k]` in frequency domain.
    fn multiply_input_by_ir_in_frequency_domain(&mut self) {
        for (input_bin, ir_bin) in self.fft_scratch.iter_mut().zip(self.ir_fft_kernel.iter()) {
            *input_bin *= *ir_bin;
        }
    }

    /// Performs overlap-add accumulation of the current IFFT block into output queue.
    ///
    /// This method:
    /// - normalizes by FFT size,
    /// - applies cabinet gain,
    /// - accumulates into queued samples so block boundaries remain continuous.
    ///
    /// Why "add" and not "replace"?
    ///
    /// Convolving one input block with an IR produces an output that is longer than
    /// the input block (`input_len + ir_len - 1`). The tail of the current block
    /// lands in the same timeline region as the start of future blocks.
    ///
    /// If we replaced samples, that tail energy would be lost and you would hear
    /// discontinuities (clicks/crackle) at block edges. By adding into existing
    /// queued values, block outputs stitch together into one continuous signal.
    fn overlap_add_ifft_block_into_queue(&mut self) {
        let fft_normalization = self.ir_fft_size as f32;
        let linear_conv_len = self.input_block.len() + self.ir_buffer.len().saturating_sub(1);

        if self.output_queue.len() < linear_conv_len {
            self.output_queue.resize(linear_conv_len, 0.0);
        }

        for sample_index in 0..linear_conv_len {
            if let Some(output_slot) = self.output_queue.get_mut(sample_index) {
                *output_slot +=
                    (self.fft_scratch[sample_index].re / fft_normalization) * self.cabinet_gain;
            }
        }
    }

    /// Runs one full block convolution pass for the currently buffered input block.
    ///
    /// Signal flow in plain language:
    ///
    /// 1. Put the current time-domain input block (`x`) into the FFT buffer.
    /// 2. Convert it to frequency domain with FFT (`X = FFT(x)`).
    /// 3. Apply cabinet tone by multiplying each frequency bin with the precomputed
    ///    IR kernel (`Y[k] = X[k] * H[k]`).
    /// 4. Convert back to time domain (`y = IFFT(Y)`).
    /// 5. Overlap-add `y` into `output_queue` so neighboring blocks combine correctly.
    ///
    /// Compact form: `x -> FFT(x) -> FFT(x) * FFT(h) -> IFFT -> overlap-add`.
    fn convolve_current_block(&mut self) {
        if self.input_block.is_empty() {
            return;
        }

        if self.ir_fft_kernel.is_empty() {
            if !self.has_logged_ir_unavailable {
                warn!(
                    "Cabinet IR kernel is empty. Using passthrough until a valid IR can be loaded."
                );
                self.has_logged_ir_unavailable = true;
            }
            self.push_passthrough_block_for_ir_unavailable();
            self.input_block.clear();
            return;
        }

        self.prepare_fft_input_from_block();
        self.fft_forward.process(&mut self.fft_scratch);
        self.multiply_input_by_ir_in_frequency_domain();
        self.fft_inverse.process(&mut self.fft_scratch);
        self.overlap_add_ifft_block_into_queue();

        self.input_block.clear();
    }

    /// Resamples an IR buffer when source and target sample rates differ.
    ///
    /// Returns `(buffer, was_resampled)`.
    /// On any resampler setup/processing failure, the original buffer is returned.
    fn resample_if_needed(
        buffer: Vec<f32>,
        source_rate: u32,
        target_rate: u32,
    ) -> (Vec<f32>, bool) {
        if buffer.len() < 2 || source_rate == 0 || target_rate == 0 || source_rate == target_rate {
            return (buffer, false);
        }

        let mut resampler =
            match ResamplerImpl::new(source_rate, target_rate, IR_RESAMPLER_CHUNK_SIZE) {
                Ok(resampler) => resampler,
                Err(err) => {
                    warn!(
					"Failed to initialize cabinet IR resampler ({} -> {}): {}. Using original IR buffer.",
					source_rate,
					target_rate,
					err
				);
                    return (buffer, false);
                }
            };

        let mut out = Vec::new();
        for &sample in &buffer {
            out.extend(resampler.process_sample(sample));
        }
        out.extend(resampler.flush());

        if out.is_empty() {
            warn!(
                "Cabinet IR resampling produced no output ({} -> {}). Using original IR buffer.",
                source_rate, target_rate
            );
            return (buffer, false);
        }

        (out, true)
    }

    /// Returns the sample rate at which cabinet DSP processing runs.
    pub fn sample_rate(&self) -> u32 {
        self.dsp_sample_rate
    }

    /// Returns the precomputed frequency-domain IR kernel.
    ///
    /// Mainly intended for diagnostics and tests.
    pub fn ir_fft_kernel(&self) -> &[Complex<f32>] {
        &self.ir_fft_kernel
    }

    /// Returns the FFT size used for cabinet convolution.
    pub fn ir_fft_size(&self) -> usize {
        self.ir_fft_size
    }
}

impl AudioProcessor for Cabinet {
    /// Processes one sample through the cabinet effect.
    ///
    /// Internally this is block-based:
    /// - one sample is dequeued from `output_queue`,
    /// - one sample is appended to `input_block`,
    /// - when the block is full, a new convolution block is computed.
    ///
    /// If queue underruns, silence is returned until the next block result is available.
    fn process(&mut self, sample: f32) -> f32 {
        let output_sample = self.output_queue.pop_front().unwrap_or(0.0);

        self.input_block.push(sample);
        if self.input_block.len() == CONV_BLOCK_SIZE {
            self.convolve_current_block();
        }

        output_sample.clamp(-OUTPUT_CLAMP, OUTPUT_CLAMP)
    }
}

impl Effect for Cabinet {
    fn id(&self) -> u32 {
        self.id
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn get_color(&self) -> String {
        self.color.clone()
    }

    /// Serializes the current cabinet state into a [`CabinetDto`].
    ///
    /// The DTO captures everything the frontend needs to re-render the pedal
    /// and everything the persistence layer needs to restore it on next launch,
    /// including the `ir_file_path` used to reload the correct IR.
    fn to_dto(&self) -> EffectDto {
        EffectDto::Cabinet(CabinetDto {
            id: self.id,
            name: self.name.clone(),
            is_active: self.is_active.load(Ordering::Relaxed),
            color: self.color.clone(),
            ir_file_path: self.ir_file_path.clone(),
        })
    }

    /// Returns a shared reference to the atomic active/bypass flag.
    ///
    /// The flag may be toggled from the UI thread without locking the audio
    /// thread, because [`AtomicBool`] operations are lock-free.
    fn active_flag(&self) -> Arc<AtomicBool> {
        Arc::clone(&self.is_active)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::audio_processor::AudioProcessor;
    use crate::domain::effect::Effect;

    const TEST_SAMPLE_RATE: u32 = 48_000;
    const BLOCK_SIZE: usize = 256;

    fn make_cabinet(ir_file: &str, is_active: bool) -> Cabinet {
        Cabinet::new(
            0,
            "Test Cabinet".to_string(),
            is_active,
            "#112233".to_string(),
            ir_file.to_string(),
            TEST_SAMPLE_RATE,
        )
    }

    fn feed_samples(cabinet: &mut Cabinet, samples: &[f32]) -> Vec<f32> {
        samples.iter().map(|&s| cabinet.process(s)).collect()
    }

    #[cfg(test)]
    mod success_path {
        use super::*;

        #[test]
        fn new_with_valid_ir_initializes_non_empty_fft_kernel() {
            let cab = make_cabinet("Vox-ac30.wav", true);
            assert!(
                !cab.ir_fft_kernel().is_empty(),
                "A valid IR file should produce a non-empty FFT kernel"
            );
        }

        #[test]
        fn new_with_valid_ir_fft_size_is_a_power_of_two() {
            let cab = make_cabinet("Vox-ac30.wav", true);
            let size = cab.ir_fft_size();
            assert!(size > 0);
            assert_eq!(
                size & (size - 1),
                0,
                "FFT size {size} is not a power of two"
            );
        }

        #[test]
        fn new_returns_the_configured_dsp_sample_rate() {
            let cab = make_cabinet("Vox-ac30.wav", true);
            assert_eq!(cab.sample_rate(), TEST_SAMPLE_RATE);
        }

        #[test]
        fn to_dto_round_trips_all_cabinet_fields() {
            let cab = make_cabinet("Vox-ac30.wav", true);
            if let EffectDto::Cabinet(dto) = cab.to_dto() {
                assert_eq!(dto.id, 0);
                assert_eq!(dto.name, "Test Cabinet");
                assert_eq!(dto.color, "#112233");
                assert_eq!(dto.ir_file_path, "Vox-ac30.wav");
                assert!(dto.is_active);
            } else {
                panic!("to_dto should return a Cabinet variant");
            }
        }

        #[test]
        fn process_produces_non_zero_output_after_one_full_block() {
            let mut cab = make_cabinet("Vox-ac30.wav", true);

            // Feed two blocks of constant input.  The first BLOCK_SIZE returns
            // are all 0.0 (queue underrun).  From index BLOCK_SIZE onward the
            // first convolution result emerges from the queue.
            let input = vec![0.5_f32; BLOCK_SIZE * 2];
            let output = feed_samples(&mut cab, &input);

            let post_block = &output[BLOCK_SIZE..];
            assert!(
                post_block.iter().any(|&s| s.abs() > 1e-6),
                "Convolution with a real IR should produce non-zero output after the first block"
            );
        }

        #[test]
        fn process_if_active_false_returns_input_sample_unchanged_without_block_delay() {
            let mut cab = make_cabinet("Vox-ac30.wav", false);

            let output = cab.process_if_active(0.75_f32);
            assert!(
                (output - 0.75).abs() < 1e-6,
                "Bypassed cabinet should return the input sample unchanged"
            );
        }

        #[test]
        fn output_is_clamped_to_output_clamp_range() {
            let mut cab = make_cabinet("Vox-ac30.wav", true);
            let input = vec![1.0_f32; BLOCK_SIZE * 2];
            let output = feed_samples(&mut cab, &input);
            for &sample in &output {
                assert!(
                    sample.abs() <= OUTPUT_CLAMP + f32::EPSILON,
                    "Output sample {sample} exceeds clamp limit {OUTPUT_CLAMP}"
                );
            }
        }
    }

    #[cfg(test)]
    mod failure_path {
        use super::*;

        #[test]
        fn new_with_missing_ir_file_produces_empty_fft_kernel() {
            let cab = make_cabinet("nonexistent-ir-file.wav", true);
            assert!(
                cab.ir_fft_kernel().is_empty(),
                "A missing IR file should result in an empty FFT kernel (passthrough mode)"
            );
        }

        #[test]
        fn process_with_empty_kernel_passes_signal_through_after_one_block_delay() {
            let mut cab = make_cabinet("nonexistent-ir-file.wav", true);
            assert!(cab.ir_fft_kernel().is_empty());

            let input: Vec<f32> = (0..BLOCK_SIZE * 2).map(|i| i as f32 * 0.001).collect();
            let output = feed_samples(&mut cab, &input);

            for (i, sample) in output.iter().enumerate().take(BLOCK_SIZE) {
                assert!(
                    sample.abs() < 1e-6,
                    "Pre-block output should be silent (got {} at index {i})",
                    sample
                );
            }

            for i in 0..BLOCK_SIZE {
                assert!(
                    (output[BLOCK_SIZE + i] - input[i]).abs() < 1e-6,
                    "Post-block output[{}] ({}) should equal input[{i}] ({})",
                    BLOCK_SIZE + i,
                    output[BLOCK_SIZE + i],
                    input[i]
                );
            }
        }

        #[test]
        fn new_with_empty_ir_path_does_not_panic() {
            let cab = make_cabinet("", true);
            let _ = cab.ir_fft_size();
            let _ = cab.sample_rate();
        }

        #[test]
        fn new_with_traversal_ir_path_falls_back_to_default_ir_file_name() {
            let cab = make_cabinet("../secrets.wav", true);

            if let EffectDto::Cabinet(dto) = cab.to_dto() {
                assert_eq!(dto.ir_file_path, DEFAULT_IR_FILE);
            } else {
                panic!("Expected Cabinet effect DTO");
            }
        }

        #[test]
        fn new_with_non_wav_ir_path_falls_back_to_default_ir_file_name() {
            let cab = make_cabinet("not-an-ir.mp3", true);

            if let EffectDto::Cabinet(dto) = cab.to_dto() {
                assert_eq!(dto.ir_file_path, DEFAULT_IR_FILE);
            } else {
                panic!("Expected Cabinet effect DTO");
            }
        }
    }
}
