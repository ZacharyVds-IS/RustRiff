use crate::domain::audio_processor::AudioProcessor;
use atomic_float::AtomicF32;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use tracing::info;

/// An audio processor that applies gain with smooth transitions.
///
/// `GainProcessor` implements the [`AudioProcessor`] trait and applies a gain factor
/// to audio samples. When the gain value changes, it smoothly transitions from the
/// current value to the new target using a simple one-pole low-pass filter, preventing
/// audible clicks and pops.
///
/// The gain value is read from an [`Arc<AtomicF32>`] that can be safely updated
/// from other threads (e.g., the UI thread) without requiring locks.
pub struct GainProcessor {
    gain: Arc<AtomicF32>,
    current: f32,
}

impl GainProcessor {
    /// Creates a new `GainProcessor` with the given atomic gain value.
    ///
    /// Initializes the internal smoothing state (`current`) to the initial gain value
    /// loaded from the atomic.
    ///
    /// # Arguments
    ///
    /// * `gain` - An [`Arc<AtomicF32>`] that holds the target gain value.
    ///   This value can be updated from other threads, and changes
    ///   will be smoothly transitioned.
    pub fn new(gain: Arc<AtomicF32>) -> Self {
        info!("initi gain processor");
        let initial = gain.load(Ordering::Relaxed);
        Self {
            gain,
            current: initial,
        }
    }
}

impl AudioProcessor for GainProcessor {
    #[cfg_attr(doc, aquamarine::aquamarine)]
    /// Processes a single audio sample with the current gain factor.
    ///
    /// Reads the target gain value from the atomic and smoothly transitions the
    /// internal state toward it using a one-pole smoothing algorithm.
    ///
    /// ### Gain Smoothing Visualization
    /// The red line shows the instantaneous jump (Target), while the curve
    /// shows the gradual adjustment of the multiplier (Current).
    ///
    /// ```mermaid
    /// xychart-beta
    ///     title "Instantaneous Jump vs. One-Pole Smoothing"
    ///     x-axis "Time (Samples)" [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10]
    ///     y-axis "Gain Factor" 0 --> 1.2
    ///     line [0.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0]
    ///     line [0.0, 0.4, 0.64, 0.78, 0.87, 0.92, 0.95, 0.97, 0.98, 0.99, 1.0]
    /// ```
    ///
    /// # Arguments
    ///
    /// * `sample` - The input audio sample to be scaled by the gain.
    ///
    /// # Returns
    ///
    /// The gain-scaled audio sample.
    fn process(&mut self, sample: f32) -> f32 {
        let target = self.gain.load(Ordering::Relaxed);

        // Simple one-pole smoothing
        self.current += (target - self.current) * 0.001;

        sample * self.current
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use atomic_float::AtomicF32;
    use std::sync::atomic::Ordering;
    use std::sync::Arc;

    fn make_processor(initial_gain: f32) -> GainProcessor {
        let gain = Arc::new(AtomicF32::new(initial_gain));
        GainProcessor::new(gain)
    }

    #[cfg(test)]
    mod success_path {
        use super::*;
        #[test]
        fn transition_to_target_should_be_smooth_up() {
            let gain = Arc::new(AtomicF32::new(0.0));
            let mut processor = GainProcessor::new(gain.clone());

            gain.store(1.0, Ordering::Relaxed);

            for _ in 0..5_000 {
                processor.process(1.0);
            }

            assert!(processor.current > 0.9);
            assert!(processor.current < 1.0);
        }

        #[test]
        fn transition_to_target_should_be_smooth_down() {
            let gain = Arc::new(AtomicF32::new(1.0));
            let mut processor = GainProcessor::new(gain.clone());

            gain.store(0.0, Ordering::Relaxed);

            for _ in 0..5_000 {
                processor.process(1.0);
            }

            assert!(processor.current < 0.1);
            assert!(processor.current > 0.0);
        }

        #[test]
        fn steady_state_does_not_change_value() {
            let mut processor = make_processor(1.0);

            for _ in 0..1_000 {
                processor.process(1.0);
            }

            assert!((processor.current - 1.0).abs() < 1e-6);
        }

        #[test]
        fn output_is_scaled_by_current_gain() {
            let gain = Arc::new(AtomicF32::new(1.0));
            let mut processor = GainProcessor::new(gain.clone());

            gain.store(0.5, Ordering::Relaxed);

            for _ in 0..1_000 {
                processor.process(1.0);
            }

            let output = processor.process(1.0);

            assert!((output - processor.current).abs() < 1e-6);
        }
    }
    //This part of the code does not have a failure path
}
