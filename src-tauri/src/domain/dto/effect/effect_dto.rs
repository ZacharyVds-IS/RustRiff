use crate::domain::dto::effect::hcdistortion_dto::HcDistortionDto;
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
