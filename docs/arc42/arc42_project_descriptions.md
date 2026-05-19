---
date: May 2026
title: "![arc42](images/arc42-logo.png) RustRiff - Architecture Documentation"
---

# RustRiff - Architecture Documentation

**About arc42**

arc42, the template for documentation of software and system
architecture.

Template Version 9.0-EN. (based upon AsciiDoc version), July 2025

Created, maintained and © by Dr. Peter Hruschka, Dr. Gernot Starke and
contributors. See <https://arc42.org>.

# Introduction and Goals {#section-introduction-and-goals}

## Requirements Overview {#_requirements_overview}

**RustRiff** is a cross-platform desktop guitar amplifier built with Rust + Tauri and a React + TypeScript frontend. It models core amp controls (gain, tone stack, channel flow), an effect chain, and cabinet simulation processing.

**Key functional requirements:**

- Capture audio input from guitar/audio interface with minimal latency
- Process audio through amplifier models, effects, and cabinet IRs in real-time
- Output processed audio to speakers/headphones with minimal latency
- Provide a React-based UI for controlling amp parameters, channels, and effects
- Support multiple platforms: Windows, macOS, Linux
- Persist and restore amplifier configurations across sessions
- Real-time spectrum analysis and visualization

**Driving forces:**

- Low latency is critical for playability (target: < 10ms round-trip)
- Cross-platform compatibility (Windows, macOS, Linux)
- Clean architecture with separation of domain, services, infrastructure, and commands
- Testability through trait-based abstractions and mockable audio handlers

## Quality Goals {#_quality_goals}

| Quality Goal | Priority | Scenario |
|---|---|---|
| Low Latency | High | Round-trip audio latency (input to output) must be at or below 10ms for acceptable playing feel |
| Stability | High | Audio stream must not drop or glitch during extended playing sessions (1+ hours) |
| Cross-Platform | High | Application must build and run on Windows, macOS, and Linux without platform-specific code paths in the domain layer |
| Testability | Medium | Audio pipeline must be testable without real hardware via mockable `AudioHandlerTrait` |
| Developer Experience | Medium | Streamlined build process accross all target platforms|

## Stakeholders {#_stakeholders}

| Role/Name | Contact | Expectations |
|---|---|---|
| Guitarists (End Users) | - | Low latency, stable audio, good tone quality, cross-platform support |
| Development Team | - | Maintainable code, clean architecture, simple build process, testability |
| Audio Interface Manufacturers | - | Proper driver compatibility with their hardware on each platform |

# Architecture Constraints {#section-architecture-constraints}

| Constraint | Description |
|---|---|
| Language | Rust |
| Framework | Tauri v2 with React + React TypeScript frontend (with React Compiler) |
| Audio Library | CPAL 0.17 (Cross-Platform Audio Library) |
| Target Platforms | Windows, macOS, Linux |
| License | GPL-3.0-or-later |
| ASIO SDK (Windows only) | Steinberg ASIO SDK requires LLVM/Clang toolchain for building due to licensing restrictions in the SDK headers |

# Context and Scope {#section-context-and-scope}

## Business Context {#_business_context}

| Communication Partner | Input | Output |
|---|---|---|
| Guitarist | Plays guitar into audio interface | Hears processed amplifier sound |
| Audio Interface | Raw guitar signal (analog/digital) | Processed audio signal |
| React Frontend | User adjusts amp knobs, channels, presets, effects | Displays current settings, spectrum visualization |

## Technical Context {#_technical_context}

| Interface | Channel | Description |
|---|---|---|
| Audio Input | CPAL (platform-native API) | Captures raw audio from guitar interface |
| Audio Output | CPAL (platform-native API) | Sends processed audio to output device |
| UI Communication | Tauri IPC | React frontend communicates with Rust backend via Tauri commands |
| Config Persistence | JSON file | Amp configuration persisted to app config directory |

```
┌──────────────────────────────────────────────────────────┐
│                    React Frontend                         │
│              (Amp Controls / Spectrum Viz)                │
└────────────────────────┬─────────────────────────────────┘
                         │ Tauri IPC (commands/events)
┌────────────────────────▼─────────────────────────────────┐
│                    Rust Backend (src-tauri)               │
│                                                          │
│  ┌────────────┐  ┌────────────┐  ┌────────────────────┐  │
│  │  commands/ │  │ services/  │  │   domain/          │  │
│  │  (Tauri    │─▶│ audio_     │─▶│   audio_processor, │  │
│  │   invoke)  │  │ service    │  │   channel, effect, │  │
│  │            │  │            │  │   tone_stack       │  │
│  └────────────┘  └─────┬──────┘  └────────────────────┘  │
│                        │                                  │
│                  ┌─────▼──────┐                           │
│                  │infrastruct-│                           │
│                  │ure/        │                           │
│                  │audio_handler│                          │
│                  └─────┬──────┘                           │
└────────────────────────┼──────────────────────────────────┘
                         │
┌────────────────────────▼──────────────────────────────────┐
│                    CPAL (0.17)                             │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐                │
│  │ Windows  │  │  macOS   │  │  Linux   │                │
│  │ WASAPI/  │  │  Core    │  │  ALSA/   │                │
│  │ ASIO     │  │  Audio   │  │  Pulse/  │                │
│  │          │  │          │  │  JACK    │                │
│  └──────────┘  └──────────┘  └──────────┘                │
└────────────────────────┬──────────────────────────────────┘
                         │
┌────────────────────────▼──────────────────────────────────┐
│              Audio Interface Hardware                      │
│         (USB/Thunderbolt - Focusrite, etc.)                │
└────────────────────────────────────────────────────────────┘
```

# Solution Strategy {#section-solution-strategy}

The architecture follows a clean layered approach:

- **Domain layer** (`domain/`) - Pure Rust models and traits. Platform-agnostic. Contains `AudioProcessor`, `Channel`, `Effect`, `ToneStack`, and DTOs...
- **Services layer** (`services/`) - Application logic. `AudioService` orchestrates the audio pipeline, DSP processors implement the effect chain, resampling handles rate mismatches.
- **Infrastructure layer** (`infrastructure/`) - External adapters. `AudioHandler` wraps CPAL for audio I/O. File loading and JSON persistence live here.
- **Commands layer** (`commands/`) - Tauri invoke handlers that expose functionality to the React frontend.

Audio processing uses a lock-free ring buffer architecture: CPAL callbacks push/pop samples to/from ring buffers, while a dedicated DSP worker thread processes samples through the effect chain. This keeps the audio callbacks allocation-free and lock-free.

Cross-platform audio is handled by CPAL, which delegates to platform-native APIs (WASAPI/ASIO on Windows, Core Audio on macOS, ALSA/PulseAudio/JACK on Linux). Platform-specific decisions (such as the Windows audio driver choice) are documented as individual ADRs linked in Section 9.

# Building Block View {#section-building-block-view}

## Whitebox Overall System {#_whitebox_overall_system}

**Motivation:** The system follows a clean architecture with four main layers: commands (Tauri handlers), services (application logic), domain (core models), and infrastructure (external adapters).

### Contained Building Blocks

| Name | Responsibility | Location |
|---|---|---|
| `commands/` | Tauri invoke handlers exposing functionality to the frontend | `src-tauri/src/commands/` |
| `services/` | Application services: `AudioService`, `DeviceService`, DSP processors, analyzers | `src-tauri/src/services/` |
| `domain/` | Core models: `AudioProcessor`, `Channel`, `Effect`, `ToneStack`, DTOs | `src-tauri/src/domain/` |
| `infrastructure/` | External adapters: `AudioHandler` (CPAL), file loading, JSON persistence | `src-tauri/src/infrastructure/` |
| React Frontend | UI for amp controls, spectrum visualization, preset management | `src/` |

### Important Interfaces

| Interface | Description |
|---|---|
| `AudioHandlerTrait` | Abstraction over CPAL audio I/O. Defines `build_input_stream`, `build_output_stream`, device/config access. Mockable via `mockall` for testing. |
| Tauri Commands | Async/sync handlers invoked from frontend via `invoke()`. Grouped by concern: `default_controls`, `effect_commands`, `settings`, `analyzer`, `latency_testing`, etc. |
| `AudioProcessor` (domain trait) | Implemented by all DSP effects. Defines `process(sample: f32) -> f32` and `process_if_active`. |
| Ring Buffer (ringbuf) | Lock-free SPSC ring buffer between CPAL callbacks and the DSP worker thread. `HeapProd<f32>` / `HeapCons<f32>`. |

### Audio Handler (Infrastructure)

**Purpose/Responsibility:** Wraps CPAL devices and stream configurations. Builds input/output streams that push/pull samples through ring buffers. Implements `AudioHandlerTrait` for testability.

**Interface(s):**
- `build_input_stream(prod: HeapProd<f32>) -> Box<dyn PlayableStream>`
- `build_output_stream(cons: HeapCons<f32>) -> Box<dyn PlayableStream>`
- `create_ringbuffer(size: usize) -> (HeapProd<f32>, HeapCons<f32>)`

**Quality/Performance Characteristics:**
- Callbacks are lock-free and allocation-free
- Samples that cannot be pushed to a full ring buffer are silently dropped
- Empty output slots are zero-filled (silence)

### Audio Service (Services)

**Purpose/Responsibility:** Orchestrates the full audio pipeline lifecycle. Manages loopback thread, resampling, DSP chain execution, channel switching, and buffer size configuration.

**Key behaviors:**
- Automatic resampling policy selection (`PreDsp`, `PostDsp`, or `Bypass`) based on input/output rate comparison
- DSP chain: Gain -> ToneStack -> EffectChain -> Volume -> MasterVolume -> (SpectrumTap)
- Dedicated background thread for DSP processing, separate from CPAL callback threads
- Hot-swappable input/output devices with automatic loopback restart

### DSP Processors (Services)

| Processor | Responsibility |
|---|---|
| `GainProcessor` | Applies gain/volume amplification via atomic float parameters |
| `ToneStackProcessor` | Baxandall-style 3-band EQ (bass, middle, treble) |
| `Resampler` (Rubato) | Sample rate conversion when input/output rates differ |
| `HcDistortion` | Hard-clipping distortion effect |
| `ScDistortion` | Soft-clipping distortion with smoothing |
| `Cabinet` | Cabinet impulse response convolution |
| `Delay` | Time-based delay effect |

# Runtime View {#section-runtime-view}

## Audio Processing Loop

The critical runtime scenario:

1. CPAL input callback fires with a buffer of captured samples
2. Each sample is pushed into the input ring buffer (`HeapProd<f32>`)
3. DSP worker thread pops samples from the input ring buffer
4. `ResamplePolicy::process()` applies resampling at the correct point:
   - `Bypass` (inputSampleRate == outputSampleRate): calls `run_dsp` directly
   - `PreDsp` (inputSampleRate > outputSampleRate): downsamples first, then `run_dsp` (DSP runs at lower rate)
   - `PostDsp` (inputSampleRate < outputSampleRate): calls `run_dsp` first, then upsamples
5. `run_dsp` executes: Gain -> ToneStack -> EffectChain -> Volume -> MasterVolume -> SpectrumTap
6. Processed samples are pushed into the output ring buffer
7. CPAL output callback fires, drains samples from output ring buffer (`HeapCons<f32>`)
8. Missing samples are zero-filled (silence)

**Notable aspects:**
- CPAL callbacks run on threads managed by the OS audio subsystem
- DSP worker runs on a dedicated `std::thread`
- Communication between callbacks and worker is via lock-free ring buffers (ringbuf crate)
- Parameter updates from UI arrive via Tauri commands on the main thread and are applied through `Arc<AtomicF32>` - no locks in the audio path

## Device Initialization (lib.rs startup)

1. `run()` is called, tracing is initialized
2. CPAL `default_host()` is obtained (platform-dependent: WASAPI on Windows, Core Audio on macOS, ALSA on Linux)
3. Default input/output devices are enumerated
4. Channel count is normalized (0 -> 2, >2 -> 2, 1 stays 1)
5. `AudioService` is created with devices and default stream configs
6. `AudioService` and `DeviceService` are managed via Tauri's state system
7. Persisted amp config is loaded from JSON and applied
8. Tauri command handlers are registered
9. App runs

## Parameter Update from UI

1. User adjusts a knob in the React frontend
2. Tauri command is invoked (e.g., `set_gain`, `set_bass`)
3. Command handler retrieves `AudioService` from Tauri state
4. Parameter is updated via `Arc<AtomicF32::store()` with `Ordering::Relaxed`
5. Next DSP iteration reads the new value atomically
6. No audio glitches occur during the update

# Deployment View {#section-deployment-view}

## Infrastructure Level 1

**Motivation:** The application runs as a single desktop process on Windows, macOS, or Linux, interfacing with the platform's native audio stack via CPAL.

```
┌──────────────────────────────────────────────────────────────┐
│                    Target OS (Win / macOS / Linux)            │
│                                                               │
│  ┌────────────────────────────────────────────────────────┐  │
│  │                  RustRiff Desktop App                   │  │
│  │                                                        │  │
│  │  ┌───────────────┐    ┌─────────────────────────────┐  │  │
│  │  │  Tauri Shell  │    │       Rust Backend           │  │  │
│  │  │  (WebView2 /  │◄──►│  ┌─────────────────────────┐ │  │  │
│  │  │   WebKit)     │    │  │  commands/ (Tauri invoke)│ │  │  │
│  │  │  + React UI   │    │  └────────────┬────────────┘ │  │  │
│  │  └───────────────┘    │               │              │  │  │
│  │                       │  ┌────────────▼────────────┐ │  │  │
│  │                       │  │  services/              │ │  │  │
│  │                       │  │  AudioService           │ │  │  │
│  │                       │  │  DSP Processors         │ │  │  │
│  │                       │  └────────────┬────────────┘ │  │  │
│  │                       │               │              │  │  │
│  │                       │  ┌────────────▼────────────┐ │  │  │
│  │                       │  │  infrastructure/        │ │  │  │
│  │                       │  │  AudioHandler (CPAL)    │ │  │  │
│  │                       │  └────────────┬────────────┘ │  │  │
│  │                       └───────────────┼──────────────┘  │  │
│  └───────────────────────────────────────┼─────────────────┘  │
│                                          │                     │
│  ┌───────────────────────────────────────▼─────────────────┐  │
│  │                    CPAL 0.17                             │  │
│  │  ┌────────────┐  ┌──────────────┐  ┌────────────────┐  │  │
│  │  │ Windows:   │  │ macOS:       │  │ Linux:         │  │  │
│  │  │ WASAPI or  │  │ Core Audio   │  │ ALSA/Pulse/JACK│  │  │
│  │  │ ASIO       │  │              │  │                │  │  │
│  │  └────────────┘  └──────────────┘  └────────────────┘  │  │
│  └───────────────────────────────────────┬─────────────────┘  │
│                                          │                     │
│  ┌───────────────────────────────────────▼─────────────────┐  │
│  │              Audio Interface Hardware                    │  │
│  │         (USB/Thunderbolt - Focusrite, etc.)              │  │
│  └──────────────────────────────────────────────────────────┘  │
└────────────────────────────────────────────────────────────────┘
```

**Mapping of Building Blocks to Infrastructure:**

| Building Block | Deployment Target |
|---|---|
| React Frontend | Tauri WebView2 (Windows) / WebKit (macOS/Linux) |
| Commands Layer | Native Rust process, invoked via Tauri IPC |
| Services Layer | Native Rust process, DSP runs on dedicated background thread |
| Infrastructure (AudioHandler) | Links against CPAL, which delegates to platform audio APIs |
| Domain Models | Pure Rust, platform-agnostic |

# Cross-cutting Concepts {#section-concepts}

## Real-Time Audio Safety

All code executed within the CPAL audio callback path must be real-time safe:
- No heap allocations
- No mutex locks
- No file I/O
- No blocking syscalls

The design achieves this by:
- Using lock-free ring buffers (`ringbuf` crate) between callbacks and the DSP worker
- The DSP worker thread does the actual processing, not the CPAL callback
- CPAL callbacks only push/pop samples from the ring buffer

## Thread Safety for Parameter Updates

DSP parameters are updated from the UI thread (via Tauri commands) but read from the DSP worker thread. The design uses:
- `Arc<AtomicF32>` for all real-time parameters (gain, volume, tone values)
- `Ordering::Relaxed` for atomic operations (sufficient for audio parameter updates)
- `Arc<Mutex<Vec<Effect>>>` for effect chain mutations (only modified outside the audio path)

## Resampling Strategy

When input and output sample rates differ, `ResamplePolicy` automatically selects the optimal strategy:

| Condition | Policy | Rationale |
|---|---|---|
| input == output | `Bypass` | No resampling needed, zero overhead |
| input > output | `PreDsp` | Downsample before DSP; DSP runs at the lower output rate (cheaper) |
| input < output | `PostDsp` | Upsample after DSP; DSP runs at the lower input rate (cheaper) |

Uses the `rubato` crate for high-quality sample rate conversion.

## Audio Backend Abstraction

The `AudioHandlerTrait` abstracts over CPAL's audio I/O:

```rust
pub trait AudioHandlerTrait: Send + Sync {
    fn build_input_stream(&self, prod: HeapProd<f32>) -> Box<dyn PlayableStream>;
    fn build_output_stream(&self, cons: HeapCons<f32>) -> Box<dyn PlayableStream>;
    fn input_device(&self) -> &Device;
    fn output_device(&self) -> &Device;
    fn input_config(&self) -> &StreamConfig;
    fn output_config(&self) -> &StreamConfig;
    fn input_sample_rate(&self) -> u32;
    fn output_sample_rate(&self) -> u32;
}
```

This abstraction enables:
- Unit testing with `MockAudioHandlerTrait` (generated by `mockall`)
- Future backend swaps (e.g., WASAPI Exclusive on Windows) without changing services or domain
- Hot-swapping devices at runtime via `set_audio_handler()`

# Architecture Decisions {#section-design-decisions}

Architecture decisions are documented as individual Architecture Decision Records (ADRs). Each ADR is a single-page markdown file that captures the context, alternatives, decision, and consequences for one specific decision. Section 9 serves as an index linking to all ADRs.

| ADR | Title | Status | Summary |
|---|---|---|---|
| [ADR-001](./adr/ADR-001-Tauri-React-Framework.md) | Tauri + React as Application Framework | Accepted | Use Tauri v2 with React + TypeScript for the desktop application shell and UI |
| [ADR-002](./adr/ADR-002-CPAL-Audio-Library.md) | CPAL as Cross-Platform Audio Library | Accepted | Use CPAL 0.17 as the audio I/O abstraction across Windows, macOS, and Linux |
| [ADR-004](./adr/ADR-004-JSON-Config-Persistence.md) | JSON-Based Configuration Persistence | Accepted | Persist amplifier configuration as JSON in the app config directory |
| [ADR-003](./adr/ADR-003-Resampling-Strategy.md) | Automatic Resampling Policy | Accepted | Use `ResamplePolicy` with Rubato to handle mismatched input/output sample rates, optimizing DSP execution rate |
| [ADR-005](./adr/ADR-ADR-005-Windows-Audio-Driver-ASIO-vs-WASAPI.md) | Windows Audio Driver: ASIO over WASAPI | Accepted | Use ASIO via CPAL's feature flag on Windows for low-latency audio, since CPAL does not support WASAPI Exclusive mode |

# Quality Requirements {#section-quality-scenarios}

## Quality Requirements Overview

| Category | Requirement |
|---|---|
| Latency | Round-trip audio latency < 10ms at 128 sample buffer size, 48kHz |
| Stability | No audio dropouts or glitches during 1+ hour sessions |
| CPU Usage | Audio processing should consume as little CPU as possible withouth impacting latency. |
| Cross-Platform | Must build and run on Windows 10/11, macOS 12+, and major Linux distributions |
| Build Simplicity | After initial platform setup, `npm run tauri build` should work with as little extra steps as possible. |

## Quality Scenarios

### Scenario 1: Low Latency Audio

| Field | Value |
|---|---|
| Scenario ID | 1 |
| Name | Low Latency Audio Processing |
| Source | Guitarist playing |
| Stimulus | Guitar signal enters audio interface |
| Environment | Windows (ASIO) / macOS (Core Audio) / Linux (ALSA), 128 sample buffer, 48kHz |
| Artifact | AudioHandler + AudioService + DSP Engine |
| Response | Processed audio exits audio interface |
| Response Measure | Round-trip latency < 10ms (measured with loopback test via `measure_round_trip_latency` command) |

### Scenario 2: Stable Audio Stream

| Field | Value |
|---|---|
| Scenario ID | 2 |
| Name | Stable Audio During Extended Use |
| Source | Continuous audio stream |
| Stimulus | Application runs for 1+ hours |
| Environment | Normal desktop usage, other applications running |
| Artifact | AudioService + AudioHandler |
| Response | Audio stream continues without dropouts |
| Response Measure | Zero buffer underruns/overruns in 1 hour (logged via error callbacks) |

### Scenario 3: Parameter Update Without Glitch

| Field | Value |
|---|---|
| Scenario ID | 3 |
| Name | Glitch-Free Parameter Updates |
| Source | User adjusting UI controls |
| Stimulus | Rapid parameter changes (e.g., turning gain knob) |
| Environment | Audio stream active |
| Artifact | DSP Engine + AtomicF32 parameters |
| Response | Parameters update smoothly without audio artifacts |
| Response Measure | No clicks, pops, or dropouts audible during parameter changes |

### Scenario 4: Cross-Platform Build

| Field | Value |
|---|---|
| Scenario ID | 4 |
| Name | Successful Build on All Platforms |
| Source | Developer running build command |
| Stimulus | `npm run tauri build` executed |
| Environment | Windows (with LLVM), macOS, Linux |
| Artifact | CI/CD pipeline |
| Response | Application binary produced |
| Response Measure | Build succeeds on all three platforms without code changes |

# Risks and Technical Debts {#section-technical-risks}

| Risk/Debt | Priority | Platform | Description | Mitigation |
|---|---|---|---|---|
| LLVM Build Dependency (ASIO) | High | Windows | ASIO SDK requires LLVM/Clang toolchain, adding complexity to developer setup and CI | Document setup clearly; consider pre-building ASIO bindings; monitor CPAL for WASAPI Exclusive support (see [ADR-005](./adr/ADR-005-Windows-Audio-Driver-ASIO-vs-WASAPI.md)) |
| CPAL ASIO Feature Maturity | Medium | Windows | ASIO support in CPAL is behind a feature flag and may have less testing than the default WASAPI path | Test thoroughly on multiple interfaces; consider contributing to CPAL ASIO support |
| No WASAPI Exclusive | Medium | Windows | WASAPI Exclusive would eliminate LLVM dependency and work with all Windows audio devices | Track CPAL issue tracker; `AudioHandlerTrait` abstraction eases future migration (see [ADR-005](./adr/ADR-005-Windows-Audio-Driver-ASIO-vs-WASAPI.md)) |
| Real-time Safety Violations | High | All | Accidental allocation or locking in audio callback causes dropouts | Code review, testing under load, ring buffer architecture isolates DSP from callbacks (see [ADR-005](./adr/ADR-005-Lock-Free-Ring-Buffer-Architecture.md)) |
| Sample Rate Mismatch | Low | All | Input/output devices may have different sample rates | `ResamplePolicy` handles this automatically with Rubato resampler (see [ADR-003](./adr/ADR-003-Resampling-Strategy.md)) |
| Multi-Channel Devices | Low | All | Devices with >2 channels may cause unexpected behavior | Channel normalization in `lib.rs` caps at 2 channels (stereo) |

# Glossary {#section-glossary}

| Term | Definition |
|---|---|
| ADR | Architecture Decision Record - A short document capturing the context, alternatives, decision, and consequences for a single architectural decision |
| ASIO | Audio Stream Input/Output - Steinberg's proprietary audio driver protocol providing low-latency direct hardware access on Windows |
| WASAPI | Windows Audio Session API - Microsoft's audio API for Windows Vista and later. Has Shared (higher latency) and Exclusive (low latency) modes |
| WASAPI Shared | Default Windows audio mode where multiple applications share the audio device. Latency typically 20-50ms |
| WASAPI Exclusive | Mode where one application has exclusive access to the audio device. Latency comparable to ASIO (3-10ms). Not currently supported by CPAL |
| Core Audio | macOS native audio API. Inherently low-latency, no equivalent to the WASAPI Shared/Exclusive distinction |
| ALSA | Advanced Linux Sound Architecture - Linux kernel-level audio API. Can provide low-latency direct hardware access |
| CPAL | Cross-Platform Audio Library - The default audio I/O crate in the Rust ecosystem (v0.17 in this project) |
| Latency | Time delay between audio input and output. Critical for real-time instrument playing |
| Buffer Size | Number of audio samples processed per callback. Smaller = lower latency but higher CPU usage |
| Sample Rate | Number of audio samples per second (e.g., 44100 Hz, 48000 Hz) |
| Round-trip Latency | Total delay from input entering the interface to processed output leaving the interface |
| DSP | Digital Signal Processing - The algorithms that simulate amplifiers, effects, etc. |
| ASIO4ALL | Universal ASIO driver wrapper that provides ASIO interface for devices without native ASIO drivers |
| LLVM | Low Level Virtual Machine - Compiler infrastructure required to build the ASIO SDK bindings on Windows |
| Tauri | Framework for building desktop applications with web frontend and Rust backend (v2 in this project) |
| Ring Buffer | Lock-free circular buffer for wait-free data transfer between threads. Used via the `ringbuf` crate |
| SPSC | Single-Producer Single-Consumer - ring buffer access pattern matching the audio callback -> worker thread model |
| ResamplePolicy | Strategy for handling mismatched input/output sample rates: `Bypass`, `PreDsp`, or `PostDsp` |
| IR | Impulse Response - A recording of a speaker cabinet's acoustic signature, used for convolution-based cabinet simulation |
| RustRiff | The project name - a cross-platform virtual guitar amplifier built with Rust + Tauri + React |
