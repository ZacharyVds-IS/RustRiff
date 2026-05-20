use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct DelayDto {
    /// Unique identifier for this effect instance. Used for targeting commands.
    pub id: String,
    /// Human-readable name displayed in the UI pedal. Example: `"Distortion"`.
    pub name: String,
    /// Whether the effect is currently active/processing audio (`true`) or bypassed (`false`).
    /// When `false`, the input signal passes through unchanged.
    pub is_active: bool,
    /// UI colour for the pedal chassis. Hex string format: `"#rrggbb"`.
    pub color: String,
    /// Time (ms) between the original sample and the echo's
    pub delay_time: u32,
    /// Output level of the Echo's in `[0.0, 0.95]`.
    /// - `0.0` = no audio
    /// - `0.95` = almost the same volume as the dry sample
    pub level: f32,
}
