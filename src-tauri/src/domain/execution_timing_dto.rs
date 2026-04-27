use serde::{Deserialize, Serialize};

/// Represents the execution-time impact of a single audio processor in the DSP chain.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(crate = "serde")]
pub struct ExecutionTimingDto {
    /// Name of the audio processor (e.g., "Gain", "Tone Stack", "Master Volume").
    pub processor_name: String,

    /// Execution cost in microseconds per sample.
    pub execution_us_per_sample: f64,
}

impl ExecutionTimingDto {
    pub fn new(processor_name: impl Into<String>, execution_us_per_sample: f64) -> Self {
        Self {
            processor_name: processor_name.into(),
            execution_us_per_sample,
        }
    }
}

