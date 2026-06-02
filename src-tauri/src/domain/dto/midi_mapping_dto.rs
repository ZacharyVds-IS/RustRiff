use crate::domain::midi_target_parameter::MidiTargetParameter;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MidiMappingDto {
    pub cc_number: u8,
    pub channel: u8,       // Usually channel 1 (byte value 0 or 176 status)
    pub effect_id: String, // The specific instance in your signal chain
    pub parameter: MidiTargetParameter,
}
