use crate::commands::helpers::persist_amp_config;
use crate::services::amp_config_service::AmpConfigPersistenceService;
use crate::services::audio_service::AudioService;
use std::sync::Mutex;
use uuid::Uuid;

#[tauri::command]
pub fn set_delay_level(
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

    let service = audio_service
        .lock()
        .map_err(|_| "Failed to lock audio service".to_string())?;
    let cm = service.channel_manager().lock().unwrap();
    cm.set_effect_parameter(
        Uuid::parse_str(&effect_id).expect("failed to parse id"),
        "level",
        level,
    )?;
    drop(cm);
    persist_amp_config(&service, &persistence_service);
    Ok(())
}

#[tauri::command]
pub fn set_delay_delay_time(
    audio_service: tauri::State<Mutex<AudioService>>,
    persistence_service: tauri::State<Mutex<AmpConfigPersistenceService>>,
    effect_id: String,
    delay_time: u32,
) -> Result<(), String> {
    let service = audio_service
        .lock()
        .map_err(|_| "Failed to lock audio service".to_string())?;
    let cm = service.channel_manager().lock().unwrap();
    cm.set_effect_parameter(
        Uuid::parse_str(&effect_id).expect("failed to parse id"),
        "delay_time",
        delay_time,
    )?;
    drop(cm);
    persist_amp_config(&service, &persistence_service);
    Ok(())
}
