use crate::domain::dto::channel_dto::ChannelDto;
use crate::services::audio_service::AudioService;
use std::sync::Mutex;
use tauri::{AppHandle, Emitter};
use tracing::info;


/// Sets the active channel ID in the audio service.
///
/// Updates the currently selected [`Channel`] within the [`AudioService`]
/// by assigning the provided channel ID.
///
/// # Arguments
///
/// * `audio_service` - The shared [`AudioService`] state.
/// * `channel_id` - The identifier of the channel to activate.
///
/// [`Channel`]: crate::domain::channel::Channel
/// [`AudioService`]: crate::services::audio_service::AudioService
#[tauri::command]
pub(crate) fn set_channel_id(audio_service: tauri::State<Mutex<AudioService>>, channel_id: u32) {
    let mut service = audio_service.inner().lock().unwrap();
    service.set_current_channel_id(channel_id);
}


/// Returns the currently active channel ID.
///
/// Retrieves the identifier of the [`Channel`] that is currently selected
/// within the [`AudioService`].
///
/// # Arguments
///
/// * `audio_service` - The shared [`AudioService`] state.
///
/// # Returns
///
/// The ID of the active channel.
///
/// [`Channel`]: crate::domain::channel::Channel
/// [`AudioService`]: crate::services::audio_service::AudioService
#[tauri::command]
pub(crate) fn get_channel_id(audio_service: tauri::State<Mutex<AudioService>>) -> u32 {
    let service = audio_service.inner().lock().unwrap();
    *service.current_channel_id()
}


/// Adds a new channel to the audio service.
///
/// Creates a new [`Channel`] with the given name, registers it with the
/// [`AudioService`], and emits a `channel-added` event to the frontend
/// containing the newly created [`ChannelDto`].
///
/// # Arguments
///
/// * `app` - The Tauri application handle used to emit events.
/// * `audio_service` - The shared [`AudioService`] state.
/// * `channel_name` - The display name of the new channel.
///
/// # Returns
///
/// * `Ok(())` if the channel was created and the event was emitted successfully.
/// * `Err(String)` if emitting the event failed.
///
/// [`Channel`]: crate::domain::channel::Channel
/// [`ChannelDto`]: crate::dto::channel_dto::ChannelDto
/// [`AudioService`]: crate::services::audio_service::AudioService
#[tauri::command]
pub(crate) fn add_channel(app: AppHandle,audio_service: tauri::State<Mutex<AudioService>>, channel_name: String) -> Result<(), String> {
    info!("add_channel command received: {channel_name}");

    let mut service = audio_service.inner().lock().unwrap();
    let channel_id = service.add_channel(channel_name.clone());
    let channel = service.channels().iter().find(|c| c.id() == channel_id).unwrap();
    let channel_dto = ChannelDto::from(channel);

    info!("emitting channel-added event for id={} name={}", channel_dto.id, channel_dto.name);


    app.emit(
        "channel-added",
        channel_dto,
    )
        .map_err(|e| e.to_string())?;

    info!("channel-added event emitted successfully");

    Ok(())
}


/// Returns all available channels.
///
/// Retrieves all [`Channel`] instances managed by the [`AudioService`] and
/// converts them into [`ChannelDto`] representations, including their gain,
/// tone stack settings, and volume.
///
/// # Arguments
///
/// * `audio_service` - The shared [`AudioService`] state.
///
/// # Returns
///
/// A vector of [`ChannelDto`] objects representing all configured channels.
///
/// [`Channel`]: crate::domain::channel::Channel
/// [`ChannelDto`]: crate::dto::channel_dto::ChannelDto
/// [`AudioService`]: crate::services::audio_service::AudioService
#[tauri::command]
pub(crate) fn get_all_channels(
    audio_service: tauri::State<Mutex<AudioService>>,
) -> Vec<ChannelDto> {
    let service = audio_service.inner().lock().unwrap();
    service
        .channels()
        .iter()
        .map(|channel| ChannelDto::from(channel))
        .collect()
}


/// Removes a channel from the audio service.
///
/// Deletes the [`Channel`] with the specified ID from the [`AudioService`].
///
/// # Arguments
///
/// * `audio_service` - The shared [`AudioService`] state.
/// * `channel_id` - The identifier of the channel to remove.
///
/// # Returns
///
/// * `Ok(())` if the channel was removed successfully.
/// * `Err(String)` if removal fails.
///
/// [`Channel`]: crate::domain::channel::Channel
/// [`AudioService`]: crate::services::audio_service::AudioService
#[tauri::command]
pub (crate) fn remove_channel(audio_service: tauri::State<Mutex<AudioService>>, channel_id: u32) -> Result<(), String> {
    let mut service = audio_service.inner().lock().unwrap();
    service.remove_channel(channel_id);
    info!("remove channel {channel_id}");
    Ok(())
}
