use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// # Data Transfer Object for SC Distortion Effect
///
/// `ScDistortionDto` is the serialisable representation of an [`SCDistortion`] effect
/// for communication between the Rust backend and the TypeScript frontend.
///
/// This DTO is automatically generated for TypeScript via the `ts-rs` crate
///
/// [`SCDistortion`]: crate::services::effects::distortion::sc_distortion::SCDistortion
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct ScDistortionDto {
    /// Unique identifier for this effect instance. Used for targeting commands.
    pub id: String,
    /// Human-readable name displayed in the UI pedal. Example: `"Distortion"`.
    pub name: String,
    /// Whether the effect is currently active/processing audio (`true`) or bypassed (`false`).
    /// When `false`, the input signal passes through unchanged.
    pub is_active: bool,
    /// UI colour for the pedal chassis. Hex string format: `"#rrggbb"`.
    pub color: String,
    /// clipping threshold in the range `[0.0, 1.0]`.
    pub threshold: f32,
    /// Normalised output level boost in `[0.0, 1.0]`.
    /// - `0.0` = unity gain (no boost)
    /// - `1.0` = ×2.0 boost
    pub level: f32,
    /// The smoothing factor that determines how much the curve is curved towards the clipping threshold `[1.0,10.0]`.
    pub smoothing: f32,
}
