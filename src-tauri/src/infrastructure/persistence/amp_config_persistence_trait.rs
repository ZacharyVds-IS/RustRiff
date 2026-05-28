use crate::domain::dto::amp_config_dto::AmpConfigDto;
use crate::domain::dto::midi_mapping_dto::MidiMappingDto;

/// Backend abstraction for amplifier configuration persistence.
///
/// This trait isolates the rest of the application from the concrete storage
/// mechanism.
///
/// Implementations are expected to:
/// - return `Ok(None)` when no persisted config exists yet,
/// - return `Err(String)` when the data exists but cannot be read or parsed,
/// - persist a full amplifier snapshot on `save`.
pub trait AmpConfigPersistence: Send + Sync {
    /// Loads the most recently persisted amplifier configuration.
    ///
    /// Returns `Ok(None)` when no stored configuration is available yet.
    fn load(&self) -> Result<Option<AmpConfigDto>, String>;

    /// Persists the supplied amplifier configuration snapshot.
    ///
    /// Implementations should overwrite the previous snapshot atomically from
    /// the application's point of view: after a successful return, the new
    /// state is considered the canonical persisted config.
    fn save(&self, config: &AmpConfigDto) -> Result<(), String>;

    /// Loads the current snapshot, replaces its `midi_bindings`, and saves.
    ///
    /// The default implementation is correct for all single-file repositories:
    /// it reads whatever is on disk (falling back to [`AmpConfigDto::default`]
    /// when the file does not exist yet), splices in the new bindings, and
    /// delegates to [`save`].
    ///
    /// [`save`]: AmpConfigPersistence::save
    fn save_midi_bindings(&self, bindings: Vec<MidiMappingDto>) -> Result<(), String> {
        let mut config = self.load()?.unwrap_or_default();
        config.midi_bindings = bindings;
        self.save(&config)
    }
}
