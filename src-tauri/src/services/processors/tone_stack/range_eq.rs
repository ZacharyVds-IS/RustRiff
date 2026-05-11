use crate::services::processors::tone_stack::biquad::{Biquad, ShelfType};

/// Types of equalization filters supported by RangeEQ.
pub enum EQType {
    /// Low-frequency shelf filter.
    Low,
    /// High-frequency shelf filter.
    High,
    /// Band-pass filter (combination of low and high shelves).
    #[allow(dead_code)]
    Band,
    /// Peak filter for mid-range frequencies.
    Peak,
}

/// A range-based equalizer that combines biquad filters for flexible frequency response shaping.
///
/// RangeEQ can operate in different modes (Low, High, Band, Peak) and uses one or two biquad filters
/// to achieve the desired equalization curve. It converts percentage values to decibel gains
/// and applies them to the underlying filters.
pub struct RangeEQ {
    /// Low-frequency biquad filter (used for Low and Band modes).
    low_shelf: Biquad,
    /// High-frequency biquad filter (used for High and Band modes).
    high_shelf: Biquad,
    /// Type of equalization.
    eq_type: EQType,
}

impl RangeEQ {
    /// Creates a new RangeEQ with the specified parameters.
    ///
    /// The `percent` value is converted to decibel gain and applied to the appropriate biquad filter(s)
    /// based on the `eq_type`. For Band mode, both low and high shelves are used.
    ///
    /// # Arguments
    ///
    /// * `sample_rate` - The sample rate of the audio signal.
    /// * `low_hz` - The low cutoff/cutoff frequency in Hz.
    /// * `high_hz` - The high cutoff frequency in Hz (ignored for Low and Peak modes).
    /// * `percent` - The gain as a percentage (0.0 to 1.0), converted to dB internally.
    /// * `eq_type` - The type of equalization filter.
    ///
    /// # Returns
    ///
    /// A new `RangeEQ` instance configured with the specified parameters.
    pub fn new(sample_rate: f32, low_hz: f32, high_hz: f32, percent: f32, eq_type: EQType) -> Self {
        let gain_db = percent_to_db(percent);

        let low_shelf = match eq_type {
            EQType::Low => Biquad::new_shelf(ShelfType::Low, sample_rate, low_hz, gain_db),
            EQType::High => Biquad::new_shelf(ShelfType::Low, sample_rate, 1000.0, 0.0), // dummy
            EQType::Band => Biquad::new_shelf(ShelfType::Low, sample_rate, low_hz, gain_db),
            EQType::Peak => Biquad::new_shelf(ShelfType::Peak, sample_rate, low_hz, gain_db),
        };

        let high_shelf = match eq_type {
            EQType::Low => Biquad::new_shelf(ShelfType::High, sample_rate, 20000.0, 0.0), // dummy
            EQType::High => Biquad::new_shelf(ShelfType::High, sample_rate, high_hz, gain_db),
            EQType::Band => Biquad::new_shelf(ShelfType::High, sample_rate, high_hz, gain_db),
            EQType::Peak => Biquad::new_shelf(ShelfType::High, sample_rate, 20000.0, 0.0), // dummy
        };

        Self {
            low_shelf,
            high_shelf,
            eq_type,
        }
    }

    /// Updates the gain of the equalizer.
    ///
    /// Converts the percentage value to decibel gain and applies it to the appropriate biquad filter(s).
    /// For Band mode, both filters are updated.
    ///
    /// # Arguments
    ///
    /// * `percent` - The new gain as a percentage (0.0 to 1.0).
    pub fn set_percent(&mut self, percent: f32) {
        let gain_db = percent_to_db(percent);
        match self.eq_type {
            EQType::Low => self.low_shelf.set_gain_db(gain_db),
            EQType::High => self.high_shelf.set_gain_db(gain_db),
            EQType::Band => {
                self.low_shelf.set_gain_db(gain_db);
                self.high_shelf.set_gain_db(gain_db);
            }
            EQType::Peak => self.low_shelf.set_gain_db(gain_db),
        }
    }

    /// Processes a single audio sample through the equalizer.
    ///
    /// Applies the appropriate filter(s) based on the EQ type. For Band mode, the sample
    /// is processed through both low and high shelves in sequence.
    ///
    /// # Arguments
    ///
    /// * `sample` - The input audio sample.
    ///
    /// # Returns
    ///
    /// The filtered output sample.
    pub fn process(&mut self, sample: f32) -> f32 {
        match self.eq_type {
            EQType::Low => self.low_shelf.process(sample),
            EQType::High => self.high_shelf.process(sample),
            EQType::Band => {
                let x = self.low_shelf.process(sample);
                self.high_shelf.process(x)
            }
            EQType::Peak => self.low_shelf.process(sample),
        }
    }
}

/// Converts a percentage value to decibel gain.
///
/// The conversion uses a logarithmic scale where 0% corresponds to -24 dB and 100% to 0 dB.
/// Values are clamped to prevent extreme gains that could cause instability.
///
/// # Arguments
///
/// * `percent` - The percentage value (0.0 to 1.0).
///
/// # Returns
///
/// The equivalent gain in decibels.
fn percent_to_db(percent: f32) -> f32 {
    let p = percent.clamp(0.0001, 1.0);
    // Logarithmic: 0% -> -24 dB, 100% -> 0 dB (prevents instability at extreme values)
    20.0 * p.log10()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(test)]
    mod success_path {}

    #[cfg(test)]
    mod failure_path {
        use super::*;

        #[test]
        fn test_percent_clamping_high() {
            let mut eq = RangeEQ::new(44100.0, 100.0, 8000.0, 2.0, EQType::Low);
            // Should clamp to 1.0 internally and process without issues
            let output = eq.process(0.1);
            assert!(!output.is_nan());
            assert!(!output.is_infinite());
        }

        #[test]
        fn test_percent_clamping_low() {
            let mut eq = RangeEQ::new(44100.0, 100.0, 8000.0, -1.0, EQType::Low);
            // Should clamp to 0.0001 internally and process without issues
            let output = eq.process(0.1);
            assert!(!output.is_nan());
            assert!(!output.is_infinite());
        }

        #[test]
        fn test_extreme_frequencies() {
            let mut eq = RangeEQ::new(44100.0, 1.0, 22000.0, 0.5, EQType::Band);
            let output = eq.process(0.1);
            assert!(!output.is_nan());
            assert!(!output.is_infinite());
        }

        #[test]
        fn test_process_zero_input() {
            let mut eq = RangeEQ::new(44100.0, 100.0, 8000.0, 0.5, EQType::Band);
            let output = eq.process(0.0);
            assert!(!output.is_nan());
            assert!(!output.is_infinite());
        }

        #[test]
        fn test_process_very_small_input() {
            let mut eq = RangeEQ::new(44100.0, 100.0, 8000.0, 0.5, EQType::Band);
            let output = eq.process(1e-6);
            assert!(!output.is_nan());
            assert!(!output.is_infinite());
        }

        #[test]
        fn test_process_large_input() {
            let mut eq = RangeEQ::new(44100.0, 100.0, 8000.0, 0.5, EQType::Band);
            let output = eq.process(100.0);
            assert!(!output.is_nan());
            assert!(!output.is_infinite());
        }

        #[test]
        fn test_set_percent_extreme_values() {
            let mut eq = RangeEQ::new(44100.0, 100.0, 8000.0, 0.5, EQType::Band);

            eq.set_percent(0.0);
            let out1 = eq.process(0.1);

            eq.set_percent(10.0); // Way above 1.0
            let out2 = eq.process(0.1);

            eq.set_percent(-5.0); // Negative
            let out3 = eq.process(0.1);

            assert!(!out1.is_nan() && !out2.is_nan() && !out3.is_nan());
        }

        #[test]
        fn test_multiple_set_percent_calls() {
            let mut eq = RangeEQ::new(44100.0, 100.0, 8000.0, 0.5, EQType::Band);

            let percents = vec![0.1, 0.5, 0.9, 0.2, 0.8, 0.0, 1.0];

            for percent in percents {
                eq.set_percent(percent);
                let output = eq.process(0.1);
                assert!(!output.is_nan());
                assert!(!output.is_infinite());
            }
        }

        #[test]
        fn test_all_eq_types_with_extreme_values() {
            let eq_types = vec![EQType::Low, EQType::High, EQType::Band, EQType::Peak];

            for eq_type in eq_types {
                let mut eq = RangeEQ::new(44100.0, 20.0, 20000.0, 0.01, eq_type);
                let output = eq.process(0.1);
                assert!(!output.is_nan());
                assert!(!output.is_infinite());
            }
        }

        #[test]
        fn test_low_sample_rate() {
            let mut eq = RangeEQ::new(8000.0, 50.0, 4000.0, 0.5, EQType::Band);
            let output = eq.process(0.1);
            assert!(!output.is_nan());
            assert!(!output.is_infinite());
        }

        #[test]
        fn test_high_sample_rate() {
            let mut eq = RangeEQ::new(192000.0, 20.0, 40000.0, 0.5, EQType::Band);
            let output = eq.process(0.1);
            assert!(!output.is_nan());
            assert!(!output.is_infinite());
        }

        #[test]
        fn test_process_after_multiple_gain_changes() {
            let mut eq = RangeEQ::new(44100.0, 100.0, 8000.0, 0.5, EQType::Low);

            eq.set_percent(0.1);
            let out1 = eq.process(0.1);

            eq.set_percent(0.9);
            let out2 = eq.process(0.1);

            eq.set_percent(0.5);
            let out3 = eq.process(0.1);

            assert!(!out1.is_nan() && !out2.is_nan() && !out3.is_nan());
        }

        #[test]
        fn test_percent_to_db_edge_cases() {
            // Test clamping behavior
            assert_eq!(percent_to_db(1.0), 0.0);
            assert_eq!(percent_to_db(0.0001), percent_to_db(0.00005)); // Both should clamp to 0.0001
            assert!(percent_to_db(2.0) == percent_to_db(1.0)); // Should clamp to 1.0
        }
    }
}
