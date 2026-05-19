# ADR-005: Windows Audio Driver

**Status:** Accepted  
**Date:** May 2026  
**Context:** RustRiff - Cross-Platform Guitar Amplifier (Windows-specific decision)

## Problem

On Windows, CPAL can use either WASAPI (default) or ASIO (feature-gated) as the audio backend. For a virtual guitar amplifier, round-trip latency is the primary concern - guitarists need to hear their processed sound with minimal delay for the instrument to feel responsive.

## Alternatives Considered

| Alternative | Latency | CPAL Support | Build Complexity | Notes |
|---|---|---|---|---|
| **WASAPI Shared** | 20-50+ms | Yes (default) | None | Too high latency for real-time guitar playing. Unacceptable for our use case. |
| **WASAPI Exclusive** | 3-10ms | **No** | Low | Would be ideal - comparable latency to ASIO without extra dependencies. Not currently supported by CPAL. |
| **ASIO** | 3-10ms | Yes (feature-gated) | Requires LLVM/Clang + ASIO SDK | Best option currently available. Industry standard for professional audio on Windows. |
| DirectSound | 30-100ms | Deprecated | N/A | Legacy API, not suitable for low-latency audio. |

## Decision

Use **ASIO** via CPAL's `asio` feature flag on Windows.

- CPAL supports ASIO on Windows through the `cpal/asio` feature flag
- ASIO provides direct hardware access with consistently low latency (< 10ms achievable)
- WASAPI Exclusive mode would provide equivalent latency but is not supported by CPAL
- ASIO is the industry standard for professional audio on Windows
- The LLVM requirement for the ASIO SDK is accepted as a one-time build setup cost

**Note:** This decision applies to Windows only. On macOS, Core Audio natively provides low-latency performance. On Linux, ALSA/JACK provide adequate low-latency without additional dependencies.

## Consequences

- **Positive:** Sub-10ms round-trip latency on Windows, professional-grade audio performance, compatible with most audio interfaces
- **Negative:** Windows builds require the Steinberg ASIO SDK and LLVM/Clang toolchain, users must have ASIO drivers installed for their audio interface, budget interfaces without ASIO drivers may need ASIO4ALL as a workaround
- **Future consideration:** Monitor CPAL development for WASAPI Exclusive mode support. If added, migrating to WASAPI on Windows would eliminate the LLVM dependency and work with all Windows audio devices (including those without ASIO drivers). The `AudioHandlerTrait` abstraction is designed to facilitate this potential migration.
