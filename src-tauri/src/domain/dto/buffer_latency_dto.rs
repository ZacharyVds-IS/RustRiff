//! DTO for the I/O buffer latency of the current CPAL stream configuration.
//!
//! Produced by [`AudioLatencyMeasurementService::measure_buffer_latency`] and
//! serialised across the Tauri IPC boundary to the developer-mode UI overlay.
//!
//! [`AudioLatencyMeasurementService::measure_buffer_latency`]: crate::services::audio_latency_measurement_service::AudioLatencyMeasurementService::measure_buffer_latency

use serde::{Deserialize, Serialize};

/// I/O buffer latency for the current audio configuration.
///
/// Buffer latency is the delay imposed by the hardware frame buffers: the driver
/// accumulates a full block of samples before delivering them to (or accepting them
/// from) the application.  The formula is:
///
/// ```text
/// latency_ms = (buffer_frames / sample_rate_hz) × 1000
/// ```
///
/// When [`cpal::BufferSize::Default`] is in use the actual frame count is unknown at
/// runtime; a conservative fallback of **256 frames** is substituted so the UI can
/// always display a practical estimate.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(crate = "serde")]
pub struct BufferLatencyDto {
    /// Input-side buffering delay in milliseconds.
    ///
    /// Time the ADC driver holds samples in its internal buffer before delivering
    /// them to the application ring buffer.
    pub input_buffer_latency_ms: f64,

    /// Output-side buffering delay in milliseconds.
    ///
    /// Time the application must fill the DAC driver buffer before the first sample
    /// is actually played.
    pub output_buffer_latency_ms: f64,

    /// Total I/O buffer delay: `input_buffer_latency_ms + output_buffer_latency_ms`.
    ///
    /// This is the minimum hardware-buffer component of the full round-trip latency.
    pub total_buffer_latency_ms: f64,
}

impl BufferLatencyDto {
    /// Creates a new `BufferLatencyDto`, computing `total_buffer_latency_ms` automatically.
    ///
    /// # Arguments
    ///
    /// * `input_buffer_latency_ms` — Input buffering delay in ms.
    /// * `output_buffer_latency_ms` — Output buffering delay in ms.
    pub fn new(input_buffer_latency_ms: f64, output_buffer_latency_ms: f64) -> Self {
        Self {
            input_buffer_latency_ms,
            output_buffer_latency_ms,
            total_buffer_latency_ms: input_buffer_latency_ms + output_buffer_latency_ms,
        }
    }
}
