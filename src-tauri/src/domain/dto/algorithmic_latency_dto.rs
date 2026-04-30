//! DTO for the algorithmic (design-inherent) latency of a single DSP processor.
//!
//! Produced by [`AudioLatencyMeasurementService::measure_all_dsp_algorithmic_latency`]
//! and serialised across the Tauri IPC boundary to the developer-mode UI overlay.
//!
//! [`AudioLatencyMeasurementService::measure_all_dsp_algorithmic_latency`]: crate::services::audio_latency_measurement_service::AudioLatencyMeasurementService::measure_all_dsp_algorithmic_latency

use serde::{Deserialize, Serialize};

/// Algorithmic latency contributed by one processor in the DSP signal chain.
///
/// Algorithmic latency is the *design-inherent* sample delay an effect introduces —
/// for example, a look-ahead limiter that must buffer `N` input samples before it can
/// emit the first output sample adds `N` samples of algorithmic latency, independent
/// of CPU speed.
///
/// For sample-by-sample processors such as `GainProcessor` and `ToneStackProcessor`
/// this value is always **zero** because no sample is ever buffered or delayed.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(crate = "serde")]
pub struct AlgorithmicLatencyDto {
    /// Name of the DSP processor (e.g. `"Gain"`, `"Tone Stack"`, `"Master Volume"`).
    pub processor_name: String,

    /// Number of audio samples of algorithmic delay introduced by this processor.
    ///
    /// Zero for all current processors.
    pub latency_samples: u32,

    /// Algorithmic delay converted to milliseconds at the current output sample rate.
    ///
    /// Computed as `(latency_samples / sample_rate_hz) × 1000`.  Zero when
    /// `latency_samples` is zero or `sample_rate_hz` is zero.
    pub latency_ms: f64,
}

impl AlgorithmicLatencyDto {
    /// Creates a new entry for the named processor, converting samples to ms automatically.
    ///
    /// # Arguments
    ///
    /// * `processor_name` — Human-readable processor identifier.
    /// * `latency_samples` — Number of samples of algorithmic delay.
    /// * `sample_rate_hz` — Output sample rate used for the ms conversion.
    ///   If `0`, `latency_ms` is set to `0.0` to avoid a division-by-zero.
    pub fn new(processor_name: impl Into<String>, latency_samples: u32, sample_rate_hz: u32) -> Self {
        let latency_ms = if sample_rate_hz == 0 {
            0.0
        } else {
            (latency_samples as f64 / sample_rate_hz as f64) * 1000.0
        };

        Self {
            processor_name: processor_name.into(),
            latency_samples,
            latency_ms,
        }
    }
}
