use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// Data transfer object describing a single cabinet impulse-response profile.
///
/// Instances are produced by [`FileService::get_all_ir_profiles`] and returned
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct IrProfileDto {
    /// `.wav` filename as stored on disk (e.g. `"vintage-4x12.wav"`).
    /// This value is what gets stored inside [`CabinetDto::ir_file_path`] and
    /// passed back to the backend when creating or restoring a Cabinet effect.
    /// [`CabinetDto::ir_file_path`]: crate::domain::dto::effect::cabinet_dto::CabinetDto::ir_file_path
    pub file_name: String,

    /// Human-readable display name shown in the frontend dropdown.
    /// Derived from `file_name` by stripping the `.wav` extension and replacing
    /// hyphens and underscores with spaces (e.g. `"vintage-4x12.wav"` → `"vintage 4x12"`).
    pub label: String,

    /// `true` when this profile was uploaded by the user and lives in the custom IR
    /// directory rather than the bundled `resources/default_ir` folder.
    /// Only custom profiles may be removed; attempting to remove a default profile
    /// returns an error from [`remove_ir_profile`].
    /// [`remove_ir_profile`]: crate::commands::effect_commands::cabinet_ir::remove_ir_profile
    pub is_custom: bool,

    /// `true` when at least one Cabinet effect in any active channel currently
    /// references this profile by `file_name`.
    /// The frontend uses this to disable the remove-button and prevent deleting
    /// an IR that is actively shaping the tone of a running effect chain.
    pub is_in_use: bool,
}
