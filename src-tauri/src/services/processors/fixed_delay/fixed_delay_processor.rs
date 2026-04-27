use crate::domain::audio_processor::AudioProcessor;
use std::collections::VecDeque;

/// A deterministic test processor that delays audio by `delay_samples`.
///
/// This is useful as a reference when validating latency measurements:
/// the expected measured latency should match `delay_samples`.
pub struct FixedDelayProcessor {
    buffer: VecDeque<f32>,
    delay_samples: usize,
}

impl FixedDelayProcessor {
    pub fn new(delay_samples: usize) -> Self {
        let mut buffer = VecDeque::with_capacity(delay_samples + 1);
        for _ in 0..delay_samples {
            buffer.push_back(0.0);
        }

        Self {
            buffer,
            delay_samples,
        }
    }

    pub fn delay_samples(&self) -> usize {
        self.delay_samples
    }
}

impl AudioProcessor for FixedDelayProcessor {
    fn process(&mut self, sample: f32) -> f32 {
        self.buffer.push_back(sample);
        self.buffer.pop_front().unwrap_or(0.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn outputs_silence_for_the_configured_delay_before_signal_arrives() {
        let mut processor = FixedDelayProcessor::new(3);

        assert_eq!(processor.process(1.0), 0.0);
        assert_eq!(processor.process(0.0), 0.0);
        assert_eq!(processor.process(0.0), 0.0);
        assert_eq!(processor.process(0.0), 1.0);
    }

    #[test]
    fn zero_delay_behaves_as_passthrough() {
        let mut processor = FixedDelayProcessor::new(0);

        assert_eq!(processor.process(0.75), 0.75);
        assert_eq!(processor.process(-0.25), -0.25);
    }
}

