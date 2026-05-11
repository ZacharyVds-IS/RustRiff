use std::f32::consts::PI;

/// Types of shelf filters supported by the biquad filter.
pub enum ShelfType {
    /// Low shelf filter, boosts or cuts frequencies below the cutoff.
    Low,
    /// High shelf filter, boosts or cuts frequencies above the cutoff.
    High,
    /// Peak filter, boosts or cuts a narrow band of frequencies around the center.
    Peak,
}

/// A biquad filter implementation for audio equalization.
///
/// This struct implements a second-order IIR filter using the biquad algorithm.
/// It maintains internal state (x1, x2, y1, y2) for processing continuous audio streams.
/// The filter coefficients are calculated based on the RBJ Audio EQ Cookbook formulas.
///
/// The filter supports shelf and peak responses for tone stack equalization.
pub struct Biquad {
    /// Feedforward coefficient b0.
    b0: f32,
    /// Feedforward coefficient b1.
    b1: f32,
    /// Feedforward coefficient b2.
    b2: f32,
    /// Feedback coefficient a1.
    a1: f32,
    /// Feedback coefficient a2.
    a2: f32,
    /// Previous input sample x[n-1].
    x1: f32,
    /// Previous previous input sample x[n-2].
    x2: f32,
    /// Previous output sample y[n-1].
    y1: f32,
    /// Previous previous output sample y[n-2].
    y2: f32,
    /// Sample rate of the audio signal.
    sample_rate: f32,
    /// Center/cutoff frequency of the filter.
    freq: f32,
    /// Type of shelf filter.
    shelf_type: ShelfType,
}

// Biquad shelf filter implementation based on RBJ Audio EQ Cookbook
// Reference: https://www.w3.org/2011/audio/audio-eq-cookbook.html
impl Biquad {
    /// Creates a new biquad shelf filter with the specified parameters.
    ///
    /// # Arguments
    ///
    /// * `shelf` - The type of shelf filter (Low, High, or Peak).
    /// * `sample_rate` - The sample rate of the audio signal (e.g., 44100.0).
    /// * `freq` - The center/cutoff frequency in Hz.
    /// * `gain_db` - The gain in decibels (positive for boost, negative for cut).
    ///
    /// # Returns
    ///
    /// A new `Biquad` instance configured with the calculated coefficients.
    pub fn new_shelf(shelf: ShelfType, sample_rate: f32, freq: f32, gain_db: f32) -> Self {
        let (b0, b1, b2, _a0, a1, a2) =
            Self::calculate_coefficients(&shelf, sample_rate, freq, gain_db);

        Self {
            b0,
            b1,
            b2,
            a1,
            a2,
            x1: 0.0,
            x2: 0.0,
            y1: 0.0,
            y2: 0.0,
            sample_rate,
            freq,
            shelf_type: shelf,
        }
    }

    /// Processes a single audio sample through the biquad filter.
    ///
    /// This method applies the filter to the input sample and updates the internal state
    /// for the next sample. It should be called for each sample in sequence.
    ///
    /// # Arguments
    ///
    /// * `x` - The input audio sample.
    ///
    /// # Returns
    ///
    /// The filtered output sample.
    pub fn process(&mut self, x: f32) -> f32 {
        let y = self.b0 * x + self.b1 * self.x1 + self.b2 * self.x2
            - self.a1 * self.y1
            - self.a2 * self.y2;
        self.x2 = self.x1;
        self.x1 = x;
        self.y2 = self.y1;
        self.y1 = y;
        y
    }

    /// Updates the gain of the biquad filter.
    ///
    /// Recalculates the filter coefficients based on the new gain value while keeping
    /// the same frequency and shelf type. The internal state is preserved.
    ///
    /// # Arguments
    ///
    /// * `gain_db` - The new gain in decibels.
    pub fn set_gain_db(&mut self, gain_db: f32) {
        let (b0, b1, b2, _a0, a1, a2) =
            Self::calculate_coefficients(&self.shelf_type, self.sample_rate, self.freq, gain_db);

        self.b0 = b0;
        self.b1 = b1;
        self.b2 = b2;
        self.a1 = a1;
        self.a2 = a2;
        /*
        println!(
            "Updated Biquad Coefficients: b0: {:.6}\t  b1: {:.6}\t  b2: {:.6}\t  a1: {:.6}\t  a2: {:.6}\t  a0: {:.6} (should always be 1.0) || Gain (dB): {:.2}",
            self.b0, self.b1, self.b2, self.a1, self.a2, a0, gain_db
        );
         */
    }

    /// Calculates the biquad filter coefficients based on the RBJ Audio EQ Cookbook.
    ///
    /// This function computes the feedforward (b0, b1, b2) and feedback (a0, a1, a2) coefficients
    /// for the specified filter type, sample rate, frequency, and gain. The coefficients are
    /// normalized so that a0 = 1.0.
    ///
    /// # Arguments
    ///
    /// * `shelf` - The type of shelf filter.
    /// * `sample_rate` - The sample rate of the audio signal.
    /// * `freq` - The center/cutoff frequency in Hz.
    /// * `gain_db` - The gain in decibels.
    ///
    /// # Returns
    ///
    /// A tuple of normalized coefficients: (b0, b1, b2, a0, a1, a2) where a0 is always 1.0.
    /// Reference: https://www.w3.org/2011/audio/audio-eq-cookbook.html
    fn calculate_coefficients(
        shelf: &ShelfType,
        sample_rate: f32,
        freq: f32,
        gain_db: f32,
    ) -> (f32, f32, f32, f32, f32, f32) {
        let a = 10.0_f32.powf(gain_db / 40.0);
        let w0 = 2.0 * PI * freq / sample_rate;
        let cos = w0.cos();
        let sin = w0.sin();

        let (b0, b1, b2, a0, a1, a2) = match shelf {
            ShelfType::Low | ShelfType::High => {
                let alpha = sin / 16.0 * (2.0 * (a + 1.0 / a)).sqrt();
                let sqrt_a = a.sqrt();
                if matches!(shelf, ShelfType::Low) {
                    (
                        a * ((a + 1.0) - (a - 1.0) * cos + 2.0 * sqrt_a * alpha),
                        2.0 * a * ((a - 1.0) - (a + 1.0) * cos),
                        a * ((a + 1.0) - (a - 1.0) * cos - 2.0 * sqrt_a * alpha),
                        (a + 1.0) + (a - 1.0) * cos + 2.0 * sqrt_a * alpha,
                        -2.0 * ((a - 1.0) + (a + 1.0) * cos),
                        (a + 1.0) + (a - 1.0) * cos - 2.0 * sqrt_a * alpha,
                    )
                } else {
                    (
                        a * ((a + 1.0) + (a - 1.0) * cos + 2.0 * sqrt_a * alpha),
                        -2.0 * a * ((a - 1.0) + (a + 1.0) * cos),
                        a * ((a + 1.0) + (a - 1.0) * cos - 2.0 * sqrt_a * alpha),
                        (a + 1.0) - (a - 1.0) * cos + 2.0 * sqrt_a * alpha,
                        2.0 * ((a - 1.0) - (a + 1.0) * cos),
                        (a + 1.0) - (a - 1.0) * cos - 2.0 * sqrt_a * alpha,
                    )
                }
            }
            ShelfType::Peak => {
                let alpha = sin / 8.0;
                (
                    1.0 + alpha * a,
                    -2.0 * cos,
                    1.0 - alpha * a,
                    1.0 + alpha / a,
                    -2.0 * cos,
                    1.0 - alpha / a,
                )
            }
        };
        //Normalize Coefficients so that a0 is 1
        (b0 / a0, b1 / a0, b2 / a0, 1.0, a1 / a0, a2 / a0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(test)]
    mod success_path {
        use super::*;

        #[test]
        fn test_process_single_sample() {
            let mut biquad = Biquad::new_shelf(ShelfType::Peak, 44100.0, 1000.0, 0.0);
            let input = 0.5;
            let output = biquad.process(input);
            assert!(!output.is_nan());
            assert!(!output.is_infinite());
        }

        #[test]
        fn test_process_multiple_samples() {
            let mut biquad = Biquad::new_shelf(ShelfType::Low, 44100.0, 200.0, 6.0);
            let samples = vec![0.1, 0.2, 0.3, -0.1, -0.2];

            for sample in samples {
                let output = biquad.process(sample);
                assert!(!output.is_nan());
                assert!(!output.is_infinite());
                // Verify state is being updated
                assert_eq!(biquad.x1, sample);
            }
        }

        #[test]
        fn test_set_gain_db() {
            let mut biquad = Biquad::new_shelf(ShelfType::Peak, 44100.0, 1000.0, 0.0);
            let initial_b0 = biquad.b0;

            biquad.set_gain_db(6.0);
            assert_ne!(biquad.b0, initial_b0);
            assert!(!biquad.b0.is_nan());
            assert!(!biquad.b0.is_infinite());
        }

        #[test]
        fn test_gain_zero_db_peak_filter() {
            let mut biquad = Biquad::new_shelf(ShelfType::Peak, 44100.0, 1000.0, 0.0);
            let input = 1.0;
            let output = biquad.process(input);
            // With 0dB gain on first sample, should be close to input
            assert!(output.abs() < 1.5);
        }

        #[test]
        fn test_positive_gain_modification() {
            let mut biquad = Biquad::new_shelf(ShelfType::Low, 44100.0, 100.0, 0.0);
            biquad.set_gain_db(12.0);
            let input = 1.0;

            let output = biquad.process(input);
            assert!(!output.is_nan());
            assert!(!output.is_infinite());
            assert!(output.abs() > input);
        }

        #[test]
        fn test_negative_gain_modification() {
            let mut biquad = Biquad::new_shelf(ShelfType::High, 44100.0, 8000.0, 0.0);
            biquad.set_gain_db(-12.0);
            let input = 1.0;

            let output = biquad.process(input);
            assert!(!output.is_nan());
            assert!(!output.is_infinite());
            assert!(output.abs() < input);
        }

        #[test]
        fn test_state_update_after_process() {
            let mut biquad = Biquad::new_shelf(ShelfType::Peak, 44100.0, 1000.0, 6.0);

            biquad.process(0.5);
            assert_eq!(biquad.x1, 0.5);
            assert_eq!(biquad.x2, 0.0);

            biquad.process(0.3);
            assert_eq!(biquad.x1, 0.3);
            assert_eq!(biquad.x2, 0.5);
        }

        #[test]
        fn test_different_sample_rates() {
            let sample_rates = vec![22050.0, 44100.0, 48000.0, 96000.0];

            for sr in sample_rates {
                let biquad = Biquad::new_shelf(ShelfType::Peak, sr, 1000.0, 6.0);
                assert_eq!(biquad.sample_rate, sr);
            }
        }

        #[test]
        fn test_different_frequencies() {
            let frequencies = vec![20.0, 100.0, 1000.0, 10000.0, 20000.0];

            for freq in frequencies {
                let biquad = Biquad::new_shelf(ShelfType::Peak, 44100.0, freq, 6.0);
                assert_eq!(biquad.freq, freq);
            }
        }
    }
    #[cfg(test)]
    mod failure_path {
        use super::*;

        #[test]
        fn test_extreme_frequency_high() {
            let biquad = Biquad::new_shelf(ShelfType::Peak, 44100.0, 22000.0, 6.0);
            assert!(!biquad.b0.is_nan());
            assert!(!biquad.b0.is_infinite());
        }

        #[test]
        fn test_extreme_frequency_low() {
            let biquad = Biquad::new_shelf(ShelfType::Peak, 44100.0, 1.0, 6.0);
            assert!(!biquad.b0.is_nan());
            assert!(!biquad.b0.is_infinite());
        }

        #[test]
        fn test_very_high_gain() {
            let biquad = Biquad::new_shelf(ShelfType::Peak, 44100.0, 1000.0, 48.0);
            assert!(!biquad.b0.is_nan());
            assert!(!biquad.b0.is_infinite());
        }

        #[test]
        fn test_very_negative_gain() {
            let biquad = Biquad::new_shelf(ShelfType::Peak, 44100.0, 1000.0, -48.0);
            assert!(!biquad.b0.is_nan());
            assert!(!biquad.b0.is_infinite());
        }

        #[test]
        fn test_process_zero_input() {
            let mut biquad = Biquad::new_shelf(ShelfType::Peak, 44100.0, 1000.0, 6.0);
            let output = biquad.process(0.0);
            assert!(!output.is_nan());
            assert!(!output.is_infinite());
        }

        #[test]
        fn test_process_very_small_input() {
            let mut biquad = Biquad::new_shelf(ShelfType::Peak, 44100.0, 1000.0, 6.0);
            let output = biquad.process(1e-6);
            assert!(!output.is_nan());
            assert!(!output.is_infinite());
        }

        #[test]
        fn test_process_large_input() {
            let mut biquad = Biquad::new_shelf(ShelfType::Peak, 44100.0, 1000.0, 6.0);
            let output = biquad.process(100.0);
            assert!(!output.is_nan());
            assert!(!output.is_infinite());
        }

        #[test]
        fn test_all_shelf_types_with_extreme_values() {
            let shelf_types = vec![ShelfType::Low, ShelfType::High, ShelfType::Peak];

            for shelf_type in shelf_types {
                let biquad = Biquad::new_shelf(shelf_type, 44100.0, 5000.0, 24.0);
                assert!(!biquad.b0.is_nan());
                assert!(!biquad.b1.is_nan());
                assert!(!biquad.b2.is_nan());
                assert!(!biquad.a1.is_nan());
                assert!(!biquad.a2.is_nan());
            }
        }

        #[test]
        fn test_set_gain_with_extreme_values() {
            let mut biquad = Biquad::new_shelf(ShelfType::Peak, 44100.0, 1000.0, 0.0);

            biquad.set_gain_db(36.0);
            assert!(!biquad.b0.is_nan());

            biquad.set_gain_db(-36.0);
            assert!(!biquad.b0.is_nan());
        }

        #[test]
        fn test_process_after_multiple_gain_changes() {
            let mut biquad = Biquad::new_shelf(ShelfType::Low, 44100.0, 100.0, 0.0);

            biquad.set_gain_db(6.0);
            let out1 = biquad.process(0.1);

            biquad.set_gain_db(-6.0);
            let out2 = biquad.process(0.1);

            biquad.set_gain_db(12.0);
            let out3 = biquad.process(0.1);

            assert!(!out1.is_nan() && !out2.is_nan() && !out3.is_nan());
        }

        #[test]
        fn test_low_sample_rate() {
            let biquad = Biquad::new_shelf(ShelfType::Peak, 8000.0, 500.0, 6.0);
            assert!(!biquad.b0.is_nan());
            assert!(!biquad.b0.is_infinite());
        }

        #[test]
        fn test_high_sample_rate() {
            let biquad = Biquad::new_shelf(ShelfType::Peak, 192000.0, 50000.0, 6.0);
            assert!(!biquad.b0.is_nan());
            assert!(!biquad.b0.is_infinite());
        }
    }
}
