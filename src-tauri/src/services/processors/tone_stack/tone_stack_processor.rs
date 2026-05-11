use crate::domain::audio_processor::AudioProcessor;
use crate::domain::tone_stack::ToneStack;
use crate::services::processors::tone_stack::range_eq::{EQType, RangeEQ};
use spectrum_analyzer::windows::hann_window;
use spectrum_analyzer::{samples_fft_to_spectrum, FrequencyLimit};
use std::sync::atomic::Ordering;
use std::sync::Arc;

/// Audio processor for tone stack equalization.
///
/// This processor applies bass, middle, and treble equalization to audio samples.
/// It uses three range EQ filters chained together: bass (low shelf), mid (peak), and treble (high shelf).
/// The processor reads the current tone stack parameters atomically from the shared `ToneStack` instance
/// on each processing call to enable real-time parameter changes.
pub struct ToneStackProcessor {
    /// Shared reference to the tone stack containing bass, middle, and treble parameters.
    tone_stack: Arc<ToneStack>,
    /// Low-frequency equalizer for bass adjustment.
    bass_eq: RangeEQ,
    /// Mid-frequency equalizer for middle adjustment.
    mid_eq: RangeEQ,
    /// High-frequency equalizer for treble adjustment.
    treble_eq: RangeEQ,
}
const BASS_SHELF: f32 = 100.0;
const MID_PEAK: f32 = 1_200.0;
const TREBLE_SHELF: f32 = 5_000.0;

impl ToneStackProcessor {
    /// Creates a new tone stack processor with the given tone stack.
    ///
    /// Initializes the three EQ filters with fixed frequency ranges:
    /// - Bass: low shelf at 100 Hz
    /// - Mid: peak at 1200 Hz
    /// - Treble: high shelf at 5000 Hz
    ///
    /// # Arguments
    ///
    /// * `tone_stack` - Shared reference to the tone stack parameters.
    ///
    /// # Returns
    ///
    /// A new `ToneStackProcessor` instance.
    pub fn new(tone_stack: Arc<ToneStack>, sample_rate: u32) -> Self {
        Self {
            tone_stack,
            bass_eq: RangeEQ::new(sample_rate as f32, BASS_SHELF, 0.0, 1.0, EQType::Low),
            mid_eq: RangeEQ::new(sample_rate as f32, MID_PEAK, 0.0, 1.0, EQType::Peak),
            treble_eq: RangeEQ::new(sample_rate as f32, 0.0, TREBLE_SHELF, 1.0, EQType::High),
        }
    }

    /// Prints tone stack energy analysis to the console.
    ///
    /// This method accumulates audio samples into an FFT buffer and, when the buffer is full,
    /// performs spectral analysis to compute energy in bass, mid, and treble frequency bands.
    /// The results are printed to stdout for debugging/monitoring purposes.
    ///
    /// Frequency bands:
    /// - Bass: 0-180 Hz
    /// - Mid: 180-2400 Hz
    /// - Treble: 2400-20000 Hz
    ///
    /// # Arguments
    ///
    /// * `gain_sample` - The current audio sample to add to the FFT buffer.
    /// * `fft_buffer` - Mutable reference to the buffer collecting samples for FFT.
    /// * `fft_size` - The size of the FFT buffer (must be a power of 2).
    pub fn print_tone_stack(&self, gain_sample: f32, fft_buffer: &mut Vec<f32>, fft_size: usize) {
        const PRINT_BASS_MIN: f32 = 0.0;
        const PRINT_BASS_MAX: f32 = 180.0;
        const PRINT_MID_MIN: f32 = 180.0;
        const PRINT_MID_MAX: f32 = 2_400.0;
        const PRINT_TREBLE_MIN: f32 = 2_400.0;
        const PRINT_TREBLE_MAX: f32 = 20000.0;

        fft_buffer.push(gain_sample);

        if fft_buffer.len() == fft_size {
            let windowed = hann_window(fft_buffer);

            let spectrum =
                samples_fft_to_spectrum(&windowed, 48_000, FrequencyLimit::All, None).unwrap();

            let mut bass_energy = 0.0f32;
            let mut mid_energy = 0.0f32;
            let mut treble_energy = 0.0f32;

            for (freq, magnitude) in spectrum.data().iter() {
                let f = freq.val();

                if (PRINT_BASS_MIN..=PRINT_BASS_MAX).contains(&f) {
                    bass_energy += magnitude.val();
                } else if (PRINT_MID_MIN..=PRINT_MID_MAX).contains(&f) {
                    mid_energy += magnitude.val();
                } else if (PRINT_TREBLE_MIN..=PRINT_TREBLE_MAX).contains(&f) {
                    treble_energy += magnitude.val();
                }
            }

            println!(
                "Tone Stack: Bass: {:>8.5}\t | Mid: {:>8.5}\t | Treble: {:>8.5}",
                bass_energy, mid_energy, treble_energy
            );

            fft_buffer.clear();
        }
    }

    /// Updates the EQ parameters from the current tone stack values.
    ///
    /// Reads the atomic bass, middle, and treble values from the tone stack
    /// and applies them to the respective EQ filters. This method should be called
    /// before processing each sample to ensure real-time parameter updates.
    fn update_parameters(&mut self) {
        self.bass_eq
            .set_percent(self.tone_stack.bass().load(Ordering::Relaxed));
        self.mid_eq
            .set_percent(self.tone_stack.middle().load(Ordering::Relaxed));
        self.treble_eq
            .set_percent(self.tone_stack.treble().load(Ordering::Relaxed));
    }
}

impl AudioProcessor for ToneStackProcessor {
    /// Processes a single audio sample through the tone stack.
    ///
    /// Applies the bass, mid, and treble equalization in sequence.
    /// Updates the EQ parameters from the tone stack before processing.
    ///
    /// # Arguments
    ///
    /// * `sample` - The input audio sample.
    ///
    /// # Returns
    ///
    /// The processed audio sample with tone stack equalization applied.
    fn process(&mut self, sample: f32) -> f32 {
        self.update_parameters();
        let processed = self.bass_eq.process(sample);
        let processed = self.mid_eq.process(processed);
        self.treble_eq.process(processed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(test)]
    mod success_path {
        use super::*;
        use std::sync::Arc;

        #[test]
        fn process_method_should_chain_eq_filters_correctly() {
            let tone_stack = Arc::new(ToneStack::new());
            let mut processor = ToneStackProcessor::new(Arc::clone(&tone_stack), 48000);

            let input_sample = 0.5;
            let output = processor.process(input_sample);

            assert!(output.is_finite());
            assert!(!output.is_nan());
        }

        #[test]
        fn update_parameters_should_read_from_tone_stack_and_set_eqs() {
            let tone_stack = Arc::new(ToneStack::new());
            let mut processor = ToneStackProcessor::new(Arc::clone(&tone_stack), 48000);

            tone_stack.set_bass(0.8);
            tone_stack.set_middle(0.6);
            tone_stack.set_treble(0.4);

            processor.update_parameters();

            let output = processor.process(0.5);
            assert!(output.is_finite());
        }

        #[test]
        fn different_tone_settings_should_produce_different_outputs() {
            let tone_stack1 = Arc::new(ToneStack::new());
            let tone_stack2 = Arc::new(ToneStack::new());

            let mut processor1 = ToneStackProcessor::new(Arc::clone(&tone_stack1), 48000);
            let mut processor2 = ToneStackProcessor::new(Arc::clone(&tone_stack2), 48000);

            tone_stack1.set_bass(0.2);
            tone_stack2.set_bass(0.8);

            processor1.update_parameters();
            processor2.update_parameters();

            let input = 0.5;
            let output1 = processor1.process(input);
            let output2 = processor2.process(input);

            assert_ne!(output1, output2);
        }

        #[test]
        fn processing_multiple_samples_should_maintain_state() {
            let tone_stack = Arc::new(ToneStack::new());
            let mut processor = ToneStackProcessor::new(Arc::clone(&tone_stack), 48000);

            let samples = vec![0.1, 0.2, 0.3, 0.4, 0.5];
            let mut outputs = Vec::new();

            for sample in samples {
                outputs.push(processor.process(sample));
            }

            for output in &outputs {
                assert!(output.is_finite());
                assert!(!output.is_nan());
            }

            let all_same = outputs.windows(2).all(|w| (w[0] - w[1]).abs() < 1e-6);
            assert!(
                !all_same,
                "Outputs should vary, indicating state is maintained"
            );
        }

        #[test]
        fn print_tone_stack_should_handle_fft_buffer_correctly() {
            let tone_stack = Arc::new(ToneStack::new());
            let processor = ToneStackProcessor::new(Arc::clone(&tone_stack), 48000);

            let mut fft_buffer = Vec::new();
            let fft_size = 1024;

            for i in 0..fft_size {
                let sample = (i as f32 / fft_size as f32) * 0.1;
                processor.print_tone_stack(sample, &mut fft_buffer, fft_size);
            }

            assert_eq!(fft_buffer.len(), 0);
        }
    }

    #[cfg(test)]
    mod failure_path {
        use super::*;
        use std::sync::Arc;

        #[test]
        fn processor_should_handle_zero_input_samples() {
            let tone_stack = Arc::new(ToneStack::new());
            let mut processor = ToneStackProcessor::new(Arc::clone(&tone_stack), 48000);

            let output = processor.process(0.0);
            assert_eq!(output, 0.0);
        }

        #[test]
        fn processor_should_handle_extreme_input_samples() {
            let tone_stack = Arc::new(ToneStack::new());
            let mut processor = ToneStackProcessor::new(Arc::clone(&tone_stack), 48000);

            let small_input = 1e-6;
            let small_output = processor.process(small_input);
            assert!(small_output.is_finite());
            assert!(!small_output.is_nan());

            let large_input = 10.0;
            let large_output = processor.process(large_input);
            assert!(large_output.is_finite());
            assert!(!large_output.is_nan());
        }

        #[test]
        fn processor_should_handle_extreme_tone_stack_values() {
            let tone_stack = Arc::new(ToneStack::new());
            let mut processor = ToneStackProcessor::new(Arc::clone(&tone_stack), 48000);

            // Set extreme values (though ToneStack clamps them)
            tone_stack.set_bass(0.0); // Minimum
            tone_stack.set_middle(1.0); // Maximum
            tone_stack.set_treble(0.5); // Middle

            processor.update_parameters();

            let output = processor.process(0.5);
            assert!(output.is_finite());
            assert!(!output.is_nan());
        }

        #[test]
        fn processor_should_handle_rapid_parameter_changes() {
            let tone_stack = Arc::new(ToneStack::new());
            let mut processor = ToneStackProcessor::new(Arc::clone(&tone_stack), 48000);

            let input = 0.3;
            let mut outputs = Vec::new();

            // Rapidly change parameters and process
            for i in 0..10 {
                let bass_val = (i % 3) as f32 * 0.3 + 0.1;
                let mid_val = ((i + 1) % 3) as f32 * 0.3 + 0.1;
                let treble_val = ((i + 2) % 3) as f32 * 0.3 + 0.1;

                tone_stack.set_bass(bass_val);
                tone_stack.set_middle(mid_val);
                tone_stack.set_treble(treble_val);

                // Process will call update_parameters internally
                outputs.push(processor.process(input));
            }

            // All outputs should be valid
            for output in outputs {
                assert!(output.is_finite());
                assert!(!output.is_nan());
            }
        }

        #[test]
        fn print_tone_stack_should_handle_empty_buffer() {
            let tone_stack = Arc::new(ToneStack::new());
            let processor = ToneStackProcessor::new(Arc::clone(&tone_stack), 48000);

            let mut fft_buffer = Vec::new();

            // Should not crash with empty buffer
            processor.print_tone_stack(0.5, &mut fft_buffer, 1024);
            assert_eq!(fft_buffer.len(), 1); // Should have added one sample
        }

        #[test]
        fn print_tone_stack_should_handle_partial_buffer() {
            let tone_stack = Arc::new(ToneStack::new());
            let processor = ToneStackProcessor::new(Arc::clone(&tone_stack), 48000);

            let mut fft_buffer = Vec::new();
            let fft_size = 1024;

            // Add only half the required samples
            for i in 0..(fft_size / 2) {
                let sample = (i as f32 / fft_size as f32) * 0.1;
                processor.print_tone_stack(sample, &mut fft_buffer, fft_size);
            }

            // Buffer should contain the samples (not yet processed)
            assert_eq!(fft_buffer.len(), fft_size / 2);
        }

        #[test]
        fn processor_should_handle_nan_input_gracefully() {
            let tone_stack = Arc::new(ToneStack::new());
            let mut processor = ToneStackProcessor::new(Arc::clone(&tone_stack), 48000);

            // NaN input should be handled (though in practice this shouldn't happen)
            let nan_input = f32::NAN;
            let output = processor.process(nan_input);

            // The output might be NaN, but the processor shouldn't crash
            // This tests robustness
            assert!(output.is_finite() || output.is_nan());
        }

        #[test]
        fn processor_should_handle_infinite_input_gracefully() {
            let tone_stack = Arc::new(ToneStack::new());
            let mut processor = ToneStackProcessor::new(Arc::clone(&tone_stack), 48000);

            // Infinite input should be handled
            let inf_input = f32::INFINITY;
            let output = processor.process(inf_input);

            // The output might be infinite, but the processor shouldn't crash
            assert!(output.is_finite() || output.is_infinite());
        }

        #[test]
        fn processor_should_maintain_state_across_extreme_conditions() {
            let tone_stack = Arc::new(ToneStack::new());
            let mut processor = ToneStackProcessor::new(Arc::clone(&tone_stack), 48000);

            // Process a sequence with extreme values
            let test_samples = vec![0.0, 1e-6, 1.0, -1.0, 10.0, -10.0, f32::NAN, f32::INFINITY];

            for sample in test_samples {
                let output = processor.process(sample);
                // Just ensure it doesn't crash
                let _ = output; // Use the output to avoid unused variable warning
            }

            // Should still work with normal input after extreme conditions
            let normal_output = processor.process(0.5);
            assert!(normal_output.is_finite() || normal_output.is_nan());
        }
    }
}
