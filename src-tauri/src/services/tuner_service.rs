use crate::services::analyzers::spectrum_tap::SpectrumTap;
use log::info;
use pitch_detection::detector::mcleod::McLeodDetector;
use pitch_detection::detector::PitchDetector;
use std::cell::RefCell;

pub struct PitchSnapshot {
    /// Found fundamental frequency in Hz.
    pub frequency_hz: f32,
    /// Musical note name with octave (e.g., "E2", "A4").
    pub note_name: String,
    /// Deviation from perfect tuning in cents (-50.0 to +50.0).
    pub cents_deviation: f32,
    /// Algorithm confidence score between 0.0 and 1.0.
    pub clarity: f32,
}

pub struct TunerService;

thread_local! {
    static MCLEOD_DETECTOR: RefCell<McLeodDetector<f64>> =
        RefCell::new(McLeodDetector::new(2048, 1024));
}

impl TunerService {
    /// Processes the current time-domain ring buffer state into an active tuning note.
    pub fn detect_pitch(tap: &SpectrumTap) -> Option<PitchSnapshot> {
        let sample_rate = tap.sample_rate_hz() as usize;
        let samples = tap.snapshot_window();

        if samples.is_empty() {
            return None;
        }

        let signal: Vec<f64> = samples.iter().map(|&s| s as f64).collect();


        let power_threshold = 0.005;
        let clarity_threshold = 0.80;

        MCLEOD_DETECTOR.with(|cell| {
            let mut detector = cell.borrow_mut();

            let pitch = detector.get_pitch(
                &signal,
                sample_rate,
                power_threshold,
                clarity_threshold
            )?;

            Some(Self::hz_to_pitch_snapshot(pitch.frequency as f32, pitch.clarity as f32))
        })
    }

    /// Translates raw Hz values into standardized musical coordinates
    fn hz_to_pitch_snapshot(frequency: f32, clarity: f32) -> PitchSnapshot {
        if frequency <= 20.0 {
            return PitchSnapshot {
                frequency_hz: frequency,
                note_name: "---".to_string(),
                cents_deviation: 0.0,
                clarity,
            };
        }

        let n = 12.0 * (frequency / 440.0).log2();
        let midi_note = (n.round() + 69.0) as i32;

        let note_names = ["C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B"];

        let note_index = ((midi_note % 12 + 12) % 12) as usize;
        let octave = (midi_note / 12) - 1;
        let note_name = format!("{}{}", note_names[note_index], octave);

        let cents_deviation = (n - n.round()) * 100.0;

        info!("Detected pitch: {:.2} Hz, note: {}, cents deviation: {:.2}, clarity: {:.2}",
            frequency, note_name, cents_deviation, clarity);
        PitchSnapshot {
            frequency_hz: frequency,
            note_name,
            cents_deviation,
            clarity,
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::analyzers::spectrum_tap::SPECTRUM_WINDOW_SIZE;
    use std::f32::consts::PI;

    #[cfg(test)]
    mod success_path {
        use super::*;

        #[test]
        fn detect_pitch_identifies_perfect_a4_concert_pitch() {
            let sample_rate = 44_100;
            let tap = SpectrumTap::new(sample_rate);

            // Generate a pure 440 Hz (A4) sine wave at a healthy amplitude (0.5)
            let target_freq = 440.0_f32;
            for n in 0..SPECTRUM_WINDOW_SIZE {
                let sample = (2.0 * PI * target_freq * (n as f32 / sample_rate as f32)).sin() * 0.5;
                tap.push_sample(sample);
            }

            let result = TunerService::detect_pitch(&tap);

            assert!(result.is_some(), "Should cleanly detect a pitch for a clear 440Hz tone");
            let snapshot = result.unwrap();

            assert_eq!(snapshot.note_name, "A4");
            assert!(snapshot.frequency_hz > 439.0 && snapshot.frequency_hz < 441.0);
            assert!(snapshot.cents_deviation.abs() < 1.0, "Cents deviation should be near zero");
            assert!(snapshot.clarity > 0.9, "Clarity score should be extremely high for a pure tone");
        }

        #[test]
        fn detect_pitch_tracks_sharp_and_flat_deviations() {
            let sample_rate = 44_100;
            let tap = SpectrumTap::new(sample_rate);

            // Perfect E4 is 329.63 Hz. Let's feed it a slightly sharp 332.0 Hz signal instead.
            let sharp_freq = 332.0_f32;
            for n in 0..SPECTRUM_WINDOW_SIZE {
                let sample = (2.0 * PI * sharp_freq * (n as f32 / sample_rate as f32)).sin() * 0.5;
                tap.push_sample(sample);
            }

            let result = TunerService::detect_pitch(&tap);

            assert!(result.is_some());
            let snapshot = result.unwrap();

            assert_eq!(snapshot.note_name, "E4");
            assert!(snapshot.cents_deviation > 0.0, "Cents deviation must register as sharp (positive value)");
            assert!(snapshot.cents_deviation < 50.0, "Should not spill into the next semitone index");
        }
    }

    #[cfg(test)]
    mod failure_path {
        use super::*;

        #[test]
        fn detect_pitch_with_silent_input_returns_none() {
            // A freshly initialized tap contains completely zeroed-out memory
            let tap = SpectrumTap::new(48_000);

            let result = TunerService::detect_pitch(&tap);

            assert!(result.is_none(), "Silence should gracefully fail the power threshold check");
        }

        #[test]
        fn detect_pitch_with_uncorrelated_noise_returns_none() {
            let tap = SpectrumTap::new(48_000);

            // Fill the circular buffer with non-periodic pseudo-random white noise
            let mut seed = 123_456_789_u32;
            for _ in 0..SPECTRUM_WINDOW_SIZE {
                seed = seed.wrapping_mul(1_103_515_245).wrapping_add(12_345);
                let noise_sample = ((seed % 2000) as f32 / 1000.0) - 1.0;
                tap.push_sample(noise_sample * 0.2);
            }

            let result = TunerService::detect_pitch(&tap);

            assert!(result.is_none(), "Pure white noise must be rejected by the clarity/periodicity threshold");
        }
    }
}


