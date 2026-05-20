# ADR-001: Tauri + React as Application Framework

**Status:** Accepted  
**Date:** May 2026  
**Context:** RustRiff - Cross-Platform Guitar Amplifier

## Problem
We need a framework for building a cross-platform desktop applcation which allows us to use simple frontend frameworks so our main focus can shift towards DSP.

## Alternatives Considered

| Alternative | Pros | Cons |
|---|---|---|
| **Tauri + React** | Small binary size, Rust backend, web frontend, good ecosystem | Requires WebView2 on Windows, relatively young ecosystem |
| Electron + React | Mature ecosystem, large community | Large binary size, high memory usage, Chromium bundled |
| egui / iced (pure Rust GUI) | Pure Rust, no web dependencies | Less mature UI component ecosystem, steeper learning curve for complex UIs |
| Flutter Desktop | Good cross-platform support, rich widgets | Dart language, larger binary, less Rust integration |

## Decision

Use **Tauri v2** with a **React + TypeScript** frontend.

- Tauri provides a lightweight desktop shell with native OS webviews (WebView2 on Windows, WebKit on macOS/Linux)
- Rust backend handles all audio processing, DSP, and system-level operations
- React + TypeScript provides a rich, component-based UI with strong typing
- Tauri's IPC mechanism (commands/events) cleanly separates frontend from backend

## Consequences

- **Positive:** Small binary size compared to Electron, Rust performance for audio processing, familiar web technologies for UI development, cross-platform from a single codebase
- **Negative:** WebView2 dependency on Windows 10/11 (not available on Windows 7/8), Tauri ecosystem is younger than Electron's, some web APIs may behave differently across platforms
- **Risk:** Tauri v2 API changes may require migration effort in the future
