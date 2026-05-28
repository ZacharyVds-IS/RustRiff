use crate::domain::dto::amp_config_dto::AmpConfigDto;
use crate::domain::dto::midi_mapping_dto::MidiMappingDto;
use crate::infrastructure::persistence::amp_config_persistence_trait::AmpConfigPersistence;
use crate::services::audio_service::AudioService;
use std::sync::{Arc, Condvar, Mutex};
use std::thread;
use tracing::error;

/// Application service coordinating amplifier configuration persistence.
///
/// Command handlers should depend on this service rather than on a repository
/// directly. That keeps infrastructure details out of the command layer and
/// provides one place to centralize snapshot-related behavior.
pub struct AmpConfigPersistenceService {
    repository: Arc<dyn AmpConfigPersistence>,
    pending_snapshot: Arc<(Mutex<Option<AmpConfigDto>>, Condvar)>,
}

impl AmpConfigPersistenceService {
    /// Creates the service with the chosen persistence backend.
    pub fn new(repository: Box<dyn AmpConfigPersistence>) -> Self {
        let repository: Arc<dyn AmpConfigPersistence> = Arc::from(repository);
        let pending_snapshot = Arc::new((Mutex::new(None), Condvar::new()));
        let worker_pending_snapshot = Arc::clone(&pending_snapshot);
        let worker_repository = Arc::clone(&repository);

        // Persist snapshots on a single background worker to keep command
        // paths non-blocking. The pending slot is single-item and
        // overwrite-only: the newest snapshot always wins.
        thread::spawn(move || loop {
            let latest_snapshot = {
                let (lock, cv) = &*worker_pending_snapshot;
                let mut pending = lock
                    .lock()
                    .expect("pending snapshot lock should be available");
                while pending.is_none() {
                    pending = cv
                        .wait(pending)
                        .expect("pending snapshot lock should be available after wait");
                }
                pending
                    .take()
                    .expect("snapshot should be available when worker wakes")
            };

            if let Err(err) = worker_repository.save(&latest_snapshot) {
                error!("Failed to persist amp config snapshot in background worker: {err}");
            }
        });

        Self {
            repository,
            pending_snapshot,
        }
    }

    /// Loads the last persisted amplifier configuration, if any.
    pub fn load_amp_config(&self) -> Result<Option<AmpConfigDto>, String> {
        self.repository.load()
    }

    /// Captures a snapshot from the current [`AudioService`] state and
    /// enqueues it for persistence.
    ///
    /// `midi_bindings` is read from disk and merged into the snapshot so that
    /// a save triggered by an amp-state change never silently drops the
    /// bindings that `MidiService` wrote on its last mutation.
    ///
    /// Disk I/O is executed by the background worker thread so command
    /// handlers return quickly.
    pub fn persist_from_audio_service(&self, audio_service: &AudioService) -> Result<(), String> {
        let mut snapshot = AmpConfigDto::from_service(audio_service);

        // Preserve whatever MIDI bindings are currently on disk so that an
        // amp-config save never clobbers a binding written by MidiService.
        snapshot.midi_bindings = self
            .repository
            .load()
            .unwrap_or_default()
            .unwrap_or_default()
            .midi_bindings;

        self.persist_snapshot(snapshot)
    }

    /// Enqueues a precomputed snapshot for asynchronous persistence.
    pub fn persist_snapshot(&self, snapshot: AmpConfigDto) -> Result<(), String> {
        let (lock, cv) = &*self.pending_snapshot;
        let mut pending = lock
            .lock()
            .map_err(|_| "Amp config persistence lock is unavailable".to_string())?;
        *pending = Some(snapshot);
        cv.notify_one();
        Ok(())
    }

    /// Merges the supplied MIDI bindings into the latest on-disk snapshot and
    /// enqueues the result for asynchronous persistence.
    ///
    /// This is the method `MidiService` calls after every `add_mapping` /
    /// `remove_mapping` so that a single background worker owns all writes
    /// and there is no risk of two concurrent saves racing on the file.
    pub fn persist_midi_bindings(&self, bindings: Vec<MidiMappingDto>) -> Result<(), String> {
        // Read the current on-disk snapshot so we can splice in the new
        // bindings without touching amp-config fields.
        let mut snapshot = self
            .repository
            .load()
            .unwrap_or_default()
            .unwrap_or_default();

        snapshot.midi_bindings = bindings;
        self.persist_snapshot(snapshot)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::channel_manager::ChannelManager;
    use crate::domain::dto::midi_mapping_dto::MidiMappingDto;
    use crate::domain::midi_target_parameter::MidiTargetParameter;
    use crate::infrastructure::audio_handler::MockAudioHandlerTrait;
    use std::sync::{Arc, Condvar, Mutex};
    use std::time::Duration;

    // ── Spy repository ────────────────────────────────────────────────────────

    struct SpyRepositoryState {
        saved_configs: Mutex<Vec<AmpConfigDto>>,
        saved_configs_cv: Condvar,
        load_result: Mutex<Result<Option<AmpConfigDto>, String>>,
        save_result: Mutex<Result<(), String>>,
        save_started_count: Mutex<usize>,
        save_started_cv: Condvar,
        block_saves: Mutex<bool>,
        block_saves_cv: Condvar,
    }

    impl SpyRepositoryState {
        fn new() -> Self {
            Self {
                saved_configs: Mutex::new(Vec::new()),
                saved_configs_cv: Condvar::new(),
                load_result: Mutex::new(Ok(None)),
                save_result: Mutex::new(Ok(())),
                save_started_count: Mutex::new(0),
                save_started_cv: Condvar::new(),
                block_saves: Mutex::new(false),
                block_saves_cv: Condvar::new(),
            }
        }

        fn wait_for_saved_count(
            &self,
            minimum_count: usize,
            timeout: Duration,
        ) -> Vec<AmpConfigDto> {
            let mut saved = self
                .saved_configs
                .lock()
                .expect("saved_configs should be lockable");
            let wait_result = self
                .saved_configs_cv
                .wait_timeout_while(saved, timeout, |configs| configs.len() < minimum_count)
                .expect("saved_configs should remain lockable while waiting");
            saved = wait_result.0;
            saved.clone()
        }

        fn wait_for_save_started_count(&self, minimum_count: usize, timeout: Duration) {
            let started = self
                .save_started_count
                .lock()
                .expect("save_started_count should be lockable");
            let _ = self
                .save_started_cv
                .wait_timeout_while(started, timeout, |count| *count < minimum_count)
                .expect("save_started_count should remain lockable while waiting");
        }

        fn set_block_saves(&self, should_block: bool) {
            let mut block_saves = self
                .block_saves
                .lock()
                .expect("block_saves should be lockable");
            *block_saves = should_block;
            if !should_block {
                self.block_saves_cv.notify_all();
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
            {
                let mut started_count = self
                    .state
                    .save_started_count
                    .lock()
                    .expect("save_started_count should be lockable");
                *started_count += 1;
                self.state.save_started_cv.notify_all();
            }

            let mut block_saves = self
                .state
                .block_saves
                .lock()
                .expect("block_saves should be lockable");
            while *block_saves {
                block_saves = self
                    .state
                    .block_saves_cv
                    .wait(block_saves)
                    .expect("block_saves should remain lockable while waiting");
            }
            drop(block_saves);

            self.state
                .saved_configs
                .lock()
                .expect("saved_configs should be lockable")
                .push(config.clone());
            self.state.saved_configs_cv.notify_all();

            self.state
                .save_result
                .lock()
                .expect("save_result should be lockable")
                .clone()
        }
    }

    // ── Helper ────────────────────────────────────────────────────────────────

    fn make_audio_service() -> AudioService {
        let mock = MockAudioHandlerTrait::new();
        AudioService::new_with_handler(Arc::new(mock), Arc::new(Mutex::new(ChannelManager::new())))
    }

    fn sample_binding() -> MidiMappingDto {
        MidiMappingDto {
            channel: 1,
            cc_number: 11,
            effect_id: uuid::Uuid::new_v4().to_string(),
            parameter: MidiTargetParameter::WahPedalPosition,
        }
    }

    // ── Tests ─────────────────────────────────────────────────────────────────

    #[test]
    fn load_amp_config_returns_repository_value() {
        let state = Arc::new(SpyRepositoryState::new());
        let expected_id = uuid::Uuid::new_v4().to_string();
        let expected = AmpConfigDto {
            master_volume: 0.72,
            is_active: false,
            channels: Vec::new(),
            current_channel: expected_id.clone(),
            midi_bindings: Vec::new(),
        };
        *state.load_result.lock().unwrap() = Ok(Some(expected.clone()));

        let service = AmpConfigPersistenceService::new(Box::new(SpyRepository {
            state: Arc::clone(&state),
        }));

        let loaded = service.load_amp_config().expect("load should succeed");
        assert!(loaded.is_some());
        assert_eq!(loaded.unwrap().current_channel, expected_id);
    }

    #[test]
    fn load_amp_config_propagates_repository_error() {
        let state = Arc::new(SpyRepositoryState::new());
        *state.load_result.lock().unwrap() = Err("load failed".to_string());

        let service = AmpConfigPersistenceService::new(Box::new(SpyRepository {
            state: Arc::clone(&state),
        }));

        let err = service.load_amp_config().expect_err("load should fail");
        assert_eq!(err, "load failed");
    }

    #[test]
    fn persist_from_audio_service_saves_snapshot() {
        let state = Arc::new(SpyRepositoryState::new());
        let service = AmpConfigPersistenceService::new(Box::new(SpyRepository {
            state: Arc::clone(&state),
        }));

        let audio_service = make_audio_service();
        service
            .persist_from_audio_service(&audio_service)
            .expect("persist should succeed");

        let saved = state.wait_for_saved_count(1, Duration::from_secs(1));
        assert_eq!(saved.len(), 1);

        let cm = audio_service.channel_manager().lock().unwrap();
        assert_eq!(
            saved[0].current_channel,
            cm.current_channel_id().to_string()
        );
        assert!(!saved[0].is_active);
    }

    #[test]
    fn persist_from_audio_service_preserves_existing_midi_bindings() {
        let state = Arc::new(SpyRepositoryState::new());

        // Pre-load a config that already has a MIDI binding on disk.
        let binding = sample_binding();
        *state.load_result.lock().unwrap() = Ok(Some(AmpConfigDto {
            midi_bindings: vec![binding.clone()],
            ..AmpConfigDto::default()
        }));

        let service = AmpConfigPersistenceService::new(Box::new(SpyRepository {
            state: Arc::clone(&state),
        }));

        let audio_service = make_audio_service();
        service
            .persist_from_audio_service(&audio_service)
            .expect("persist should succeed");

        let saved = state.wait_for_saved_count(1, Duration::from_secs(1));
        assert_eq!(saved[0].midi_bindings.len(), 1);
        assert_eq!(saved[0].midi_bindings[0].cc_number, binding.cc_number);
    }

    #[test]
    fn persist_from_audio_service_enqueues_even_when_background_save_fails() {
        let state = Arc::new(SpyRepositoryState::new());
        *state.save_result.lock().unwrap() = Err("save failed".to_string());

        let service = AmpConfigPersistenceService::new(Box::new(SpyRepository {
            state: Arc::clone(&state),
        }));

        service
            .persist_from_audio_service(&make_audio_service())
            .expect("enqueue should succeed");

        let saved = state.wait_for_saved_count(1, Duration::from_secs(1));
        assert_eq!(saved.len(), 1);
    }

    #[test]
    fn persist_snapshot_keeps_only_newest_pending_snapshot() {
        let state = Arc::new(SpyRepositoryState::new());
        state.set_block_saves(true);

        let service = AmpConfigPersistenceService::new(Box::new(SpyRepository {
            state: Arc::clone(&state),
        }));

        let snapshot = |current_channel: String| AmpConfigDto {
            master_volume: 0.5,
            is_active: false,
            channels: Vec::new(),
            current_channel,
            midi_bindings: Vec::new(),
        };

        let id_1 = uuid::Uuid::new_v4().to_string();
        let id_2 = uuid::Uuid::new_v4().to_string();
        let id_3 = uuid::Uuid::new_v4().to_string();

        service
            .persist_snapshot(snapshot(id_1.clone()))
            .expect("first enqueue should succeed");
        state.wait_for_save_started_count(1, Duration::from_secs(1));

        service
            .persist_snapshot(snapshot(id_2.clone()))
            .expect("second enqueue should succeed");
        service
            .persist_snapshot(snapshot(id_3.clone()))
            .expect("third enqueue should succeed");

        state.set_block_saves(false);

        let saved = state.wait_for_saved_count(2, Duration::from_secs(1));

        assert_eq!(saved.len(), 2);
        assert_eq!(saved.first().unwrap().current_channel, id_1);
        assert_eq!(saved.last().unwrap().current_channel, id_3);
        assert!(!saved.iter().any(|cfg| cfg.current_channel == id_2));
    }

    #[test]
    fn persist_midi_bindings_enqueues_snapshot_with_supplied_bindings() {
        let state = Arc::new(SpyRepositoryState::new());
        let service = AmpConfigPersistenceService::new(Box::new(SpyRepository {
            state: Arc::clone(&state),
        }));

        let binding = sample_binding();
        service
            .persist_midi_bindings(vec![binding.clone()])
            .expect("persist_midi_bindings should succeed");

        let saved = state.wait_for_saved_count(1, Duration::from_secs(1));
        assert_eq!(saved.len(), 1);
        assert_eq!(saved[0].midi_bindings.len(), 1);
        assert_eq!(saved[0].midi_bindings[0].cc_number, binding.cc_number);
    }

    #[test]
    fn persist_midi_bindings_preserves_existing_amp_config_fields() {
        let state = Arc::new(SpyRepositoryState::new());

        // Pretend an amp config is already on disk.
        *state.load_result.lock().unwrap() = Ok(Some(AmpConfigDto {
            master_volume: 0.65,
            current_channel: "some-channel-id".to_string(),
            ..AmpConfigDto::default()
        }));

        let service = AmpConfigPersistenceService::new(Box::new(SpyRepository {
            state: Arc::clone(&state),
        }));

        service
            .persist_midi_bindings(vec![sample_binding()])
            .expect("persist_midi_bindings should succeed");

        let saved = state.wait_for_saved_count(1, Duration::from_secs(1));
        // Amp fields must survive the MIDI-only update.
        assert!((saved[0].master_volume - 0.65).abs() < 1e-6);
        assert_eq!(saved[0].current_channel, "some-channel-id");
        assert_eq!(saved[0].midi_bindings.len(), 1);
    }
}
