//! Round-trip latency measurement using dedicated CPAL streams.
//!
//! # Overview
//!
//! Round-trip latency is the total time a sound takes to travel from the software output,
//! through the DAC, over the physical audio cable (or internal loopback), back in through
//! the ADC, and arrive at the input ring buffer where the software can read it.
//!
//! This module measures that latency by:
//!
//! 1. **Opening dedicated streams** — a private input/output pair, completely separate from
//!    the main loopback in [`AudioService`].  The main engine is unaffected and can stay
//!    running during the measurement.
//! 2. **Warming up** — sleeping for `stream_warmup` after stream creation so the OS audio
//!    stack and hardware buffers reach a stable steady state before any timing begins.
//! 3. **Calibrating the noise floor** — collecting [`CALIBRATION_SAMPLES`] silent ambient
//!    samples from the input to measure the peak background noise.  The detection threshold
//!    is derived from that peak so a genuine echo can be distinguished from ambient noise.
//! 4. **Emitting impulses** — writing a single full-amplitude sample ([`IMPULSE_AMPLITUDE`])
//!    to the output ring buffer while recording the exact wall-clock time.
//! 5. **Detecting the echo** — reading incoming input samples and waiting for the first
//!    sample that crosses the derived threshold.  A guard window of [`GUARD_SAMPLES`] is
//!    skipped immediately after the impulse to avoid detecting the outgoing signal itself
//!    or very early electrical bleed-through.
//! 6. **Averaging** — repeating the impulse/echo cycle [`IMPULSE_COUNT`] times and
//!    returning the mean of all measured round-trip durations to smooth out callback jitter.
//!
//! # Physical setup requirement
//!
//! The output of the audio interface must be physically (or virtually) connected back to its
//! input for the echo to be heard.  On a typical guitar-amp setup this is the instrument
//! cable path: software → DAC → speaker/DI → microphone/DI → ADC → software.  On a
//! development machine a virtual loopback driver (e.g. VB-Audio Cable) can be used instead.
//!
//! # Isolation from the main audio engine
//!
//! [`RoundTripLatencySession::run`] is a **blocking, self-contained call**.  It opens its
//! own CPAL streams via the same [`AudioHandlerTrait`] the main engine uses, but does **not**
//! share ring buffers, threads, or any state with [`AudioService`].  The caller
//! (`measure_round_trip_latency` command) releases the `Mutex<AudioService>` lock before
//! spawning the measurement thread, so the UI and all other commands remain fully responsive
//! during the several-second measurement window.
//!
//! # Module layout
//!
//! | File | Contents |
//! |---|---|
//! | `constants.rs` | All tuning constants (`CALIBRATION_SAMPLES`, `IMPULSE_COUNT`, etc.) |
//! | `measurement_state.rs` | `RoundTripTickOutcome`, `RoundTripMeasurementPhase`, `RoundTripMeasurementState` + unit tests |
//! | `session.rs` | `RoundTripLatencySession::run` — stream lifecycle and measurement loop |
//!
//! [`AudioService`]: crate::services::audio_service::AudioService
//! [`AudioHandlerTrait`]: crate::infrastructure::audio_handler::AudioHandlerTrait
//! [`CALIBRATION_SAMPLES`]: constants::CALIBRATION_SAMPLES
//! [`IMPULSE_AMPLITUDE`]: constants::IMPULSE_AMPLITUDE
//! [`GUARD_SAMPLES`]: constants::GUARD_SAMPLES
//! [`IMPULSE_COUNT`]: constants::IMPULSE_COUNT

pub mod constants;
pub mod measurement_state;
pub mod session;

// Re-export the public surface so callers can use the old import paths unchanged.
pub use constants::IMPULSE_AMPLITUDE;
pub use measurement_state::{
    RoundTripMeasurementPhase, RoundTripMeasurementState, RoundTripTickOutcome,
};
pub use session::RoundTripLatencySession;

