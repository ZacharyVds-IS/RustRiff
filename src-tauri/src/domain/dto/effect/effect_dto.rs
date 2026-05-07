use crate::domain::dto::effect::cabinet_dto::CabinetDto;
use crate::domain::dto::effect::delay_dto::DelayDto;
use crate::domain::dto::effect::hcdistortion_dto::HcDistortionDto;
use crate::domain::effect::Effect;
use crate::services::effects::cabinet::cabinet::Cabinet;
use crate::services::effects::delay::delay::Delay;
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
    Delay(DelayDto),
    /// Placeholder impulse-response cabinet effect.
    Cabinet(CabinetDto),
}

impl EffectDto {
    pub fn add_to_domain(self, next_effect_id: u32, dsp_sample_rate: u32) -> Box<dyn Effect> {
        match self {
            EffectDto::HCDistortion(dto) => Box::new(HCDistortion::new(
                next_effect_id,
                dto.name,
                dto.is_active,
                dto.threshold,
                dto.level,
                dto.color,
            )),
            EffectDto::Cabinet(dto) => Box::new(Cabinet::new(
                next_effect_id,
                dto.name,
                dto.is_active,
                dto.color,
                dto.ir_file_path,
                dsp_sample_rate,
            )),
            EffectDto::Delay(dto) => Box::new(Delay::new(
                next_effect_id,
                dto.name,
                dto.is_active,
                dto.color,
                dsp_sample_rate,
                dto.delay_time,
                dto.level
            ))
        }
    }

    pub fn to_domain(self, dsp_sample_rate: u32) -> Box<dyn Effect> {
        match self {
            EffectDto::HCDistortion(dto) => Box::new(HCDistortion::new(
                dto.id,
                dto.name,
                dto.is_active,
                dto.threshold,
                dto.level,
                dto.color,
            )),
            EffectDto::Cabinet(dto) => Box::new(Cabinet::new(
                dto.id,
                dto.name,
                dto.is_active,
                dto.color,
                dto.ir_file_path,
                dsp_sample_rate,
            )),
            EffectDto::Delay(dto) => Box::new(Delay::new(
                dto.id,
                dto.name,
                dto.is_active,
                dto.color,
                dsp_sample_rate,
                dto.delay_time,
                dto.level,
            ))
        }
    }
}