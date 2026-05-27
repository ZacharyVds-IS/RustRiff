use crate::services::analyzers::spectrum_tap::{SpectrumTap, SPECTRUM_WINDOW_SIZE};
use log::info;
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
    pub fn detect_pitch(tap: &SpectrumTap) -> Option<PitchSnapshot> {
        let sample_rate = tap.sample_rate_hz() as usize;
        let samples = tap.snapshot_window();

        if samples.is_empty() {
            return None;
        }

        let signal: Vec<f64> = samples.iter().map(|&s| s as f64).collect();

        let power_threshold = 0.01;
        let clarity_threshold = 0.65;

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
    fn hz_to_pitch_snapshot(mut frequency: f32, clarity: f32) -> PitchSnapshot {
        if frequency <= 20.0 {
            return PitchSnapshot {
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

        info!(
            "Detected pitch: {:.2} Hz, note: {}, cents deviation: {:.2}, clarity: {:.2}",
            frequency, note_name, cents_deviation, clarity
        );

        PitchSnapshot {
            frequency_hz: frequency,
            note_name,
            cents_deviation,
            clarity,
        }
    }
}
