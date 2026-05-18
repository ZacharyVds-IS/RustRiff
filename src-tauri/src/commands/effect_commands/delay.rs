use crate::commands::helpers::persist_amp_config;
use crate::services::amp_config_service::AmpConfigPersistenceService;
use crate::services::audio_service::AudioService;
use std::sync::Mutex;
use tracing::info;

#[tauri::command]
pub fn set_delay_level(
    audio_service: tauri::State<Mutex<AudioService>>,
    persistence_service: tauri::State<Mutex<AmpConfigPersistenceService>>,
    effect_id: u32,
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
    let channel = service
        .channels()
        .iter()
        .find(|c| c.id() == *service.current_channel_id())
        .ok_or("No active channel")?;
    channel.set_effect_param(effect_id, "level", level)?;
    info!(
        channel_id = *service.current_channel_id(),
        effect_id, level, "Delay level updated"
    );
    persist_amp_config(&service, &persistence_service);
    Ok(())
}

#[tauri::command]
pub fn set_delay_delay_time(
    audio_service: tauri::State<Mutex<AudioService>>,
    persistence_service: tauri::State<Mutex<AmpConfigPersistenceService>>,
    effect_id: u32,
    delay_time: u32,
) -> Result<(), String> {
    let service = audio_service
        .lock()
        .map_err(|_| "Failed to lock audio service".to_string())?;
    let channel = service
        .channels()
        .iter()
        .find(|c| c.id() == *service.current_channel_id())
        .ok_or("No active channel")?;
    channel.set_effect_param(effect_id, "delay_time", delay_time)?;
    /* info!(
        channel_id = *service.current_channel_id(),
        effect_id, delay_time, "Delay delay_time updated"
    );*/
    persist_amp_config(&service, &persistence_service);
    Ok(())
}
