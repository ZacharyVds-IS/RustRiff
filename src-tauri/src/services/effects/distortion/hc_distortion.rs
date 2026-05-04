use crate::domain::audio_processor::AudioProcessor;
use crate::domain::dto::effect::effect_dto::EffectDto;
use crate::domain::dto::effect::hcdistortion_dto::HcDistortionDto;
use crate::domain::effect::Effect;

/// Hard-clipping distortion effect.
///
/// Any sample whose absolute value exceeds `threshold` is clamped to `±threshold`,
/// producing the flat-top waveform characteristic of hard clipping.
///
/// # Parameters
///
/// * `threshold` — Clip level in the range `(0.0, 1.0]`.  A value of `1.0` means
///   no clipping occurs until the signal hits full scale.  Lower values produce
///   heavier distortion.
pub struct HCDistortion {
    id: u32,
    name: String,
    is_active: bool,
    limit: f32,
    color: String,
}

impl HCDistortion {
    /// Creates a new `HCDistortion` effect.
    ///
    /// # Arguments
    ///
    /// * `id` — Unique effect identifier.
    /// * `name` — Display name.
    /// * `is_active` — Whether the effect is enabled on creation.
    /// * `threshold` — Clip level; clamped internally to the range `[0.001, 1.0]`
    ///   so that a threshold of exactly zero cannot produce silence for all input.
    pub fn new(id: u32, name: String, is_active: bool, threshold: f32, color:String) -> Self {
        Self {
            id,
            name,
            is_active,
            limit: threshold.clamp(0.001, 1.0),
            color
        }
    }

    pub fn threshold(&self) -> f32 {
        self.limit
    }

    pub fn set_threshold(&mut self, threshold: f32) {
        self.limit = threshold.clamp(0.001, 1.0);
    }
}

impl AudioProcessor for HCDistortion {
    fn process(&mut self, sample: f32) -> f32 {
        sample.clamp(-self.limit, self.limit)
    }
}

impl Effect for HCDistortion {
    fn id(&self) -> u32 {
        self.id
    }

    fn name(&self) -> &str {
        self.name.as_str()
    }

    fn is_active(&self) -> bool {
        self.is_active
    }

    fn set_active(&mut self, active: bool) {
        self.is_active = active;
    }

    fn get_color(&self) -> String {
        "#e67e22".to_string()
    }

    fn to_dto(&self) -> EffectDto {
        EffectDto::HCDistortion(HcDistortionDto {
            id: self.id,
            name: self.name.clone(),
            is_active: self.is_active,
            color: self.color.clone(),
            threshold: self.limit,
        })
    }

    fn process_if_active(&mut self, sample: f32) -> f32 {
        if self.is_active {
            self.process(sample)
        } else {
            sample
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn distortion(threshold: f32) -> HCDistortion {
        HCDistortion::new(0, "HC".to_string(), true, threshold, "#00FF00".to_string())
    }

    #[cfg(test)]
    mod success_path {
        use super::*;

        #[test]
        fn sample_within_threshold_is_unchanged() {
            let mut fx = distortion(0.5);
            assert_eq!(fx.process(0.3), 0.3);
            assert_eq!(fx.process(-0.3), -0.3);
        }

        #[test]
        fn sample_above_threshold_is_clipped_to_threshold() {
            let mut fx = distortion(0.5);
            assert_eq!(fx.process(0.9), 0.5);
        }

        #[test]
        fn sample_below_negative_threshold_is_clipped_to_negative_threshold() {
            let mut fx = distortion(0.5);
            assert_eq!(fx.process(-0.9), -0.5);
        }

        #[test]
        fn sample_exactly_at_threshold_is_unchanged() {
            let mut fx = distortion(0.5);
            assert_eq!(fx.process(0.5), 0.5);
            assert_eq!(fx.process(-0.5), -0.5);
        }

        #[test]
        fn process_if_active_clips_when_active() {
            let mut fx = distortion(0.5);
            assert_eq!(fx.process_if_active(0.9), 0.5);
        }

        #[test]
        fn process_if_active_passes_through_when_inactive() {
            let mut fx = distortion(0.5);
            fx.set_active(false);
            assert_eq!(fx.process_if_active(0.9), 0.9);
        }

        #[test]
        fn set_threshold_updates_clip_level() {
            let mut fx = distortion(0.8);
            fx.set_threshold(0.3);
            assert_eq!(fx.threshold(), 0.3);
            assert_eq!(fx.process(0.9), 0.3);
        }
    }

    #[cfg(test)]
    mod failure_path {
        use super::*;

        #[test]
        fn threshold_above_one_is_clamped_to_one() {
            let fx = distortion(2.0);
            assert_eq!(fx.threshold(), 1.0);
        }

        #[test]
        fn threshold_of_zero_is_clamped_to_minimum() {
            let fx = distortion(0.0);
            assert!(fx.threshold() > 0.0);
        }

        #[test]
        fn negative_threshold_is_clamped_to_minimum() {
            let fx = distortion(-1.0);
            assert!(fx.threshold() > 0.0);
        }
    }
}