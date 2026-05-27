use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TunerContractDto {
    pub live_tuner_event: String,
}