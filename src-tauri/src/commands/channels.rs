use crate::commands::helpers::persist_amp_config;
use crate::domain::dto::channel_dto::ChannelDto;
use crate::services::amp_config_service::AmpConfigPersistenceService;
use crate::services::audio_service::AudioService;
use std::sync::Mutex;
use tauri::{AppHandle, Emitter};
use tracing::info;
use uuid::Uuid;

#[tauri::command]
pub(crate) fn set_channel_id(
    audio_service: tauri::State<Mutex<AudioService>>,
    persistence_service: tauri::State<Mutex<AmpConfigPersistenceService>>,
    channel_id: String,
) {
    let mut service = audio_service.inner().lock().unwrap();
    service.set_current_channel_id(Uuid::parse_str(&channel_id).expect("failed to parse id"));
    persist_amp_config(&service, &persistence_service);
}

#[tauri::command]
pub(crate) fn get_channel_id(audio_service: tauri::State<Mutex<AudioService>>) -> String {
    let service = audio_service.inner().lock().unwrap();
    let cm = service.channel_manager().lock().unwrap();
    cm.current_channel_id().to_string()
}

#[tauri::command]
pub(crate) fn add_channel(
    app: AppHandle,
    audio_service: tauri::State<Mutex<AudioService>>,
    persistence_service: tauri::State<Mutex<AmpConfigPersistenceService>>,
    channel_name: String,
) -> Result<(), String> {
    info!("add_channel command received: {channel_name}");

    let mut service = audio_service.inner().lock().unwrap();
    let channel_id = service.add_channel(channel_name.clone());
    let cm = service.channel_manager().lock().unwrap();
    let channel = cm.channels().iter().find(|c| c.id() == channel_id).unwrap();
    let channel_dto = ChannelDto::from(channel);
    drop(cm);
    persist_amp_config(&service, &persistence_service);

    info!(
        "emitting channel-added event for id={} name={}",
        channel_dto.id, channel_dto.name
    );

    app.emit("channel-added", channel_dto)
        .map_err(|e| e.to_string())?;

    info!("channel-added event emitted successfully");

    Ok(())
}

#[tauri::command]
pub(crate) fn get_all_channels(
    audio_service: tauri::State<Mutex<AudioService>>,
) -> Vec<ChannelDto> {
    let service = audio_service.inner().lock().unwrap();
    let cm = service.channel_manager().lock().unwrap();
    cm.to_channel_dtos()
}

#[tauri::command]
pub(crate) fn remove_channel(
    audio_service: tauri::State<Mutex<AudioService>>,
    persistence_service: tauri::State<Mutex<AmpConfigPersistenceService>>,
    channel_id: String,
) -> Result<(), String> {
    let mut service = audio_service.inner().lock().unwrap();
    service.remove_channel(Uuid::parse_str(&channel_id).expect("failed to parse id"));
    persist_amp_config(&service, &persistence_service);
    info!("remove channel {channel_id}");
    Ok(())
}
