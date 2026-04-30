//! Low-level benchmark utility for measuring DSP processor CPU cost.
//!
//! [`LatencyAnalyzer`] provides the timing primitives used by
//! [`AudioLatencyMeasurementService`] to derive per-processor execution costs.
//! It is intentionally a pure measurement tool with no knowledge of audio routing
//! or the service layer.
//!
//! # Methodology
//!
//! Both functions run the processor (or a zero-work passthrough) over
//! `iterations × block_size` samples, alternating between `+0.5` and `-0.5` inputs to
//! exercise any branch-dependent code paths.  [`std::hint::black_box`] is used on each
//! output to prevent the compiler from optimising the loop body away.
//!
//! The *net* cost reported by [`measure_effect_added_execution_us`] is:
//!
//! ```text
//! net_us_per_sample = max(effect_us_per_sample − passthrough_us_per_sample, 0)
//! ```
//!
//! Clamping to `≥ 0` prevents occasional negative readings caused by CPU scheduling
//! noise on the passthrough run.
//!
//! [`AudioLatencyMeasurementService`]: crate::services::audio_latency_measurement_service::AudioLatencyMeasurementService
//! [`measure_effect_added_execution_us`]: LatencyAnalyzer::measure_effect_added_execution_us

use crate::domain::audio_processor::AudioProcessor;
use std::hint::black_box;
use std::time::Instant;

/// Stateless benchmark utility for measuring DSP processor CPU execution cost.
pub struct LatencyAnalyzer;

impl LatencyAnalyzer {
    /// Measures the average wall-clock execution time of a processor in µs per sample.
    ///
    /// Runs `effect` over `iterations × block_size` synthetic samples and returns the
    /// mean time spent per sample.  The input alternates between `+0.5` and `-0.5` to
    /// exercise both halves of any branch-dependent code, and [`black_box`] prevents
    /// dead-code elimination of the loop body.
    ///
    /// Returns `0.0` immediately if `iterations × block_size` overflows or is zero.
    ///
    /// # Arguments
    ///
    /// * `effect` — The processor to benchmark.  Mutable because processors may carry
    ///   internal filter state that updates on every sample.
    /// * `iterations` — Number of full `block_size` passes to run.
    /// * `block_size` — Samples per iteration.  Larger values reduce timer-call overhead
    ///   relative to actual processing; 256–2 048 is a practical range.
    ///
    /// # Returns
    ///
    /// Total wall-clock time divided by total samples, in **microseconds per sample**.
    pub fn measure_processor_execution_us<E: AudioProcessor>(
        effect: &mut E,
        iterations: usize,
        block_size: usize,
    ) -> f64 {
        let total_samples = iterations.saturating_mul(block_size);
        if total_samples == 0 {
            return 0.0;
        }

        let started = Instant::now();
        for sample_index in 0..total_samples {
            let input_sample = if sample_index % 2 == 0 { 0.5 } else { -0.5 };
            let processed_sample = effect.process(input_sample);
            black_box(processed_sample);
        }

        let total_us = started.elapsed().as_secs_f64() * 1_000_000.0;
        total_us / total_samples as f64
    }

    /// Measures the *net* CPU cost added by a processor, relative to a zero-work passthrough.
    ///
    /// Runs [`measure_processor_execution_us`] twice — once for a [`PassthroughProcessor`]
    /// that simply returns its input unchanged, and once for `effect` — then subtracts the
    /// baseline.  The passthrough baseline accounts for loop overhead, `Instant::now()` cost,
    /// and `black_box` calls, so the returned value reflects only the processor's own work.
    ///
    /// The result is clamped to `≥ 0.0` to avoid negative readings from measurement noise
    /// when the processor is extremely cheap (sub-nanosecond per sample).
    ///
    /// # Arguments
    ///
    /// * `effect` — The processor under test.
    /// * `iterations` — Number of benchmark iterations (passed to
    ///   [`measure_processor_execution_us`]).
    /// * `block_size` — Samples per iteration.
    ///
    /// # Returns
    ///
    /// Net added execution cost in **microseconds per sample** (µs/sample), `≥ 0`.
    ///
    /// [`measure_processor_execution_us`]: LatencyAnalyzer::measure_processor_execution_us
    /// [`PassthroughProcessor`]: PassthroughProcessor
    pub fn measure_effect_added_execution_us<E: AudioProcessor>(
        effect: &mut E,
        iterations: usize,
        block_size: usize,
    ) -> f64 {
        let mut passthrough = PassthroughProcessor;

        let baseline_us =
            Self::measure_processor_execution_us(&mut passthrough, iterations, block_size);
        let effect_us = Self::measure_processor_execution_us(effect, iterations, block_size);

        (effect_us - baseline_us).max(0.0)
    }
}

/// Zero-work [`AudioProcessor`] used as the baseline in
/// [`LatencyAnalyzer::measure_effect_added_execution_us`].
///
/// Returns every sample unchanged.  Its execution cost represents pure loop and
/// timer overhead rather than any meaningful DSP work.
struct PassthroughProcessor;

impl AudioProcessor for PassthroughProcessor {
    fn process(&mut self, sample: f32) -> f32 {
        sample
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::processors::fixed_delay::fixed_delay_processor::FixedDelayProcessor;

    struct BusyProcessor;

    impl AudioProcessor for BusyProcessor {
        fn process(&mut self, sample: f32) -> f32 {
            let mut value = sample;
            for _ in 0..64 {
                value = (value * 0.99).sin();
            }
            value
        }
    }

    #[test]
    fn zero_workload_returns_zero_microseconds() {
        let mut passthrough = FixedDelayProcessor::new(0);
        let measured = LatencyAnalyzer::measure_processor_execution_us(&mut passthrough, 0, 1024);

        assert_eq!(measured, 0.0);
    }

    #[test]
    fn busy_processor_adds_non_zero_execution_time() {
        let mut busy = BusyProcessor;
        let added_us = LatencyAnalyzer::measure_effect_added_execution_us(&mut busy, 128, 1024);

        assert!(added_us > 0.0);
    }
}
