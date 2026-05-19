# ADR-002: CPAL as Cross-Platform Audio Library

**Status:** Accepted  
**Date:** May 2026  
**Context:** RustRiff - Cross-Platform Guitar Amplifier

## Problem

We need a cross-platform audio I/O library that works on Windows, macOS, and Linux, provides low-latency audio capture and playback, and integrates well with the Rust ecosystem.

## Alternatives Considered

| Alternative | Pros | Cons |
|---|---|---|
| **CPAL** | De-facto standard in Rust, supports all 3 platforms, good documentation | Some platform-specific features behind feature flags|
| cubeb | Mozilla-backed, good cross-platform support | Smaller community, less Rust-idiomatic API |
| rodio | Built on CPAL, higher-level API | Inherits CPAL's limitations, less control over low-level audio |
| Direct platform APIs (WASAPI, Core Audio, ALSA) | Maximum control, lowest latency | Platform-specific code, not cross-platform, complex C FFI |

## Decision

Use **CPAL** as the cross-platform audio I/O library.

- CPAL abstracts over platform-native APIs: WASAPI/ASIO on Windows, Core Audio on macOS, ALSA/PulseAudio/JACK on Linux
- It is the most widely used audio crate in the Rust ecosystem
- Provides device enumeration, stream configuration, and callback-based audio I/O
- The trait-based API enables our `AudioHandlerTrait` abstraction for testability

## Consequences

- **Positive:** Single codebase for audio I/O across all platforms, access to ASIO on Windows, mockable through our abstraction layer, good community support
- **Negative:** Some advanced features require platform-specific code