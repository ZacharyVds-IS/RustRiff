use crate::commands::helpers::persist_amp_config;
use crate::services::amp_config_service::AmpConfigPersistenceService;
use crate::services::audio_service::AudioService;
use crate::services::device_service::DeviceService;
use std::sync::Mutex;
use uuid::Uuid;

#[tauri::command]
pub fn set_delay_level(
    audio_service: tauri::State<Mutex<AudioService>>,
    device_service: tauri::State<Mutex<DeviceService>>,
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

    let audio_service = audio_service
        .lock()
        .map_err(|_| "Failed to lock audio service".to_string())?;
    let device_service = device_service
        .lock()
        .map_err(|_| "Failed to lock device service".to_string())?;
    let channel = audio_service
        .channels()
        .iter()
        .find(|c| c.id() == *audio_service.current_channel_id())
        .ok_or("No active channel")?;
    channel.set_effect_param(
        Uuid::parse_str(&effect_id).expect("failed to parse id"),
        "level",
        level,
    )?;
    persist_amp_config(&audio_service, &device_service, &persistence_service);
    Ok(())
}

#[tauri::command]
pub fn set_delay_delay_time(
    audio_service: tauri::State<Mutex<AudioService>>,
    device_service: tauri::State<Mutex<DeviceService>>,
    persistence_service: tauri::State<Mutex<AmpConfigPersistenceService>>,
    effect_id: String,
    delay_time: u32,
) -> Result<(), String> {
    let audio_service = audio_service
        .lock()
        .map_err(|_| "Failed to lock audio service".to_string())?;
    let device_service = device_service
        .lock()
        .map_err(|_| "Failed to lock device service".to_string())?;
    let channel = audio_service
        .channels()
        .iter()
        .find(|c| c.id() == *audio_service.current_channel_id())
        .ok_or("No active channel")?;
    channel.set_effect_param(
        Uuid::parse_str(&effect_id).expect("failed to parse id"),
        "delay_time",
        delay_time,
    )?;
    persist_amp_config(&audio_service, &device_service, &persistence_service);
    Ok(())
}
