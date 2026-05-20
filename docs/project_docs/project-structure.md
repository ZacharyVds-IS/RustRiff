# Project Structure

This repository is split into frontend, backend, and docs concerns:

## Top-level folders

- `src/`: React UI, routes, components, hooks, and frontend state.
- `src-tauri/src/`: Rust backend (commands, domain, infrastructure, services).
- `docs/`: VitePress content and generated API docs in `docs/public`.

## Backend module responsibilities

- `commands/`: Tauri command handlers exposed to the frontend.
- `domain/`: Core data types and business rules.
- `infrastructure/`: Integration points such as audio handling.
- `services/`: Higher-level orchestration and processing services.

## Frontend responsibilities

- `screens/`: Page-level UI.
- `components/`: Reusable visual and control components.
- `hooks/`: Integration hooks for Tauri/device interactions.
- `state/`: Shared application state.

