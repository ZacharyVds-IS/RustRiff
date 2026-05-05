use crate::commands::helpers::persist_amp_config;
use crate::services::amp_config_service::AmpConfigPersistenceService;
use crate::services::audio_service::AudioService;
use std::sync::Mutex;

/// Starts the audio loopback on a dedicated background thread.
///
/// Delegates to [`AudioService::start_loopback`] to begin capturing and processing audio.
/// If the loopback is already running, this command is a no-op.
///
/// # Arguments
///
/// * `audio_service` - The shared [`AudioService`] state, accessed via Tauri's state management.
#[tauri::command]
pub(crate) fn start_loopback(
    audio_service: tauri::State<'_, Mutex<AudioService>>,
    persistence_service: tauri::State<'_, Mutex<AmpConfigPersistenceService>>,
) {
    let mut service = audio_service.lock().unwrap();
    service.start_loopback();
    persist_amp_config(&service, &persistence_service);
}
