use crate::domain::dto::effect::cabinet_dto::CabinetDto;
use crate::domain::dto::effect::delay_dto::DelayDto;
use crate::domain::dto::effect::hcdistortion_dto::HcDistortionDto;
use crate::domain::dto::effect::scdistortion_dto::ScDistortionDto;
use crate::domain::dto::effect::wah_dto::WahDto;
use crate::domain::effect::Effect;
use crate::services::effects::cabinet::cabinet::Cabinet;
use crate::services::effects::delay::delay::Delay;
use crate::services::effects::distortion::hc_distortion::HCDistortion;
use crate::services::effects::distortion::sc_distortion::SCDistortion;
use crate::services::effects::wah::wah::Wah;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A serialisable, tagged representation of any effect in the signal chain.
///
/// Uses serde's adjacently-tagged format so that both the Rust serialisation
/// and the TypeScript typegen agree on the wire shape:
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "data")]
pub enum EffectDto {
    /// Hard-clipping distortion effect.
    HCDistortion(HcDistortionDto),
    /// Soft-clipping distortion effect.
    SCDistortion(ScDistortionDto),
    /// Delay effect.
    Delay(DelayDto),
    /// Placeholder impulse-response cabinet effect.
    Cabinet(CabinetDto),
    /// Resonant bandpass filter wah-wah effect.
    Wah(WahDto),
}

impl EffectDto {
    pub fn add_to_domain(self, dsp_sample_rate: u32) -> Box<dyn Effect> {
        match self {
            EffectDto::HCDistortion(dto) => Box::new(HCDistortion::new(
                Uuid::new_v4(),
                dto.name,
                dto.is_active,
                dto.threshold,
                dto.level,
                dto.color,
            )),
            EffectDto::SCDistortion(dto) => Box::new(SCDistortion::new(
                Uuid::new_v4(),
                dto.name,
                dto.is_active,
                dto.threshold,
                dto.level,
                dto.smoothing,
                dto.color,
            )),
            EffectDto::Cabinet(dto) => Box::new(Cabinet::new(
                Uuid::new_v4(),
                dto.name,
                dto.is_active,
                dto.color,
                dto.ir_file_path,
                dsp_sample_rate,
            )),
            EffectDto::Delay(dto) => Box::new(Delay::new(
                Uuid::new_v4(),
                dto.name,
                dto.is_active,
                dto.color,
                dsp_sample_rate,
                dto.delay_time,
                dto.level,
            )),
            EffectDto::Wah(dto) => Box::new(Wah::new(
                Uuid::new_v4(),
                dto.name,
                dto.color,
                dto.is_active,
                dto.pedal_position,
                dsp_sample_rate as f32,
            )),
        }
    }

    pub fn to_domain(self, dsp_sample_rate: u32) -> Box<dyn Effect> {
        match self {
            EffectDto::HCDistortion(dto) => Box::new(HCDistortion::new(
                Uuid::parse_str(dto.id.as_str()).expect("invalid uuid"),
                dto.name,
                dto.is_active,
                dto.threshold,
                dto.level,
                dto.color,
            )),
            EffectDto::SCDistortion(dto) => Box::new(SCDistortion::new(
                Uuid::parse_str(dto.id.as_str()).expect("invalid uuid"),
                dto.name,
                dto.is_active,
                dto.threshold,
                dto.level,
                dto.smoothing,
                dto.color,
            )),
            EffectDto::Cabinet(dto) => Box::new(Cabinet::new(
                Uuid::parse_str(dto.id.as_str()).expect("invalid uuid"),
                dto.name,
                dto.is_active,
                dto.color,
                dto.ir_file_path,
                dsp_sample_rate,
            )),
            EffectDto::Delay(dto) => Box::new(Delay::new(
                Uuid::parse_str(dto.id.as_str()).expect("invalid uuid"),
                dto.name,
                dto.is_active,
                dto.color,
                dsp_sample_rate,
                dto.delay_time,
                dto.level,
            )),
            EffectDto::Wah(dto) => Box::new(Wah::new(
                Uuid::parse_str(dto.id.as_str()).expect("invalid uuid"),
                dto.name,
                dto.color,
                dto.is_active,
                dto.pedal_position,
                dsp_sample_rate as f32,
            )),
        }
    }
}
