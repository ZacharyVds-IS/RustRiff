use crate::config::DEFAULT_IR_FILE;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

fn default_ir_file_path() -> String {
    DEFAULT_IR_FILE.to_string()
}

/// Data Transfer Object for the placeholder cabinet effect.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct CabinetDto {
    /// Unique identifier for this effect instance.
    pub id: u32,
    /// Human-readable name displayed in the UI.
    pub name: String,
    /// Whether the effect is currently active or bypassed.
    pub is_active: bool,
    /// UI color for the pedal chassis. Hex string format: `"#rrggbb"`.
    pub color: String,
    /// IR filename resolved from either bundled defaults (`resources/default_ir`)
    /// or the user custom IR directory, depending on what the effect references.
    #[serde(default = "default_ir_file_path")]
    pub ir_file_path: String,
}

