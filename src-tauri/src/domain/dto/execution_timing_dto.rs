//! DTO for the CPU execution cost of a single DSP processor.
//!
//! Produced by [`AudioLatencyMeasurementService::measure_all_dsp_timings`] and
//! serialised across the Tauri IPC boundary to the developer-mode UI overlay.
//!
//! [`AudioLatencyMeasurementService::measure_all_dsp_timings`]: crate::services::audio_latency_measurement_service::AudioLatencyMeasurementService::measure_all_dsp_timings

use serde::{Deserialize, Serialize};

/// CPU execution cost contributed by one processor in the DSP signal chain.
///
/// The value is the *net* added cost — a zero-work passthrough baseline has already been
/// subtracted — so it represents only the work the processor itself performs, not
/// timer-call or loop overhead.  The result is clamped to `≥ 0` to avoid negative
/// readings from measurement noise.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(crate = "serde")]
pub struct ExecutionTimingDto {
    /// Name of the DSP processor (e.g. `"Gain"`, `"Tone Stack"`, `"Master Volume"`).
    pub processor_name: String,

    /// Net CPU cost of this processor in **microseconds per sample** (µs/sample).
    ///
    /// Multiply by the sample rate (Hz) to obtain the percentage of a 1-second
    /// CPU budget consumed by this processor alone.
    pub execution_us_per_sample: f64,
}

impl ExecutionTimingDto {
    /// Creates a new timing entry for the named processor.
    ///
    /// # Arguments
    ///
    /// * `processor_name` — Human-readable processor identifier.
    /// * `execution_us_per_sample` — Net execution cost in µs/sample (should be `≥ 0`).
    pub fn new(processor_name: impl Into<String>, execution_us_per_sample: f64) -> Self {
        Self {
            processor_name: processor_name.into(),
            execution_us_per_sample,
        }
    }
}
