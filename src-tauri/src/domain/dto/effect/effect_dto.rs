use crate::domain::dto::effect::hcdistortion_dto::HcDistortionDto;
use crate::domain::effect::Effect;
use crate::services::effects::distortion::hc_distortion::HCDistortion;
use serde::{Deserialize, Serialize};

/// A serialisable, tagged representation of any effect in the signal chain.
///
/// Uses serde's adjacently-tagged format so that both the Rust serialisation
/// and the TypeScript typegen agree on the wire shape:
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "data")]
pub enum EffectDto {
    /// Hard-clipping distortion effect.
    HCDistortion(HcDistortionDto),
}

impl EffectDto {
    pub fn add_to_domain(self, next_effect_id: u32) -> Box<dyn Effect> {
        match self { 
            EffectDto::HCDistortion(dto) => Box::new(HCDistortion::new(
                next_effect_id,
                dto.name,
                dto.is_active,
                dto.threshold,
                dto.level,
                dto.color,
            )),
        }
    }

    pub fn to_domain(self) -> Box<dyn Effect> {
        match self {
            EffectDto::HCDistortion(dto) => Box::new(HCDistortion::new(
                dto.id,
                dto.name,
                dto.is_active,
                dto.threshold,
                dto.level,
                dto.color,
            )),
        }
    }
}