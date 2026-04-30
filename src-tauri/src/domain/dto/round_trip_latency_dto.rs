//! DTO for the result of a hardware round-trip latency measurement.
//!
//! Produced by [`AudioLatencyMeasurementService::measure_round_trip_latency`] and
//! serialised across the Tauri IPC boundary so the frontend can display the result
//! or an actionable error message.
//!
//! [`AudioLatencyMeasurementService::measure_round_trip_latency`]: crate::services::audio_latency_measurement_service::AudioLatencyMeasurementService::measure_round_trip_latency

use serde::{Deserialize, Serialize};

/// Result of a hardware round-trip latency measurement.
///
/// Round-trip latency is the total wall-clock time from when a sample is written to
/// the output ring buffer to when it returns on the input ring buffer after passing
/// through the DAC, the physical audio path, and the ADC.  It captures:
///
/// - Input and output buffer delays
/// - Hardware AD/DA conversion time
/// - OS scheduling and driver latency
/// - Any resampler buffering in the signal path
///
/// The measurement is performed by [`RoundTripLatencySession`] on a dedicated thread
/// using private CPAL streams, completely separate from the main audio loopback.
///
/// # Validity
///
/// Always check [`is_valid`] before using [`latency_ms`].  When `is_valid` is `false`
/// the `latency_ms` field is `0.0` and [`error`] contains a human-readable description
/// of what went wrong (e.g. no echo detected, timeout).
///
/// [`RoundTripLatencySession`]: crate::services::round_trip_latency_session::RoundTripLatencySession
/// [`is_valid`]: RoundTripLatencyDto::is_valid
/// [`latency_ms`]: RoundTripLatencyDto::latency_ms
/// [`error`]: RoundTripLatencyDto::error
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(crate = "serde")]
pub struct RoundTripLatencyDto {
    /// Average round-trip latency in milliseconds across all impulse/echo cycles.
    ///
    /// Only meaningful when [`is_valid`] is `true`.  Set to `0.0` on failure.
    ///
    /// [`is_valid`]: RoundTripLatencyDto::is_valid
    pub latency_ms: f64,

    /// Whether the measurement completed successfully.
    ///
    /// `true` when at least [`IMPULSE_COUNT`] echoes were detected within the timeout.
    /// `false` when the measurement timed out or the echo signal was undetectable.
    ///
    /// [`IMPULSE_COUNT`]: crate::services::round_trip_latency_session::IMPULSE_AMPLITUDE
    pub is_valid: bool,

    /// Human-readable failure reason, or `None` on success.
    ///
    /// Typical messages:
    /// - `"Echo not detected above threshold …"` — output not routed to input.
    /// - `"Round-trip measurement timed out …"` — overall deadline exceeded.
    /// - `"Round-trip measurement thread panicked"` — unexpected internal error.
    pub error: Option<String>,
}

impl RoundTripLatencyDto {
    /// Creates a successful result with the given averaged latency.
    ///
    /// Sets `is_valid = true` and `error = None`.
    pub fn success(latency_ms: f64) -> Self {
        Self {
            latency_ms,
            is_valid: true,
            error: None,
        }
    }

    /// Creates a failed result with the given error message.
    ///
    /// Sets `is_valid = false` and `latency_ms = 0.0`.
    pub fn failure(error: String) -> Self {
        Self {
            latency_ms: 0.0,
            is_valid: false,
            error: Some(error),
        }
    }
}
