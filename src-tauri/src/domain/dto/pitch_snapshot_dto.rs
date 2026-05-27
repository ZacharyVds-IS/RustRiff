use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
/// A point-in-time state resolution of a pitch detection calculation.
///
/// Contains both raw physical coordinates (Hz) and formatted musical
/// metadata suitable for rendering directly to user interface tuner widgets.
pub struct PitchSnapshotDto {
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