use crate::services::amp_config_service::AmpConfigPersistenceService;
use crate::services::audio_service::AudioService;
use crate::services::device_service::DeviceService;
use std::sync::Mutex;
use tauri::State;
use tracing::{error, info};

/// Snapshots and enqueues amp-config persistence after a successful command mutation.
pub(crate) fn persist_amp_config(
    audio_service: &AudioService,
    device_service: &DeviceService,
    persistence_service_state: &State<'_, Mutex<AmpConfigPersistenceService>>,
) {
    match persistence_service_state.lock() {
        Ok(persistence_service) => {
            //todo:remove after debugging
            info!("Persisting: Audio Drivers: {}", device_service.selected_audio_driver());

            if let Err(err) = persistence_service.persist_from_audio_service(audio_service, device_service) {
                error!("Failed to persist amp config: {err}");
            }
        }
        Err(_) => error!("Failed to lock amp config persistence service"),
    }
}
