# ADR-004: JSON-Based Configuration Persistence

**Status:** Accepted  
**Date:** May 2026  
**Context:** RustRiff - Cross-Platform Guitar Amplifier

## Problem

Amplifier configurations (channels, gain, tone stack, effects, master volume) need to be persisted between application sessions so users don't lose their settings.

## Alternatives Considered

| Alternative | Pros | Cons |
|---|---|---|
| **JSON file** | Human-readable, easy to debug, serde support built-in, cross-platform | No built-in versioning/migration, manual schema evolution |
| SQLite database | Structured, queryable, supports migrations | Overkill for a single config object, adds dependency, harder to inspect/edit manually |

## Decision

Use **JSON file persistence** stored in the platform's app config directory. But with abstraction for potentialy switching out to a database once the project's scope grows.

- `JsonFileAmpConfigRepository` in `infrastructure/persistence/` handles read/write
- Configuration is saved as `amp-config.json` in the Tauri app config/data directory
- `AmpConfigPersistenceService` in `services/` orchestrates load/save operations
- Configuration is loaded on application startup and applied to `AudioService`
- DTOs use `serde` for serialization and `ts-rs` for TypeScript type generation

## Consequences

- **Positive:** Simple implementation, human-readable config files, easy to backup/share presets, no additional database dependencies
- **Negative:** No built-in migration system for schema changes, concurrent writes not handled (single-user application mitigates this)
- **Risk:** Schema changes between versions may break loading of old config files. Mitigated by backward-compatible deserialization (e.g., tone value normalization from 0-100 to 0.0-1.0 range)
