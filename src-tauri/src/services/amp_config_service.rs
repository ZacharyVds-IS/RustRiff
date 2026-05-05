use crate::domain::dto::amp_config_dto::AmpConfigDto;
use crate::infrastructure::persistence::amp_config_persistence_trait::AmpConfigPersistence;
use crate::services::audio_service::AudioService;

/// Application service coordinating amplifier configuration persistence.
///
/// Command handlers should depend on this service rather than on a repository
/// directly. That keeps infrastructure details out of the command layer and
/// provides one place to centralize snapshot-related behavior.
pub struct AmpConfigPersistenceService {
    repository: Box<dyn AmpConfigPersistence>,
}

impl AmpConfigPersistenceService {
    /// Creates the service with the chosen persistence backend.
    pub fn new(repository: Box<dyn AmpConfigPersistence>) -> Self {
        Self { repository }
    }

    /// Loads the last persisted amplifier configuration, if any.
    pub fn load_amp_config(&self) -> Result<Option<AmpConfigDto>, String> {
        self.repository.load()
    }

    /// Captures a snapshot from the current [`AudioService`] state and persists it.
    ///
    /// This is the primary method used by mutating Tauri commands after they
    /// successfully update amplifier state.
    pub fn persist_from_audio_service(&self, audio_service: &AudioService) -> Result<(), String> {
        let snapshot = AmpConfigDto::from_service(audio_service);
        self.repository.save(&snapshot)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infrastructure::audio_handler::MockAudioHandlerTrait;
    use std::sync::{Arc, Mutex};

    struct SpyRepositoryState {
        saved_configs: Mutex<Vec<AmpConfigDto>>,
        load_result: Mutex<Result<Option<AmpConfigDto>, String>>,
        save_result: Mutex<Result<(), String>>,
    }

    impl SpyRepositoryState {
        fn new() -> Self {
            Self {
                saved_configs: Mutex::new(Vec::new()),
                load_result: Mutex::new(Ok(None)),
                save_result: Mutex::new(Ok(())),
            }
        }
    }

    struct SpyRepository {
        state: Arc<SpyRepositoryState>,
    }

    impl AmpConfigPersistence for SpyRepository {
        fn load(&self) -> Result<Option<AmpConfigDto>, String> {
            self.state
                .load_result
                .lock()
                .expect("load_result should be lockable")
                .clone()
        }

        fn save(&self, config: &AmpConfigDto) -> Result<(), String> {
            self.state
                .saved_configs
                .lock()
                .expect("saved_configs should be lockable")
                .push(config.clone());

            self.state
                .save_result
                .lock()
                .expect("save_result should be lockable")
                .clone()
        }
    }

    #[test]
    fn load_amp_config_returns_repository_value() {
        let state = Arc::new(SpyRepositoryState::new());
        let expected = AmpConfigDto {
            master_volume: 0.72,
            is_active: false,
            channels: Vec::new(),
            current_channel: 2,
        };
        *state
            .load_result
            .lock()
            .expect("load_result should be lockable") = Ok(Some(expected.clone()));

        let service = AmpConfigPersistenceService::new(Box::new(SpyRepository { state }));
        let loaded = service.load_amp_config().expect("load should succeed");

        assert!(loaded.is_some());
        assert_eq!(loaded.expect("value should be present").current_channel, expected.current_channel);
    }

    #[test]
    fn load_amp_config_propagates_repository_error() {
        let state = Arc::new(SpyRepositoryState::new());
        *state
            .load_result
            .lock()
            .expect("load_result should be lockable") = Err("load failed".to_string());

        let service = AmpConfigPersistenceService::new(Box::new(SpyRepository { state }));
        let err = service.load_amp_config().expect_err("load should fail");

        assert_eq!(err, "load failed");
    }

    #[test]
    fn persist_from_audio_service_saves_snapshot() {
        let state = Arc::new(SpyRepositoryState::new());
        let service = AmpConfigPersistenceService::new(Box::new(SpyRepository {
            state: Arc::clone(&state),
        }));

        let mock = MockAudioHandlerTrait::new();
        let audio_service = AudioService::new_with_handler(Arc::new(mock));

        service
            .persist_from_audio_service(&audio_service)
            .expect("persist should succeed");

        let saved = state
            .saved_configs
            .lock()
            .expect("saved_configs should be lockable");
        assert_eq!(saved.len(), 1);
        assert_eq!(saved[0].current_channel, 0);
        assert!(!saved[0].is_active);
    }

    #[test]
    fn persist_from_audio_service_propagates_repository_error() {
        let state = Arc::new(SpyRepositoryState::new());
        *state
            .save_result
            .lock()
            .expect("save_result should be lockable") = Err("save failed".to_string());

        let service = AmpConfigPersistenceService::new(Box::new(SpyRepository { state }));
        let mock = MockAudioHandlerTrait::new();
        let audio_service = AudioService::new_with_handler(Arc::new(mock));

        let err = service
            .persist_from_audio_service(&audio_service)
            .expect_err("persist should fail");

        assert_eq!(err, "save failed");
    }
}

