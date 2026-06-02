use crate::commands::helpers::persist_amp_config;
use crate::services::amp_config_service::AmpConfigPersistenceService;
use crate::services::audio_service::AudioService;
use crate::services::device_service::DeviceService;
use std::sync::Mutex;
use tracing::info;
use uuid::Uuid;

/// # Sets the Pedal Position on a Wah Effect
///
/// Adjusts the wah sweep position:
/// - `0.0` = heel‑down (dark)
/// - `1.0` = toe‑down (bright)
///
/// # Arguments
/// * `effect_id` — ID of the Wah effect to modify
/// * `pedal_position` — Normalised sweep position in range `[0.0, 1.0]`
///
/// # Returns
/// * `Ok(())` — Pedal position updated successfully
/// * `Err(String)` — Error if:
///   - Effect not found
///   - Parameter update fails
///   - Value is NaN or infinite
///
/// # Validation
/// - Rejects NaN and infinite values
/// - Clamps to `[0.0, 1.0]`
/// - Prevents invalid values from reaching the audio thread
#[tauri::command]
pub fn set_wah_pedal_position(
    audio_service: tauri::State<Mutex<AudioService>>,
    device_service: tauri::State<Mutex<DeviceService>>,
    persistence_service: tauri::State<Mutex<AmpConfigPersistenceService>>,
    effect_id: String,
    pedal_position: f32,
) -> Result<(), String> {
    // Validate before touching audio thread
    if !pedal_position.is_finite() {
        return Err(format!(
            "Invalid pedal_position: {} (must be finite, not NaN or infinite)",
            pedal_position
        ));
    }

    // Clamp to safe range
    let safe_position = pedal_position.clamp(0.0, 1.0);

    let audio_service = audio_service
        .lock()
        .map_err(|_| "Failed to lock audio service".to_string())?;
    let cm = audio_service.channel_manager().lock().unwrap();
    cm.set_effect_parameter(
        Uuid::parse_str(&effect_id).expect("failed to parse id"),
        "pedal_position",
        safe_position,
    )?;
    info!(
        channel_id = cm.current_channel_id().to_string(),
        effect_id,
        pedal_position = safe_position,
        "Wah pedal position updated"
    );
    drop(cm);

    let device_service = device_service
        .lock()
        .map_err(|_| "Failed to lock device service".to_string())?;

    // Persist config
    persist_amp_config(&audio_service, &device_service, &persistence_service);

    Ok(())
}
