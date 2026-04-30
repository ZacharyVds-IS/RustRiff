//! Tuning constants for the round-trip latency measurement protocol.
//!
//! All values here are deliberately centralised so that changing the measurement
//! behaviour requires touching exactly one file.

use std::time::Duration;

/// Number of input samples collected during the ambient calibration phase.
///
/// At 44 100 Hz this is roughly 11 ms of listening time — long enough to capture
/// a representative noise-floor peak without delaying the measurement significantly.
pub const CALIBRATION_SAMPLES: usize = 512;

/// Number of impulse/echo cycles to run per measurement session.
///
/// The final reported latency is the arithmetic mean of all [`IMPULSE_COUNT`] individual
/// round-trip measurements, which reduces the impact of single-callback scheduling jitter.
pub const IMPULSE_COUNT: usize = 3;

/// Number of input samples to ignore immediately after emitting an impulse.
///
/// Electrical bleed-through and the outgoing impulse itself can appear on the input within
/// microseconds of being written.  Skipping these samples prevents a false-positive detection
/// before the signal has had time to traverse the physical audio path.
///
/// At 44 100 Hz this guard window is approximately 11 ms.
pub const GUARD_SAMPLES: usize = 512;

/// Minimum quiet time enforced between consecutive impulses.
///
/// After an echo is detected the previous impulse's reverb tail may still be decaying.
/// Waiting [`INTER_IMPULSE_GAP`] before the next emission prevents that tail from being
/// mistaken for the next echo.
pub const INTER_IMPULSE_GAP: Duration = Duration::from_millis(200);

/// Peak amplitude of the synthetic test impulse written to the output ring buffer.
///
/// A near-full-scale value is used so the echo stands well above the noise floor even after
/// passing through lossy physical paths.  The detection threshold is clamped to at most
/// `IMPULSE_AMPLITUDE * 0.5` so that a valid echo is always detectable.
pub const IMPULSE_AMPLITUDE: f32 = 0.95;

