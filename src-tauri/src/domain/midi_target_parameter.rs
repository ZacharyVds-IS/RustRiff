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

impl MidiTargetParameter {
    /// Returns the string key used in [`Effect::f32_params`] / [`Channel::set_effect_param`].
    pub fn param_name(&self) -> Option<&'static str> {
        match self {
            MidiTargetParameter::ToggleBypass => None,
            MidiTargetParameter::WahPedalPosition => Some("pedal_position"),
            MidiTargetParameter::DelayTime => None, // u32 param, handled separately
            MidiTargetParameter::DelayLevel => Some("level"),
            MidiTargetParameter::DistortionLevel => Some("level"),
            MidiTargetParameter::DistortionThreshold => Some("threshold"),
        }
    }

    /// Whether this parameter is a discrete on/off toggle.
    ///
    /// Toggle parameters ignore the incoming MIDI value and just flip the current
    /// state (or cycle).  Continuous parameters always use the normalized CC value.
    pub fn is_toggle(&self) -> bool {
        matches!(self, MidiTargetParameter::ToggleBypass)
    }
}
