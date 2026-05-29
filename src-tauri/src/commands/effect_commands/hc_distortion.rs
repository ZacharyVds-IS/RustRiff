use crate::commands::helpers::persist_amp_config;
use crate::services::amp_config_service::AmpConfigPersistenceService;
use crate::services::audio_service::AudioService;
use crate::services::device_service::DeviceService;
use std::sync::Mutex;
use tracing::info;
use uuid::Uuid;

/// # Sets the Clipping Threshold on an HC Distortion Effect
///
/// Adjusts the Drive parameter: lower thresholds produce heavier distortion.
///
/// # Arguments
/// * `effect_id` — ID of the HCDistortion effect to modify
/// * `threshold` — Clipping level in range `(0.0, 1.0]`
///                 * Values < 0.001 are clamped to 0.001
///                 * Values > 1.0 are clamped to 1.0
///
/// # Returns
/// * `Ok(())` — Threshold updated successfully
/// * `Err(String)` — Error if:
///   - Effect not found or parameter update fails
///   - Threshold is NaN or infinite (audio thread safety)
///
/// # Validation
///
/// This command validates the threshold before forwarding to the audio thread:
/// - Rejects NaN and infinite values (would panic in audio processor)
/// - Clamps finite values to safe range `[0.001, 1.0]`
/// - Prevents audio thread crashes from invalid clamp operations

#[tauri::command]
pub fn set_hc_distortion_threshold(
    audio_service: tauri::State<Mutex<AudioService>>,
    device_service: tauri::State<Mutex<DeviceService>>,
    persistence_service: tauri::State<Mutex<AmpConfigPersistenceService>>,
    effect_id: String,
    threshold: f32,
) -> Result<(), String> {
    if !threshold.is_finite() {
        return Err(format!(
            "Invalid threshold: {} (must be finite, not NaN or infinite)",
            threshold
        ));
    }

    let safe_threshold = threshold.clamp(0.001, 1.0);

    let service = audio_service
        .lock()
        .map_err(|_| "Failed to lock audio service".to_string())?;
    let cm = service.channel_manager().lock().unwrap();
    cm.set_effect_parameter(
        Uuid::parse_str(&effect_id).expect("failed to parse id"),
        "threshold",
        safe_threshold,
    )?;
    info!(
        channel_id = cm.current_channel_id().to_string(),
        effect_id,
        threshold = safe_threshold,
        "HCDistortion threshold updated"
    );
    let device_service = device_service
        .lock()
        .map_err(|_| "Failed to lock device service".to_string())?;
    drop(cm);
    persist_amp_config(&service, &device_service, &persistence_service);
    Ok(())
}

/// # Sets the Output Level (Boost) on an HC Distortion Effect
///
/// Adjusts the Level parameter: controls output amplitude after clipping.
///
/// # Arguments
/// * `effect_id` — ID of the HCDistortion effect to modify
/// * `level` — Normalised level in range `[0.0, 1.0]`
///            * `0.0` = unity gain (no boost)
///            * `1.0` = ×2.0 boost
///            * Values outside range are clamped
///
/// # Returns
/// * `Ok(())` — Level updated successfully
/// * `Err(String)` — Error if:
///   - Effect not found or parameter update fails
///   - Level is NaN or infinite (audio thread safety)
///
/// # Validation
///
/// This command validates the level before forwarding to the audio thread:
/// - Rejects NaN and infinite values (would create invalid gain multiplier)
/// - Clamps finite values to range `[0.0, 1.0]`
/// - Maps to internal gain `[1.0, 2.0]` after validation
/// - Prevents audio thread crashes from invalid gain operations

#[tauri::command]
pub fn set_hc_distortion_level(
    audio_service: tauri::State<Mutex<AudioService>>,
    device_service: tauri::State<Mutex<DeviceService>>,
    persistence_service: tauri::State<Mutex<AmpConfigPersistenceService>>,
    effect_id: String,
    level: f32,
) -> Result<(), String> {
    // Validate level before forwarding to audio thread
    if !level.is_finite() {
        return Err(format!(
            "Invalid level: {} (must be finite, not NaN or infinite)",
            level
        ));
    }

    // Clamp to safe range [0.0, 1.0] then map to internal gain [1.0, 2.0]
    let safe_level = level.clamp(0.0, 1.0);
    let gain = 1.0 + safe_level;

    let service = audio_service
        .lock()
        .map_err(|_| "Failed to lock audio service".to_string())?;
    let cm = service.channel_manager().lock().unwrap();
    cm.set_effect_parameter(
        Uuid::parse_str(&effect_id).expect("failed to parse id"),
        "level",
        gain,
    )?;
    info!(
        channel_id = cm.current_channel_id().to_string(),
        effect_id,
        level = safe_level,
        gain,
        "HCDistortion level updated"
    );
    let device_service = device_service
        .lock()
        .map_err(|_| "Failed to lock device service".to_string())?;
    drop(cm);
    persist_amp_config(&service, &device_service, &persistence_service);
    Ok(())
}
