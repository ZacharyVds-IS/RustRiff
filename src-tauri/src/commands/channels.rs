use crate::domain::channel_dto::ChannelDto;
use crate::services::audio_service::AudioService;
use std::sync::Mutex;
use tauri::async_runtime::channel;
use tauri::{AppHandle, Emitter};
use tracing::info;

#[tauri::command]
pub(crate) fn set_channel_id(audio_service: tauri::State<Mutex<AudioService>>, channel_id: u32) {
    let mut service = audio_service.inner().lock().unwrap();
    service.set_current_channel_id(channel_id);
}

#[tauri::command]
pub(crate) fn get_channel_id(audio_service: tauri::State<Mutex<AudioService>>) -> u32 {
    let service = audio_service.inner().lock().unwrap();
    *service.current_channel_id()
}

#[tauri::command]
pub(crate) fn add_channel(app: AppHandle,audio_service: tauri::State<Mutex<AudioService>>, channel_name: String) -> Result<(), String> {
    info!("add_channel command received: {channel_name}");

    let mut service = audio_service.inner().lock().unwrap();
    let channel = service.add_channel(channel_name.clone());
    let channel_dto = ChannelDto::from(&channel);

    info!("emitting channel-added event for id={} name={}", channel_dto.id, channel_dto.name);


    app.emit(
        "channel-added",
        channel_dto,
    )
        .map_err(|e| e.to_string())?;

    info!("channel-added event emitted successfully");

    Ok(())
}

#[tauri::command]
pub(crate) fn get_all_channels(
    audio_service: tauri::State<Mutex<AudioService>>,
) -> Vec<ChannelDto> {
    let service = audio_service.inner().lock().unwrap();
    service
        .channels()
        .iter()
        .map(|channel| ChannelDto {
            id: channel.id(),
            name: channel.name().clone(),
            gain: channel.gain().load(std::sync::atomic::Ordering::Relaxed),
            tone_stack: crate::domain::tone_stack_dto::ToneStackDto {
                bass: channel
                    .tone_stack()
                    .bass()
                    .load(std::sync::atomic::Ordering::Relaxed),
                middle: channel
                    .tone_stack()
                    .middle()
                    .load(std::sync::atomic::Ordering::Relaxed),
                treble: channel
                    .tone_stack()
                    .treble()
                    .load(std::sync::atomic::Ordering::Relaxed),
            },
            volume: channel.volume().load(std::sync::atomic::Ordering::Relaxed),
        })
        .collect()
}

#[tauri::command]
pub (crate) fn remove_channel(audio_service: tauri::State<Mutex<AudioService>>, channel_id: u32) -> Result<(), String> {
    let mut service = audio_service.inner().lock().unwrap();
    service.remove_channel(channel_id);
    info!("remove channel {channel_id}");
    Ok(())
}
