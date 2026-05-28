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

#[cfg(test)]
mod tests {
    use super::*;

    mod param_name {
        use super::*;

        #[test]
        fn toggle_bypass_returns_none() {
            assert_eq!(MidiTargetParameter::ToggleBypass.param_name(), None);
        }

        #[test]
        fn wah_pedal_position_returns_pedal_position() {
            assert_eq!(
                MidiTargetParameter::WahPedalPosition.param_name(),
                Some("pedal_position")
            );
        }

        #[test]
        fn delay_time_returns_none() {
            assert_eq!(MidiTargetParameter::DelayTime.param_name(), None);
        }

        #[test]
        fn delay_level_returns_level() {
            assert_eq!(MidiTargetParameter::DelayLevel.param_name(), Some("level"));
        }

        #[test]
        fn distortion_level_returns_level() {
            assert_eq!(
                MidiTargetParameter::DistortionLevel.param_name(),
                Some("level")
            );
        }

        #[test]
        fn distortion_threshold_returns_threshold() {
            assert_eq!(
                MidiTargetParameter::DistortionThreshold.param_name(),
                Some("threshold")
            );
        }
    }

    mod is_toggle {
        use super::*;

        #[test]
        fn toggle_bypass_is_toggle() {
            assert!(MidiTargetParameter::ToggleBypass.is_toggle());
        }

        #[test]
        fn wah_pedal_position_is_not_toggle() {
            assert!(!MidiTargetParameter::WahPedalPosition.is_toggle());
        }

        #[test]
        fn delay_time_is_not_toggle() {
            assert!(!MidiTargetParameter::DelayTime.is_toggle());
        }

        #[test]
        fn delay_level_is_not_toggle() {
            assert!(!MidiTargetParameter::DelayLevel.is_toggle());
        }

        #[test]
        fn distortion_level_is_not_toggle() {
            assert!(!MidiTargetParameter::DistortionLevel.is_toggle());
        }

        #[test]
        fn distortion_threshold_is_not_toggle() {
            assert!(!MidiTargetParameter::DistortionThreshold.is_toggle());
        }
    }

    mod serialization {
        use super::*;

        #[test]
        fn round_trips_through_json() {
            let variants = [
                MidiTargetParameter::ToggleBypass,
                MidiTargetParameter::WahPedalPosition,
                MidiTargetParameter::DelayTime,
                MidiTargetParameter::DelayLevel,
                MidiTargetParameter::DistortionLevel,
                MidiTargetParameter::DistortionThreshold,
            ];
            for variant in &variants {
                let json = serde_json::to_string(variant).unwrap();
                let deserialized: MidiTargetParameter = serde_json::from_str(&json).unwrap();
                assert_eq!(*variant, deserialized);
            }
        }
    }
}
