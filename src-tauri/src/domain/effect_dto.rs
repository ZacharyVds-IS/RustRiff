use serde::{Deserialize, Serialize};
use crate::domain::effect::Effect;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EffectDto {
    pub id: u32,
    pub name: String,
    pub is_active: bool,
}


impl From<&dyn Effect> for EffectDto {
    fn from(effect: &dyn Effect) -> Self {
        Self {
            id: effect.id(),
            name: effect.name().to_string(),
            is_active: effect.is_active(),
        }
    }
}

