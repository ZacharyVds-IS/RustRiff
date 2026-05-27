use crate::commands::helpers::persist_amp_config;
use crate::services::amp_config_service::AmpConfigPersistenceService;
use crate::services::audio_service::AudioService;
use std::sync::Mutex;
use uuid::Uuid;

#[tauri::command]
pub fn set_sc_distortion_threshold(
    audio_service: tauri::State<Mutex<AudioService>>,
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
    drop(cm);
    persist_amp_config(&service, &persistence_service);
    Ok(())
}

#[tauri::command]
pub fn set_sc_distortion_level(
    audio_service: tauri::State<Mutex<AudioService>>,
    persistence_service: tauri::State<Mutex<AmpConfigPersistenceService>>,
    effect_id: String,
    level: f32,
) -> Result<(), String> {
    if !level.is_finite() {
        return Err(format!(
            "Invalid level: {} (must be finite, not NaN or infinite)",
            level
        ));
    }

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
    drop(cm);
    persist_amp_config(&service, &persistence_service);
    Ok(())
}

#[tauri::command]
pub fn set_sc_distortion_smoothing(
    audio_service: tauri::State<Mutex<AudioService>>,
    persistence_service: tauri::State<Mutex<AmpConfigPersistenceService>>,
    effect_id: String,
    smoothing: f32,
) -> Result<(), String> {
    if !smoothing.is_finite() {
        return Err(format!(
            "Invalid smoothing: {} (must be finite, not NaN or infinite)",
            smoothing
        ));
    }

    let safe_smoothing = smoothing.clamp(1.0, 10.0);

    let service = audio_service
        .lock()
        .map_err(|_| "Failed to lock audio service".to_string())?;
    let cm = service.channel_manager().lock().unwrap();
    cm.set_effect_parameter(
        Uuid::parse_str(&effect_id).expect("failed to parse id"),
        "smoothing",
        safe_smoothing,
    )?;
    drop(cm);
    persist_amp_config(&service, &persistence_service);
    Ok(())
}
