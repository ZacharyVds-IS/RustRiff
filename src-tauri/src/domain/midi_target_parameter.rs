use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum MidiTargetParameter {
    ToggleBypass,        // Available on ALL effects
    WahPedalPosition,    // Specific to WahDto
    DelayTime,           // Specific to DelayDto
    DelayLevel,          // Specific to DelayDto
    DistortionLevel,     // Specific to Hc/Sc Distortion
    DistortionThreshold, // Specific to Hc/Sc Distortion
}
