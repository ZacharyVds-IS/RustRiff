use rubato::{
    Resampler, SincFixedIn, SincInterpolationParameters, SincInterpolationType, WindowFunction,
};
use tracing::{error, info, warn};

/// A streaming, sample-accurate resampler backed by rubato's [`SincFixedIn`].
///
/// Samples are fed one at a time through [`process_sample`]. Internally they
/// are accumulated in a fixed-size chunk buffer; when the buffer is full rubato
/// processes the whole chunk and returns resampled output samples. This means
/// `process_sample` returns an empty `Vec` most of the time and a batch of
/// output samples every `input_chunk_size` calls.
///
/// Call [`flush`] before discarding the resampler (e.g. on shutdown) to
/// process any samples remaining in the internal buffer. Remaining space is
/// zero-padded so that rubato receives a complete chunk.
///
/// # Construction
///
/// Use [`ResamplerImpl::new`] with non-equal, non-zero rates and a non-zero
/// chunk size. Returns `Err` if any of those constraints are violated.
///
/// [`process_sample`]: ResamplerImpl::process_sample
/// [`flush`]: ResamplerImpl::flush
pub struct ResamplerImpl {
    inner: SincFixedIn<f32>,
    input_chunk_size: usize,
    input_buffer: Vec<f32>,
}

impl ResamplerImpl {
    /// Creates a new `RubatoResampler` that converts audio from `input_rate` to `output_rate`.
    ///
    /// The resampler buffers incoming samples into chunks of exactly `input_chunk_size`
    /// before processing. A larger chunk size improves quality (more filter context)
    /// but increases latency; a smaller chunk size reduces latency at a slight quality cost.
    ///
    /// # Arguments
    ///
    /// * `input_rate`       – Sample rate of the incoming audio in Hz (must be > 0).
    /// * `output_rate`      – Sample rate of the desired output audio in Hz (must be > 0).
    /// * `input_chunk_size` – Number of input samples per rubato processing call (must be > 0).
    ///
    /// # Errors
    ///
    /// Returns `Err(String)` if:
    /// - Either rate is zero.
    /// - `input_chunk_size` is zero.
    /// - Rubato fails to initialise (e.g. an extreme or unsupported ratio).
    pub fn new(input_rate: u32, output_rate: u32, input_chunk_size: usize) -> Result<Self, String> {
        if input_rate == 0 || output_rate == 0 {
            return Err("Sample rates must be > 0".to_string());
        }

        if input_chunk_size == 0 {
            return Err("Chunk size must be > 0".to_string());
        }

        let params = SincInterpolationParameters {
            // 128 taps: good balance of frequency accuracy vs CPU cost per chunk.
            sinc_len: 128,
            // 0.95 of Nyquist: passes up to 95 % of the audio band, reduces aliasing.
            f_cutoff: 0.95,
            interpolation: SincInterpolationType::Linear,
            // 128× oversampling of the sinc table: lower interpolation error.
            oversampling_factor: 128,
            window: WindowFunction::BlackmanHarris2,
        };

        let ratio = output_rate as f64 / input_rate as f64;
        let inner = SincFixedIn::<f32>::new(
            ratio,
            // Allow the ratio to vary up to 2× — more than enough for static device-rate conversion.
            2.0,
            params,
            input_chunk_size,
            1,
        )
        .map_err(|e| format!("Failed to create rubato resampler: {e}"))?;

        Ok(Self {
            inner,
            input_chunk_size,
            input_buffer: Vec::with_capacity(input_chunk_size),
        })
    }

    /// Feeds a single sample into the resampler and returns any output samples produced.
    ///
    /// Internally the sample is appended to a chunk buffer. No output is produced until
    /// the buffer reaches `input_chunk_size`, at which point rubato processes the full
    /// chunk and returns a batch of resampled `f32` samples.
    ///
    /// # Returns
    ///
    /// - An empty `Vec` while the chunk buffer is still filling.
    /// - A `Vec` of resampled `f32` samples once a full chunk has been processed.
    pub fn process_sample(&mut self, sample: f32) -> Vec<f32> {
        self.input_buffer.push(sample);

        if self.input_buffer.len() < self.input_chunk_size {
            return Vec::new();
        }

        let chunk: Vec<f32> = self.input_buffer.drain(..self.input_chunk_size).collect();
        self.process_chunk(chunk)
    }

    /// Processes any samples remaining in the internal buffer by zero-padding to a full chunk.
    ///
    /// Call this once when the audio stream ends or the loopback is stopped to avoid
    /// losing samples that have not yet formed a complete chunk. The trailing silence
    /// introduced by the padding is audibly negligible given typical chunk sizes.
    ///
    /// # Returns
    ///
    /// - A `Vec` of resampled `f32` samples produced from the padded chunk.
    /// - An empty `Vec` if the buffer was already empty.
    pub fn flush(&mut self) -> Vec<f32> {
        if self.input_buffer.is_empty() {
            return Vec::new();
        }

        let mut padded_chunk: Vec<f32> = self.input_buffer.drain(..).collect();
        padded_chunk.resize(self.input_chunk_size, 0.0);
        self.process_chunk(padded_chunk)
    }

    /// Runs a full chunk of `input_chunk_size` samples through the rubato resampler.
    ///
    /// This is the only place rubato is called. Errors are logged and an empty
    /// `Vec` is returned so the pipeline degrades gracefully rather than panicking.
    fn process_chunk(&mut self, input_chunk: Vec<f32>) -> Vec<f32> {
        let input = vec![input_chunk];

        match self.inner.process(&input, None) {
            Ok(output) => output.into_iter().next().unwrap_or_default(),
            Err(e) => {
                error!("Rubato processing failed: {e}");
                Vec::new()
            }
        }
    }
}

/// Determines when resampling occurs relative to the DSP chain based on the
/// input and output sample rates.
///
/// Construct the correct variant using [`ResamplePolicy::from_rates`]; do not
/// construct variants directly unless writing tests.
///
/// | Condition            | Variant    | Resampler placement                                 |
/// |----------------------|------------|-----------------------------------------------------|
/// | `input == output`    | `Bypass`   | No resampler — zero overhead                        |
/// | `input  > output`    | `PreDsp`   | Downsample **before** DSP; DSP runs at output rate  |
/// | `input  < output`    | `PostDsp`  | Upsample **after** DSP; DSP runs at input rate      |
///
/// Both `PreDsp` and `PostDsp` run the DSP chain at the *lower* of the two rates,
/// which minimises CPU cost for gain, EQ, and any future processors.
///
/// [`Bypass`]: ResamplePolicy::Bypass
/// [`PreDsp`]: ResamplePolicy::PreDsp
/// [`PostDsp`]: ResamplePolicy::PostDsp
pub enum ResamplePolicy {
    /// Input and output rates are equal — the DSP chain is applied directly with no
    /// resampling overhead.
    Bypass,

    /// Input rate is higher than output rate. The [`ResamplerImpl`] downsamples each
    /// input sample to the output rate **before** it enters the DSP chain.
    PreDsp(ResamplerImpl),

    /// Input rate is lower than output rate. The DSP chain processes the sample at the
    /// input rate first, then the [`ResamplerImpl`] upsamples the result to the output rate.
    PostDsp(ResamplerImpl),
}

impl ResamplePolicy {
    /// Selects and initialises the correct [`ResamplePolicy`] variant for the given rates.
    ///
    /// Logs the chosen path at `info` level on startup. If the [`ResamplerImpl`] fails to
    /// initialise (e.g. rates are zero or rubato rejects the ratio) the method logs a `warn`
    /// and falls back to [`Bypass`] so the pipeline keeps running without resampling.
    ///
    /// # Arguments
    ///
    /// * `input_rate`  – Sample rate of the input device in Hz.
    /// * `output_rate` – Sample rate of the output device in Hz.
    /// * `chunk_size`  – Number of input samples per rubato processing call (forwarded to [`ResamplerImpl::new`]).
    ///
    /// # Returns
    ///
    /// The most appropriate [`ResamplePolicy`] variant for the given rate pair.
    ///
    /// [`Bypass`]: ResamplePolicy::Bypass
    pub fn from_rates(input_rate: u32, output_rate: u32, chunk_size: usize) -> Self {
        match input_rate.cmp(&output_rate) {
            std::cmp::Ordering::Equal => {
                info!("Sample rate is equal ({input_rate} Hz) — no resampling needed");
                Self::Bypass
            }
            std::cmp::Ordering::Greater => {
                info!(
                    "Sample rates differ: input ({input_rate} Hz) > output ({output_rate} Hz) — downsampling before DSP"
                );
                match ResamplerImpl::new(input_rate, output_rate, chunk_size) {
                    Ok(r) => Self::PreDsp(r),
                    Err(e) => {
                        warn!("Failed to initialise pre-DSP downsampler, using bypass: {e}");
                        Self::Bypass
                    }
                }
            }
            std::cmp::Ordering::Less => {
                info!(
                    "Sample rates differ: input ({input_rate} Hz) < output ({output_rate} Hz) — upsampling after DSP"
                );
                match ResamplerImpl::new(input_rate, output_rate, chunk_size) {
                    Ok(r) => Self::PostDsp(r),
                    Err(e) => {
                        warn!("Failed to initialise post-DSP upsampler, using bypass: {e}");
                        Self::Bypass
                    }
                }
            }
        }
    }

    /// Processes a single input sample through the resampling policy and DSP chain.
    ///
    /// The `dsp` closure represents the full effects chain (gain → EQ → master volume).
    /// Where exactly in the pipeline `dsp` is called depends on the active variant:
    ///
    /// | Variant    | Order                                   |
    /// |------------|-----------------------------------------|
    /// | `Bypass`   | `dsp(sample)` → 1 output sample         |
    /// | `PreDsp`   | resample → `dsp` per result             |
    /// | `PostDsp`  | `dsp(sample)` → resample output         |
    ///
    /// # Arguments
    ///
    /// * `sample` – A single raw `f32` audio sample from the input ring buffer.
    /// * `dsp`    – Mutable closure that applies the full DSP chain to one sample and returns the result.
    ///
    /// # Returns
    ///
    /// Zero or more `f32` output samples ready to be pushed to the output ring buffer.
    /// Returns an empty `Vec` when the resampler's internal chunk buffer is not yet full.
    pub fn process(&mut self, sample: f32, dsp: &mut impl FnMut(f32) -> f32) -> Vec<f32> {
        match self {
            Self::Bypass => vec![dsp(sample)],
            Self::PreDsp(resampler) => resampler
                .process_sample(sample)
                .into_iter()
                .map(dsp)
                .collect(),
            Self::PostDsp(resampler) => resampler.process_sample(dsp(sample)),
        }
    }

    /// Flushes any samples remaining in the resampler's internal buffer at shutdown.
    ///
    /// Should be called once after the worker loop exits to avoid losing the tail of
    /// audio that has not yet filled a complete chunk. The `dsp` closure is applied to
    /// flushed samples in the same position as during normal processing.
    ///
    /// Returns an empty `Vec` for [`Bypass`] (nothing buffered) and for active resamplers
    /// whose buffer was already empty.
    ///
    /// # Arguments
    ///
    /// * `dsp` – The same DSP closure used in [`process`], applied to any remaining samples.
    ///
    /// # Returns
    ///
    /// Zero or more `f32` output samples from the flushed remainder.
    ///
    /// [`Bypass`]: ResamplePolicy::Bypass
    /// [`process`]: ResamplePolicy::process
    pub fn flush(&mut self, dsp: &mut impl FnMut(f32) -> f32) -> Vec<f32> {
        match self {
            Self::Bypass => Vec::new(),
            Self::PreDsp(resampler) => resampler.flush().into_iter().map(dsp).collect(),
            Self::PostDsp(resampler) => resampler.flush(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod rubato_resampler_tests {
        use super::*;

        mod success_path {
            use super::*;

            #[test]
            fn upsampling_eventually_produces_output_samples() {
                let mut resampler = ResamplerImpl::new(44_100, 48_000, 32).unwrap();
                let mut produced = 0usize;

                for _ in 0..512 {
                    produced += resampler.process_sample(0.2).len();
                }
                produced += resampler.flush().len();

                assert!(
                    produced > 0,
                    "Upsampler should eventually produce output samples"
                );
            }

            #[test]
            fn downsampling_outputs_fewer_samples_than_input() {
                let mut resampler = ResamplerImpl::new(48_000, 44_100, 32).unwrap();
                let mut produced = 0usize;

                for _ in 0..128 {
                    produced += resampler.process_sample(0.2).len();
                }
                produced += resampler.flush().len();

                assert!(
                    produced < 128,
                    "Downsampler should produce fewer samples than consumed"
                );
            }

            #[test]
            fn process_sample_buffers_until_chunk_is_full() {
                let chunk_size = 16;
                let mut resampler = ResamplerImpl::new(44_100, 48_000, chunk_size).unwrap();
                for _ in 0..(chunk_size - 1) {
                    assert!(
                        resampler.process_sample(0.1).is_empty(),
                        "Should buffer until chunk is full"
                    );
                }

                let _ = resampler.process_sample(0.1);

                let mut eventually_produced = false;
                for _ in 0..(chunk_size * 8) {
                    if !resampler.process_sample(0.1).is_empty() {
                        eventually_produced = true;
                        break;
                    }
                }

                assert!(
                    eventually_produced,
                    "Resampler should eventually produce output after receiving enough samples"
                );
            }

            #[test]
            fn flush_clears_remaining_buffered_input() {
                let chunk_size = 32;
                let mut resampler = ResamplerImpl::new(44_100, 48_000, chunk_size).unwrap();
                for _ in 0..10 {
                    resampler.process_sample(0.1);
                }

                assert!(
                    !resampler.input_buffer.is_empty(),
                    "Input buffer should contain pending samples before flush"
                );
                let _ = resampler.flush();
                assert!(
                    resampler.input_buffer.is_empty(),
                    "Flush should clear pending input samples"
                );
            }

            #[test]
            fn flush_on_empty_buffer_returns_nothing() {
                let mut resampler = ResamplerImpl::new(44_100, 48_000, 32).unwrap();
                let flushed = resampler.flush();
                assert!(
                    flushed.is_empty(),
                    "Flush on an empty buffer should return no samples"
                );
            }
        }

        mod failure_path {
            use super::*;

            #[test]
            fn zero_input_rate_returns_error() {
                let result = ResamplerImpl::new(0, 48_000, 32);
                assert!(result.is_err(), "Zero input rate should return an error");
            }

            #[test]
            fn zero_output_rate_returns_error() {
                let result = ResamplerImpl::new(44_100, 0, 32);
                assert!(result.is_err(), "Zero output rate should return an error");
            }

            #[test]
            fn zero_chunk_size_returns_error() {
                let result = ResamplerImpl::new(44_100, 48_000, 0);
                assert!(result.is_err(), "Zero chunk size should return an error");
            }
        }
    }

    mod resample_policy_tests {
        use super::*;

        mod success_path {
            use super::*;

            #[test]
            fn equal_rates_selects_bypass() {
                let policy = ResamplePolicy::from_rates(48_000, 48_000, 32);
                assert!(matches!(policy, ResamplePolicy::Bypass));
            }

            #[test]
            fn higher_input_rate_selects_pre_dsp() {
                let policy = ResamplePolicy::from_rates(48_000, 44_100, 32);
                assert!(matches!(policy, ResamplePolicy::PreDsp(_)));
            }

            #[test]
            fn lower_input_rate_selects_post_dsp() {
                let policy = ResamplePolicy::from_rates(44_100, 48_000, 32);
                assert!(matches!(policy, ResamplePolicy::PostDsp(_)));
            }

            #[test]
            fn bypass_process_applies_dsp_and_returns_one_sample() {
                let mut policy = ResamplePolicy::Bypass;
                let result = policy.process(0.5, &mut |s| s * 2.0);

                assert_eq!(result.len(), 1);
                assert!(
                    (result[0] - 1.0).abs() < 1e-6,
                    "Bypass should apply DSP directly"
                );
            }

            #[test]
            fn bypass_flush_returns_empty() {
                let mut policy = ResamplePolicy::Bypass;
                let result = policy.flush(&mut |s| s);

                assert!(
                    result.is_empty(),
                    "Bypass flush should always return nothing"
                );
            }

            #[test]
            fn pre_dsp_applies_dsp_to_resampled_output() {
                let mut policy = ResamplePolicy::from_rates(48_000, 44_100, 32);
                let mut dsp_called = false;

                for _ in 0..1024 {
                    policy.process(0.5, &mut |s| {
                        dsp_called = true;
                        s
                    });
                }

                let _ = policy.flush(&mut |s| {
                    dsp_called = true;
                    s
                });

                assert!(
                    dsp_called,
                    "PreDsp should call DSP on the downsampled samples"
                );
            }

            #[test]
            fn post_dsp_applies_dsp_before_resampling() {
                let mut policy = ResamplePolicy::from_rates(44_100, 48_000, 32);
                let mut dsp_call_count = 0;
                let input_count = 128;

                for _ in 0..input_count {
                    policy.process(0.5, &mut |s| {
                        dsp_call_count += 1;
                        s
                    });
                }
                assert_eq!(
                    dsp_call_count, input_count,
                    "PostDsp should call DSP once per input sample"
                );
            }
        }

        mod failure_path {
            use super::*;

            #[test]
            fn invalid_rates_fall_back_to_bypass() {
                let policy = ResamplePolicy::from_rates(0, 48_000, 32);
                assert!(
                    matches!(policy, ResamplePolicy::Bypass),
                    "Invalid rates should fall back to Bypass"
                );
            }

            #[test]
            fn zero_chunk_size_falls_back_to_bypass() {
                let policy = ResamplePolicy::from_rates(44_100, 48_000, 0);
                assert!(
                    matches!(policy, ResamplePolicy::Bypass),
                    "Zero chunk size should fall back to Bypass"
                );
            }
        }
    }
}
