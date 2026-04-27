use crate::domain::audio_processor::AudioProcessor;
use std::hint::black_box;
use std::time::Instant;

pub struct LatencyAnalyzer;

impl LatencyAnalyzer {
    /// Measures average execution time in microseconds per processed sample.
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

    /// Measures additional execution cost in microseconds per sample, relative to passthrough.
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
