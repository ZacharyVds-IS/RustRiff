//! Latency-oriented measurement helpers that sit on top of [`AudioService`].
//!
//! # What this module measures
//!
//! There are three distinct kinds of latency in the signal chain, each exposed
//! by a different family of functions:
//!
//! | Kind | Functions | What it captures |
//! |---|---|---|
//! | **CPU execution time** | [`measure_gain_latency`], [`measure_tone_stack_latency`], [`measure_all_dsp_timings`] | How many µs each DSP processor adds per sample of CPU work |
//! | **Algorithmic latency** | [`measure_all_dsp_algorithmic_latency`] | Inherent sample-delay introduced by an effect's design (lookahead, delay lines, etc.) |
//! | **I/O buffer latency** | [`measure_buffer_latency`] | Buffering delay imposed by the CPAL input and output stream frame sizes |
//! | **Round-trip latency** | [`measure_round_trip_latency`] | True end-to-end wall-clock delay measured by injecting an impulse and timing its echo |
use crate::domain::dto::algorithmic_latency_dto::AlgorithmicLatencyDto;
use crate::domain::dto::buffer_latency_dto::BufferLatencyDto;
use crate::domain::dto::execution_timing_dto::ExecutionTimingDto;
use crate::domain::dto::round_trip_latency_dto::RoundTripLatencyDto;
use crate::infrastructure::audio_handler::AudioHandlerTrait;
use crate::services::analyzers::latency_analyzer::LatencyAnalyzer;
use crate::services::audio_service::AudioService;
use crate::services::processors::gain::gain_processor::GainProcessor;
use crate::services::processors::tone_stack::tone_stack_processor::ToneStackProcessor;
use crate::services::round_trip_latency_session::RoundTripLatencySession;
use cpal::BufferSize;
use std::time::Duration;

/// Stateless facade that groups all latency measurement operations.
///
/// Every method takes a shared reference to an [`AudioService`] (or a raw
/// [`AudioHandlerTrait`] for round-trip) so the caller can hold whichever lock
/// granularity is appropriate.  None of the methods start or stop the loopback.
///
/// [`AudioService`]: crate::services::audio_service::AudioService
/// [`AudioHandlerTrait`]: crate::infrastructure::audio_handler::AudioHandlerTrait
pub struct AudioLatencyMeasurementService;

impl AudioLatencyMeasurementService {
    /// Measures the CPU execution cost added by the [`GainProcessor`] in the current channel.
    ///
    /// Internally this benchmarks the gain processor against a zero-work passthrough over
    /// `iterations × block_size` samples and returns the *net* cost — i.e. the passthrough
    /// baseline is subtracted so the result isolates the processor's own work.
    ///
    /// # Arguments
    ///
    /// * `audio_service` — Service snapshot used to read the current channel's gain arc.
    /// * `block_size` — Number of samples per iteration.  Larger values reduce timer
    ///   overhead noise; 2 048 is the recommended default for command calls.
    ///
    /// # Returns
    ///
    /// Added execution cost in **microseconds per sample** (µs/sample), clamped to `≥ 0`.
    ///
    /// [`GainProcessor`]: crate::services::processors::gain::gain_processor::GainProcessor
    pub fn measure_gain_latency(audio_service: &AudioService, block_size: usize) -> f64 {
        let gain_arc = {
            let cm = audio_service.channel_manager().lock().unwrap();
            cm.current_channel().unwrap().gain().clone()
        };
        let mut gain = GainProcessor::new(gain_arc);
        LatencyAnalyzer::measure_effect_added_execution_us(&mut gain, 256, block_size)
    }

    pub fn measure_tone_stack_latency(audio_service: &AudioService, block_size: usize) -> f64 {
        let (tone_stack_arc, dsp_rate) = {
            let cm = audio_service.channel_manager().lock().unwrap();
            (
                cm.current_channel().unwrap().tone_stack().clone(),
                audio_service.dsp_chain_sample_rate(),
            )
        };
        let mut tone_stack = ToneStackProcessor::new(tone_stack_arc, dsp_rate);
        LatencyAnalyzer::measure_effect_added_execution_us(&mut tone_stack, 256, block_size)
    }

    pub fn measure_volume_latency(audio_service: &AudioService, block_size: usize) -> f64 {
        let volume_arc = {
            let cm = audio_service.channel_manager().lock().unwrap();
            cm.current_channel().unwrap().volume().clone()
        };
        let mut volume = GainProcessor::new(volume_arc);
        LatencyAnalyzer::measure_effect_added_execution_us(&mut volume, 256, block_size)
    }

    /// Measures the CPU execution cost of every processor in the active DSP chain.
    ///
    /// Runs individual benchmarks for all four processors in signal-chain order and
    /// returns the results as a vector.
    ///
    /// # Arguments
    ///
    /// * `audio_service` — Service snapshot providing channel and master-volume arcs.
    /// * `block_size` — Number of samples per benchmark iteration (recommended: 2 048).
    ///
    /// # Returns
    ///
    /// A `Vec<ExecutionTimingDto>` with exactly four entries, in signal-chain order:
    ///
    /// | Index | Processor |
    /// |---|---|
    /// | 0 | Gain |
    /// | 1 | Tone Stack |
    /// | 2 | Volume |
    /// | 3 | Master Volume |
    ///
    /// [`measure_gain_latency`]: AudioLatencyMeasurementService::measure_gain_latency
    /// [`measure_tone_stack_latency`]: AudioLatencyMeasurementService::measure_tone_stack_latency
    /// [`measure_volume_latency`]: AudioLatencyMeasurementService::measure_volume_latency
    /// [`GainProcessor`]: crate::services::processors::gain::gain_processor::GainProcessor
    pub fn measure_all_dsp_timings(
        audio_service: &AudioService,
        block_size: usize,
    ) -> Vec<ExecutionTimingDto> {
        let gain_us = Self::measure_gain_latency(audio_service, block_size);
        let tone_stack_us = Self::measure_tone_stack_latency(audio_service, block_size);
        let volume_us = Self::measure_volume_latency(audio_service, block_size);
        let master_volume_us = {
            let mut master_volume = GainProcessor::new(audio_service.master_volume().clone());
            LatencyAnalyzer::measure_effect_added_execution_us(&mut master_volume, 256, block_size)
        };

        vec![
            ExecutionTimingDto::new("Gain", gain_us),
            ExecutionTimingDto::new("Tone Stack", tone_stack_us),
            ExecutionTimingDto::new("Volume", volume_us),
            ExecutionTimingDto::new("Master Volume", master_volume_us),
        ]
    }

    /// Returns the algorithmic (design-inherent) delay for every processor in the DSP chain.
    ///
    /// For the current chain (Gain → Tone Stack → Volume → Master Volume) every processor
    /// is a sample-by-sample filter with no lookahead or delay line, so all values are **zero**.
    ///
    /// # Arguments
    ///
    /// * `audio_service` — Used only to read the output sample rate for ms conversion.
    ///
    /// # Returns
    ///
    /// A `Vec<AlgorithmicLatencyDto>` with exactly four entries (Gain, Tone Stack, Volume,
    /// Master Volume), each reporting `latency_samples = 0` and `latency_ms = 0.0`.
    pub fn measure_all_dsp_algorithmic_latency(
        audio_service: &AudioService,
    ) -> Vec<AlgorithmicLatencyDto> {
        let sample_rate_hz = audio_service.audio_handler().output_sample_rate();

        vec![
            AlgorithmicLatencyDto::new("Gain", 0, sample_rate_hz),
            AlgorithmicLatencyDto::new("Tone Stack", 0, sample_rate_hz),
            AlgorithmicLatencyDto::new("Volume", 0, sample_rate_hz),
            AlgorithmicLatencyDto::new("Master Volume", 0, sample_rate_hz),
        ]
    }

    /// Estimates the I/O buffer latency from the current CPAL stream configuration.
    ///
    /// Buffer latency is the delay introduced by the hardware frame buffers: each side
    /// accumulates a full buffer of samples before the driver delivers or accepts them.
    /// The formula is:
    ///
    /// ```text
    /// latency_ms = (buffer_frames / sample_rate_hz) × 1000
    /// ```
    ///
    /// When CPAL is configured with [`BufferSize::Default`] the actual frame count is
    /// unknown at runtime.  In that case a conservative fallback of **256 frames** is
    /// used so the UI can display a practical estimate rather than zero or an error.
    ///
    /// # Arguments
    ///
    /// * `audio_service` — Used to read both stream configs and sample rates.
    ///
    /// # Returns
    ///
    /// A [`BufferLatencyDto`] containing `input_buffer_latency_ms`,
    /// `output_buffer_latency_ms`, and their sum as `total_buffer_latency_ms`.
    ///
    /// [`BufferSize::Default`]: cpal::BufferSize::Default
    /// [`BufferLatencyDto`]: crate::domain::dto::buffer_latency_dto::BufferLatencyDto
    pub fn measure_buffer_latency(audio_service: &AudioService) -> BufferLatencyDto {
        const DEFAULT_BUFFER_FRAMES_FALLBACK: u32 = 256;

        let input_frames = match audio_service.audio_handler().input_config().buffer_size {
            BufferSize::Fixed(frames) => frames,
            BufferSize::Default => DEFAULT_BUFFER_FRAMES_FALLBACK,
        };

        let output_frames = match audio_service.audio_handler().output_config().buffer_size {
            BufferSize::Fixed(frames) => frames,
            BufferSize::Default => DEFAULT_BUFFER_FRAMES_FALLBACK,
        };

        let input_ms = (input_frames as f64
            / audio_service.audio_handler().input_sample_rate() as f64)
            * 1000.0;
        let output_ms = (output_frames as f64
            / audio_service.audio_handler().output_sample_rate() as f64)
            * 1000.0;

        BufferLatencyDto::new(input_ms, output_ms)
    }

    /// Measures true end-to-end round-trip latency using a dedicated pair of CPAL streams.
    ///
    /// Unlike the other measurement functions this one performs a **real-world, hardware
    /// measurement** rather than an analytical estimate.  It delegates to
    /// [`RoundTripLatencySession::run`], which:
    ///
    /// 1. Opens its own private input/output CPAL streams — completely separate from the
    ///    main loopback.
    /// 2. Warms up the streams for 1.5 s so the OS audio stack stabilises.
    /// 3. Calibrates a detection threshold from ambient noise.
    /// 4. Injects three impulses and times how long each takes to return on the input.
    /// 5. Returns the average of the three round-trip durations.
    ///
    /// The caller (`measure_round_trip_latency` Tauri command) is responsible for releasing
    /// the `Mutex<AudioService>` lock and spawning a dedicated thread *before* calling this
    /// function, so the main audio engine and UI remain responsive throughout.
    ///
    /// # Arguments
    ///
    /// * `handler` — The audio I/O factory cloned from [`AudioService`] before the mutex
    ///   was released.  Used only to open the temporary measurement streams.
    ///
    /// # Returns
    ///
    /// A [`RoundTripLatencyDto`] with `is_valid = true` and `latency_ms` set on success,
    /// or `is_valid = false` and a human-readable `error` message on failure.
    ///
    /// # Physical requirement
    ///
    /// The audio output must be physically (or virtually) looped back into the input for
    /// the echo to be detectable.  If it is not, the measurement times out and returns an
    /// error explaining the likely cause.
    ///
    /// [`RoundTripLatencySession::run`]: crate::services::round_trip_latency_session::RoundTripLatencySession::run
    /// [`AudioService`]: crate::services::audio_service::AudioService
    /// [`RoundTripLatencyDto`]: crate::domain::dto::round_trip_latency_dto::RoundTripLatencyDto
    pub fn measure_round_trip_latency(handler: &dyn AudioHandlerTrait) -> RoundTripLatencyDto {
        match RoundTripLatencySession::run(
            handler,
            Duration::from_secs(10),
            Duration::from_millis(2500),
        ) {
            Ok(latency_ms) => RoundTripLatencyDto::success(latency_ms),
            Err(error) => RoundTripLatencyDto::failure(error),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::channel_manager::ChannelManager;
    use crate::infrastructure::audio_handler::MockAudioHandlerTrait;
    use crate::services::audio_service::AudioService;
    use cpal::StreamConfig;
    use std::sync::{Arc, Mutex};

    fn build_service_with_buffer_config(
        input_rate: u32,
        output_rate: u32,
        input_buffer_size: BufferSize,
        output_buffer_size: BufferSize,
    ) -> AudioService {
        let mut mock = MockAudioHandlerTrait::new();

        let input_config = StreamConfig {
            channels: 1,
            sample_rate: input_rate,
            buffer_size: input_buffer_size,
        };

        let output_config = StreamConfig {
            channels: 1,
            sample_rate: output_rate,
            buffer_size: output_buffer_size,
        };

        mock.expect_input_sample_rate().return_const(input_rate);
        mock.expect_output_sample_rate().return_const(output_rate);
        mock.expect_input_config().return_const(input_config);
        mock.expect_output_config().return_const(output_config);

        AudioService::new_with_handler(Arc::new(mock), Arc::new(Mutex::new(ChannelManager::new())))
    }

    fn assert_approx_eq(actual: f64, expected: f64, epsilon: f64) {
        assert!(
            (actual - expected).abs() <= epsilon,
            "expected {actual} ~= {expected} (epsilon {epsilon})"
        );
    }

    #[cfg(test)]
    mod success_path {
        use super::*;

        #[test]
        fn measure_all_dsp_timings_returns_expected_processors() {
            let service = build_service_with_buffer_config(
                48_000,
                48_000,
                BufferSize::Fixed(256),
                BufferSize::Fixed(256),
            );

            let timings = AudioLatencyMeasurementService::measure_all_dsp_timings(&service, 512);

            assert_eq!(timings.len(), 4);
            assert_eq!(timings[0].processor_name, "Gain");
            assert_eq!(timings[1].processor_name, "Tone Stack");
            assert_eq!(timings[2].processor_name, "Volume");
            assert_eq!(timings[3].processor_name, "Master Volume");
            assert!(timings
                .iter()
                .all(|t| t.execution_us_per_sample.is_finite()));
            assert!(timings.iter().all(|t| t.execution_us_per_sample >= 0.0));
        }

        #[test]
        fn measure_all_dsp_algorithmic_latency_is_zero_for_simple_chain() {
            let service = build_service_with_buffer_config(
                48_000,
                48_000,
                BufferSize::Fixed(256),
                BufferSize::Fixed(256),
            );

            let latency =
                AudioLatencyMeasurementService::measure_all_dsp_algorithmic_latency(&service);

            assert_eq!(latency.len(), 4);
            assert_eq!(latency[0].processor_name, "Gain");
            assert_eq!(latency[1].processor_name, "Tone Stack");
            assert_eq!(latency[2].processor_name, "Volume");
            assert_eq!(latency[3].processor_name, "Master Volume");
            assert!(latency.iter().all(|item| item.latency_samples == 0));
            assert!(latency.iter().all(|item| item.latency_ms == 0.0));
        }

        #[test]
        fn measure_buffer_latency_uses_fixed_buffer_sizes() {
            let service = build_service_with_buffer_config(
                48_000,
                96_000,
                BufferSize::Fixed(480),
                BufferSize::Fixed(960),
            );

            let latency = AudioLatencyMeasurementService::measure_buffer_latency(&service);

            assert_approx_eq(latency.input_buffer_latency_ms, 10.0, 1e-9);
            assert_approx_eq(latency.output_buffer_latency_ms, 10.0, 1e-9);
            assert_approx_eq(latency.total_buffer_latency_ms, 20.0, 1e-9);
        }
    }

    #[cfg(test)]
    mod failure_path {
        use super::*;

        #[test]
        fn measure_buffer_latency_falls_back_for_default_buffer_size() {
            let service = build_service_with_buffer_config(
                48_000,
                48_000,
                BufferSize::Default,
                BufferSize::Default,
            );

            let latency = AudioLatencyMeasurementService::measure_buffer_latency(&service);
            let expected_single_side_ms = (256.0 / 48_000.0) * 1000.0;

            assert_approx_eq(
                latency.input_buffer_latency_ms,
                expected_single_side_ms,
                1e-9,
            );
            assert_approx_eq(
                latency.output_buffer_latency_ms,
                expected_single_side_ms,
                1e-9,
            );
            assert_approx_eq(
                latency.total_buffer_latency_ms,
                expected_single_side_ms * 2.0,
                1e-9,
            );
        }
    }
}
