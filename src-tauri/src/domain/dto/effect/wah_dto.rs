use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// # Data Transfer Object for Wah Effect
///
/// `WahDto` is the serialisable representation of a [`Wah`] effect
/// for communication between the Rust backend and the TypeScript frontend.
///
/// This DTO is automatically generated for TypeScript via the `ts-rs` crate.
///
/// [`Wah`]: crate::services::effects::wah::Wah
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct WahDto {
    /// Unique identifier for this effect instance. Used for targeting commands.
    pub id: String,
    /// Human-readable name displayed in the UI pedal. Example: `"Wah"`.
    pub name: String,
    /// Whether the effect is currently active/processing audio (`true`) or bypassed (`false`).
    /// When `false`, the input signal passes through unchanged.
    pub is_active: bool,
    /// UI color for the pedal chassis. Hex string format: `"#rrggbb"`.
    pub color: String,
    /// Pedal expression value in `[0.0, 1.0]`.
    /// - `0.0` = Heel down (Muffled/Bass)
    /// - `1.0` = Toe down (Bright/Treble)
    pub pedal_position: f32,
}
