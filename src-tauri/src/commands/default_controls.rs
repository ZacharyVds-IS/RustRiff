use crate::commands::helpers::persist_amp_config;
use crate::domain::dto::amp_config_dto::AmpConfigDto;
use crate::domain::dto::tone_stack_dto::ToneStackDto;
use crate::services::amp_config_service::AmpConfigPersistenceService;
use crate::services::audio_service::AudioService;
use std::sync::Mutex;

/// Retrieves the current amplifier configuration as an [`AmpConfigDto`].
///
/// This command captures the state of gain, master volume, and other parameters
/// from the [`AudioService`].
///
/// # Arguments
///
/// * `audio_service` - The shared [`AudioService`] state, accessed via Tauri's state management.
///
/// # Returns
///
/// Returns `Ok(AmpConfigDto)` on success, or `Err(String)` if the service state cannot be locked.
#[tauri::command]
pub fn get_amp_config(
    audio_service: tauri::State<'_, Mutex<AudioService>>
) -> Result<AmpConfigDto, String> {
    let service = audio_service.lock()
        .map_err(|_| "Failed to lock audio service".to_string())?;

    Ok(AmpConfigDto::from_service(&service))
}

/// Toggles the audio loopback on or off.
///
/// Delegates to [`AudioService::toggle_loopback`] to start or stop audio processing.
///
/// # Arguments
///
/// * `audio_service` - The shared [`AudioService`] state.
/// * `is_on` - Whether to enable (`true`) or disable (`false`) the loopback.
#[tauri::command]
pub(crate) fn toggle_on_off(
    audio_service: tauri::State<Mutex<AudioService>>,
    persistence_service: tauri::State<Mutex<AmpConfigPersistenceService>>,
    is_on: bool,
) {
    let mut service = audio_service.inner().lock().unwrap();
    service.toggle_loopback(is_on);
    persist_amp_config(&service, &persistence_service);
}

/// Sets the input gain level for the amplifier.
///
/// Applies the gain value to the [`Channel`] within the [`AudioService`].
///
/// # Arguments
///
/// * `audio_service` - The shared [`AudioService`] state.
/// * `gain` - The gain value (must be a positive value).
///
/// [`Channel`]: crate::domain::channel::Channel
#[tauri::command]
pub(crate) fn set_gain(
    audio_service: tauri::State<Mutex<AudioService>>,
    persistence_service: tauri::State<Mutex<AmpConfigPersistenceService>>,
    gain: f32,
) {
    let service = audio_service.inner().lock().unwrap();
    service.channels().iter().find(|c| c.id() == *service.current_channel_id()).unwrap().set_gain(gain);
    persist_amp_config(&service, &persistence_service);
}

/// Sets the master volume level for the amplifier.
///
/// Applies the master volume value to the [`AudioService`].
///
/// # Arguments
///
/// * `audio_service` - The shared [`AudioService`] state.
/// * `master_volume` - The master volume value (must be positive).
///
/// [`AudioService`]: crate::services::audio_service::AudioService
#[tauri::command]
pub(crate) fn set_master_volume(
    audio_service: tauri::State<Mutex<AudioService>>,
    persistence_service: tauri::State<Mutex<AmpConfigPersistenceService>>,
    master_volume: f32,
) {
    let service = audio_service.inner().lock().unwrap();
    service.set_master_volume(master_volume);
    persist_amp_config(&service, &persistence_service);
}


/// Sets the tone stack configuration for the current channel.
///
/// Applies the provided [`ToneStackDto`] to the active [`Channel`] within the
/// [`AudioService`].
///
/// # Arguments
///
/// * `audio_service` - The shared [`AudioService`] state.
/// * `tone_stack` - The tone stack configuration to apply.
///
/// [`Channel`]: crate::domain::channel::Channel
/// [`AudioService`]: crate::services::audio_service::AudioService
#[tauri::command]
pub(crate) fn set_tone_stack(
    audio_service: tauri::State<Mutex<AudioService>>,
    persistence_service: tauri::State<Mutex<AmpConfigPersistenceService>>,
    tone_stack: ToneStackDto,
){
    let service = audio_service.inner().lock().unwrap();
    service.channels().iter().find(|c| c.id() == *service.current_channel_id()).unwrap().set_tone_stack(tone_stack);
    persist_amp_config(&service, &persistence_service);
}


/// Sets the bass level for the current channel.
///
/// Updates the bass parameter of the active [`Channel`] within the
/// [`AudioService`].
///
/// # Arguments
///
/// * `audio_service` - The shared [`AudioService`] state.
/// * `bass` - The bass level value.
///
/// [`Channel`]: crate::domain::channel::Channel
/// [`AudioService`]: crate::services::audio_service::AudioService
#[tauri::command]
pub(crate) fn set_bass(
    audio_service: tauri::State<Mutex<AudioService>>,
    persistence_service: tauri::State<Mutex<AmpConfigPersistenceService>>,
    bass: f32,
){
    let service = audio_service.inner().lock().unwrap();
    service.channels().iter().find(|c| c.id() == *service.current_channel_id()).unwrap().set_bass(bass);
    persist_amp_config(&service, &persistence_service);
}



/// Sets the middle frequency level for the current channel.
///
/// Updates the mid-range parameter of the active [`Channel`] within the
/// [`AudioService`].
///
/// # Arguments
///
/// * `audio_service` - The shared [`AudioService`] state.
/// * `middle` - The middle frequency level value.
///
/// [`Channel`]: crate::domain::channel::Channel
/// [`AudioService`]: crate::services::audio_service::AudioService
#[tauri::command]
pub(crate) fn set_middle(
    audio_service: tauri::State<Mutex<AudioService>>,
    persistence_service: tauri::State<Mutex<AmpConfigPersistenceService>>,
    middle: f32,
){
    let service = audio_service.inner().lock().unwrap();
    service.channels().iter().find(|c| c.id() == *service.current_channel_id()).unwrap().set_middle(middle);
    persist_amp_config(&service, &persistence_service);
}


/// Sets the treble level for the current channel.
///
/// Updates the high-frequency parameter of the active [`Channel`] within the
/// [`AudioService`].
///
/// # Arguments
///
/// * `audio_service` - The shared [`AudioService`] state.
/// * `treble` - The treble level value.
///
/// [`Channel`]: crate::domain::channel::Channel
/// [`AudioService`]: crate::services::audio_service::AudioService
#[tauri::command]
pub(crate) fn set_treble(
    audio_service: tauri::State<Mutex<AudioService>>,
    persistence_service: tauri::State<Mutex<AmpConfigPersistenceService>>,
    treble: f32,
){
    let service = audio_service.inner().lock().unwrap();
    service.channels().iter().find(|c| c.id() == *service.current_channel_id()).unwrap().set_treble(treble);
    persist_amp_config(&service, &persistence_service);
}

/// Sets the output volume for the current channel.
///
/// Applies the volume level to the active [`Channel`] within the
/// [`AudioService`].
///
/// # Arguments
///
/// * `audio_service` - The shared [`AudioService`] state.
/// * `volume` - The volume level value.
///
/// [`Channel`]: crate::domain::channel::Channel
/// [`AudioService`]: crate::services::audio_service::AudioService
#[tauri::command]
pub(crate) fn set_volume(
    audio_service: tauri::State<Mutex<AudioService>>,
    persistence_service: tauri::State<Mutex<AmpConfigPersistenceService>>,
    volume: f32,
){
    let service = audio_service.inner().lock().unwrap();
    service.channels().iter().find(|c| c.id() == *service.current_channel_id()).unwrap().set_volume(volume);
    persist_amp_config(&service, &persistence_service);
}