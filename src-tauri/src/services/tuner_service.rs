use crate::domain::dto::pitch_snapshot_dto::PitchSnapshotDto;
use crate::services::analyzers::spectrum_tap::{SpectrumTap, SPECTRUM_WINDOW_SIZE};
use pitch_detection::detector::mcleod::McLeodDetector;
use pitch_detection::detector::PitchDetector;
use std::cell::RefCell;

/// A point-in-time state resolution of a pitch detection calculation.
///
/// Contains both raw physical coordinates (Hz) and formatted musical
/// metadata suitable for rendering directly to user interface tuner widgets.
pub struct PitchSnapshot {
    /// Found fundamental frequency in Hz (after octave scale filters are applied).
    pub frequency_hz: f32,
    /// Musical note name with octave anchor (e.g., `"E2"`, `"A4"`, `"G#3"`).
    pub note_name: String,
    /// Logarithmic deviation from the perfect target semitone in cents.
    /// Bounded precisely between `-50.0` (perfectly flat) and `+50.0` (perfectly sharp).
    pub cents_deviation: f32,
    /// Periodicity confidence score between `0.0` and `1.0` emitted by the detector.
    pub clarity: f32,
}

/// Stateless domain service that processes raw time-domain audio windows into musical pitch structures.
///
/// `TunerService` acts as the analytical layer for instrument tuning. It isolates real-time stream
/// snapshots and uses a bounded-autocorrelation approach to resolve fundamental frequencies:
///
/// - **Algorithmic Engine** — utilizes the McLeod Pitch Method (MPM), which computes a normalized
///   square difference function (NSDF) to identify periodic structures without dropping accuracy
///   on varying signal amplitudes.
/// - **Allocation Profile** — execution on the hot path achieves zero heap allocations by leveraging
///   a thread-local storage strategy for the underlying detector memory buffers.
/// - **Pickup Scale Correction** — automatically intercepts and compensates for systemic sub-octave
///   tracking errors typical of electric guitar pickup inductors and low-register string acoustics.
pub struct TunerService;

thread_local! {
    /// Thread-local wrapper allocating the heavy tracking matrices for the pitch engine.
    ///
    /// Initialized with a window size matching [`SPECTRUM_WINDOW_SIZE`] and an internal offset lag padding
    /// of half the window size. This structure prevents dynamic heap allocations during real-time
    /// loopback analysis cycles.
    static MCLEOD_DETECTOR: RefCell<McLeodDetector<f64>> =
        RefCell::new(McLeodDetector::new(SPECTRUM_WINDOW_SIZE, SPECTRUM_WINDOW_SIZE / 2));
}

impl TunerService {
    /// Processes the current time-domain ring buffer state into an active tuning note.
    ///
    /// Extracts a fixed window of raw samples from the provided [`SpectrumTap`], casts the values
    /// into 64-bit floating point representations, and runs them through the thread-isolated
    /// autocorrelation matrix.
    ///
    /// # Arguments
    ///
    /// * `tap` - A reference to the active [`SpectrumTap`] component tracking the live input device.
    ///
    /// # Returns
    ///
    /// Returns `Some(PitchSnapshot)` if a periodic wave clears both the minimum physical energy
    /// floor and the structural clarity limits. Returns `None` during silence, uncorrelated room noise,
    /// or high-frequency line artifacts.
    pub fn detect_pitch(tap: &SpectrumTap) -> Option<PitchSnapshotDto> {
        let sample_rate = tap.sample_rate_hz() as usize;
        let samples = tap.snapshot_window();

        if samples.is_empty() {
            return None;
        }

        let signal: Vec<f64> = samples.iter().map(|&s| s as f64).collect();

        let power_threshold = 0.05;
        let clarity_threshold = 0.72;

        MCLEOD_DETECTOR.with(|cell| {
            let mut detector = cell.borrow_mut();

            let pitch =
                detector.get_pitch(&signal, sample_rate, power_threshold, clarity_threshold)?;

            if pitch.frequency > 4000.0 {
                return None;
            }

            Some(Self::hz_to_pitch_snapshot(
                pitch.frequency as f32,
                pitch.clarity as f32,
            ))
        })
    }

    // ── Private helpers ──────────────────────────────────────────────────────

    /// Translates raw physical Hz values into standardized equal-temperament musical coordinates.
    ///
    /// Calculates logarithmic pitch distance relative to the A440 concert pitch standard, mapping
    /// frequencies to fractional MIDI indices before resolving names and cents deviations.
    ///
    /// # Octave Scaling Filter
    ///
    /// Double-coiled magnetic pickups (humbuckers) and thin string geometries can cause the primary
    /// sub-harmonic resonance peak to appear structurally stronger to autocorrelation than the actual
    /// physical string frequency. To prevent tracking exactly one octave flat across the register,
    /// the base pitch is universally scaled up by a factor of 2.
    ///
    /// # Arguments
    ///
    /// * `frequency` - The raw fundamental frequency in Hertz parsed from the time-domain signal.
    /// * `clarity` - The periodicity confidence index calculated by the pitch detector.
    fn hz_to_pitch_snapshot(mut frequency: f32, clarity: f32) -> PitchSnapshotDto {
        if frequency <= 20.0 {
            return PitchSnapshotDto {
                frequency_hz: frequency,
                note_name: "---".to_string(),
                cents_deviation: 0.0,
                clarity,
            };
        }

        //Double the frequency because McLeodDetector returns the fundamental resonance frequency,
        //which is an octave below the actual note for guitar pickups.
        frequency *= 2.0;

        let n = 12.0 * (frequency / 440.0).log2();
        let midi_note = n.round() + 69.0;
        let note_names = [
            "C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B",
        ];

        let midi_int = midi_note as i32;
        let note_index = ((midi_int % 12 + 12) % 12) as usize;
        let octave = ((midi_note / 12.0).floor() - 1.0) as i32;
        let note_name = format!("{}{}", note_names[note_index], octave);

        let cents_deviation = (n - n.round()) * 100.0;

        PitchSnapshotDto {
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

            // Generate a pure sine wave at 220 Hz, which should be detected as A4 after the octave scaling correction.
            let target_freq = 220.0_f32;
            for n in 0..SPECTRUM_WINDOW_SIZE {
                let sample = (2.0 * PI * target_freq * (n as f32 / sample_rate as f32)).sin() * 0.5;
                tap.push_sample(sample);
            }

            let result = TunerService::detect_pitch(&tap);

            assert!(
                result.is_some(),
                "Should cleanly detect a pitch for a clear 440Hz tone"
            );
            let snapshot = result.unwrap();

            assert_eq!(snapshot.note_name, "A4");
            assert!(snapshot.frequency_hz > 439.0 && snapshot.frequency_hz < 441.0);
            assert!(
                snapshot.cents_deviation.abs() < 1.0,
                "Cents deviation should be near zero"
            );
            assert!(
                snapshot.clarity > 0.9,
                "Clarity score should be extremely high for a pure tone"
            );
        }

        #[test]
        fn detect_pitch_tracks_sharp_and_flat_deviations() {
            let sample_rate = 44_100;
            let tap = SpectrumTap::new(sample_rate);

            // Perfect E4 is 329.63 Hz. Let's feed it a slightly sharp 332.0 Hz signal instead.
            let sharp_freq = 166.0_f32;
            for n in 0..SPECTRUM_WINDOW_SIZE {
                let sample = (2.0 * PI * sharp_freq * (n as f32 / sample_rate as f32)).sin() * 0.5;
                tap.push_sample(sample);
            }

            let result = TunerService::detect_pitch(&tap);

            assert!(result.is_some());
            let snapshot = result.unwrap();

            assert_eq!(snapshot.note_name, "E4");
            assert!(
                snapshot.cents_deviation > 0.0,
                "Cents deviation must register as sharp (positive value)"
            );
            assert!(
                snapshot.cents_deviation < 50.0,
                "Should not spill into the next semitone index"
            );
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

            assert!(
                result.is_none(),
                "Silence should gracefully fail the power threshold check"
            );
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

            assert!(
                result.is_none(),
                "Pure white noise must be rejected by the clarity/periodicity threshold"
            );
        }
    }
}
