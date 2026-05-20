# Storing data

RustRiff contains a growing number of user-controlled settings. Without
persistence, every restart would reset channels, amp settings, and effects back
to defaults.

The backend therefore persists amplifier state as a JSON snapshot and restores
it again during startup.

## What is currently persisted?

The persisted snapshot represents the amplifier configuration, not the entire
runtime state of the application.

The JSON file currently stores:

| Persisted area | Notes |
|----------------|-------|
| Channels | Includes channel ids and names |
| Amp settings | Gain, channel volume, master volume, and tone stack |
| Effect settings | Effect chain entries and their parameters |
| Active channel | The currently selected channel id |

## What is intentionally **not** persisted?

Loopback runtime state (`is_active`) is deliberately not written to disk.

That means:

- the frontend/runtime DTO still contains `is_active`,
- but the persistence-specific JSON schema omits it,
- and loading from disk always restores the amplifier with loopback **off**.

This is intentional so the application does not start processing audio
automatically after a restart.

## Backend architecture

Persistence is split into three layers:

### 1. Persistence trait

`src-tauri/src/infrastructure/persistence/amp_config_persistence_trait.rs`

This trait defines the persistence contract:

- `load() -> Result<Option<AmpConfigDto>, String>`
- `save(&AmpConfigDto) -> Result<(), String>`

The rest of the backend depends on this trait, not on JSON directly.

### 2. JSON repository

`src-tauri/src/infrastructure/persistence/json_amp_config_repository.rs`

The current implementation uses [serde](https://serde.rs/) and `serde_json` to:

- serialize the amplifier snapshot,
- write it to a JSON file,
- load it back on startup,
- return `Ok(None)` if no file exists yet.

The repository uses a persistence-only struct (`PersistedAmpConfig`) so the file
format can differ from the runtime DTO when necessary.

### 3. Persistence service

`src-tauri/src/services/amp_config_service.rs`

Commands do not talk to the repository directly. Instead they use
`AmpConfigPersistenceService`, which:

- loads the initial config during startup,
- captures snapshots from `AudioService`,
- delegates save/load to the configured repository.

This keeps command handlers thin and makes it easier to swap the storage backend
later.

## Save lifecycle

Most mutating backend commands call the helper in:

`src-tauri/src/commands/helpers.rs`

That helper snapshots the current `AudioService` state and persists it after the
command has successfully updated the in-memory model.

Examples include:

- changing gain,
- changing master volume,
- changing tone controls,
- adding/removing channels,
- toggling effect parameters.

Persistence is currently **best effort**:

- the in-memory change is applied first,
- a save is attempted afterwards,
- and save errors are logged instead of rolling back the change.

## Load lifecycle

During backend startup (`src-tauri/src/lib.rs`):

1. the JSON repository is created,
2. the persistence service attempts to load a saved config,
3. if a config exists, `AudioService::apply_amp_config(...)` restores it,
4. otherwise the app starts with the default channel/config.

`AudioService::apply_amp_config(...)` restores:

- channels,
- amp settings,
- effect chains,
- current channel selection,
- next channel id.

If the saved config is incomplete or empty, the service falls back to a default
channel so the application remains usable.

## Current storage location

At the moment the backend resolves the config file using the process working
directory and stores it as:

`amp-config.json`

This is important during development because saving inside the workspace may be
observed by dev tooling. If the storage location changes later (for example to a
dedicated app config directory), this page should be updated to match the real
implementation.

## Why the trait matters

Using a trait-based persistence boundary means the rest of the system is not
tightly coupled to JSON files.

When RustRiff grows, the JSON repository can be replaced by another
implementation such as:

- SQLite,
- a more structured local database,
- or even a synced/cloud-backed storage service,

without having to rewrite every command handler.


