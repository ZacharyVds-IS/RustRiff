use crate::services::amp_config_service::AmpConfigPersistenceService;
use crate::services::audio_service::AudioService;
use std::sync::Mutex;
use tauri::State;
use tracing::error;

/// Persists the current amplifier state after a successful command mutation.
pub(crate) fn persist_amp_config(
    service: &AudioService,
    persistence_service_state: &State<'_, Mutex<AmpConfigPersistenceService>>,
) {
    match persistence_service_state.lock() {
        Ok(persistence_service) => {
            if let Err(err) = persistence_service.persist_from_audio_service(service) {
                error!("Failed to persist amp config: {err}");
            }
        }
        Err(_) => error!("Failed to lock amp config persistence service"),
    }
}
