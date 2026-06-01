# MIDI support

Musical Instrument Digital Interface is a universal standard in which musical devices, computers and software can
communicate with each other.
MIDI does not send audio, it sends control data: which note is pressed, how hard, and controller movements.

## MIDI in RustRiff

RustRiff relies on MIDI for controlling effects in real time. For example, a wah pedal can be mapped to a hardware
expression pedal, or a distortion can be toggled on/off via a footswitch.

This enables hands-free mid-song changes, a common requirement for live guitarists.

## Data types

- **Channels** (1–16): Isolate control messages so multiple devices can share the same MIDI bus without
  cross-interference.
- **CC (Control Change)**: A number (0–127) that identifies which knob, switch, or pedal is being moved.
- **Data value** (0–127): The actual value of the controller. RustRiff treats values as continuous for "expression
  pedal" bindings (e.g. Wah sweep, delay level). All other bindings (e.g. ToggleBypass) are treated as discrete
  toggles — any value >= 64 toggles the state.

## Architecture
```
┌──────────────────────────────────────────────────────────────────┐
│                    Frontend (React/TypeScript)                   │
│                                                                  │
│   MidiSection.tsx     MidiConfigScreen.tsx     MidiBindingDialog │
│        │                     │                        │          │
│        └─────────────────────┼────────────────────────┘          │
│                              │  Tauri invoke (IPC)               │
├──────────────────────────────┼───────────────────────────────────┤
│                           Backend (Rust)                         │
│  ┌───────────────────────────┴────────────────────────────────┐  │
│  │  commands/midi.rs  (Tauri command handlers)                │  │
│  │  - get_midi_inputs                                         │  │
│  │  - connect_midi_device                                     │  │
│  │  - disconnect_midi_device                                  │  │
│  │  - register_midi_binding                                   │  │
│  │  - get_midi_bindings                                       │  │
│  │  - remove_midi_binding                                     │  │
│  └───────────────────────────┬────────────────────────────────┘  │
│                              │                                   │
│  ┌───────────────────────────┴────────────────────────────────┐  │
│  │  services/midi_service.rs  (Orchestration)                 │  │
│  │  - Manages bindings HashMap<(u8,u8), (Uuid, Param)>        │  │
│  │  - process_incoming_message() dispatches CC to effect      │  │
│  └───────────────────────────┬────────────────────────────────┘  │
│                              │                                   │
│  ┌───────────────────────────┴────────────────────────────────┐  │
│  │  infrastructure/midi_handler.rs  (midir library)           │  │
│  │  infrastructure/midi_parser.rs   (byte parsing)            │  │
│  └───────────────────────────┬────────────────────────────────┘  │
│                              │                                   │
│  ┌───────────────────────────┴────────────────────────────────┐  │
│  │  domain/                                                   │  │
│  │  - midi_target_parameter.rs  (MidiTargetParameter enum)    │  │
│  │  - dto/midi_mapping_dto.rs   (MidiMappingDto)              │  │
│  │  - dto/MidiDeviceDto.rs      (MidiDeviceDto)               │  │
│  │  - channel_manager.rs        (effect CRUD)                 │  │
│  └────────────────────────────────────────────────────────────┘  │
└──────────────────────────────────────────────────────────────────┘
```

## Mappable parameters

| Parameter             | Type       | Effect(s)        | Description                       |
|-----------------------|------------|------------------|-----------------------------------|
| `ToggleBypass`        | toggle     | All              | Flip the effect on/off            |
| `WahPedalPosition`    | continuous | Wah              | Filter sweep position (0.0 – 1.0) |
| `DelayTime`           | continuous | Delay            | Delay time in ms (0 – 2000)       |
| `DelayLevel`          | continuous | Delay            | Wet/dry mix level (0.0 – 1.0)     |
| `DistortionLevel`     | continuous | HC/SC Distortion | Output level/gain (1.0 – 2.0)     |
| `DistortionThreshold` | continuous | HC/SC Distortion | Clipping threshold (0.001 – 1.0)  |

- **Toggle** parameters ignore the MIDI value and simply invert the current bypass state.
- **Continuous** parameters map the 0–127 MIDI range onto the effect's full range.

## Setting up MIDI bindings

### 1. Settings panel — `MidiSection.tsx`

Located in the Settings screen, this component:

- Lists available MIDI input devices via `getMidiInputs()`
- Shows a **Connect** / **Disconnect** button for each device
- Provides a **Scan** button to refresh the device list
- A **Configure Advanced Mappings** button navigates to `/midi-mappings`

### 2. MIDI configuration screen — `MidiConfigScreen.tsx` (`/midi-mappings`)

Full table-based view of all active bindings with columns:

- **Port Line** — MIDI channel
- **CC Event ID** — CC number
- **Target DSP Module** — Effect name + kind
- **Intersect Parameter** — Bound parameter
- **Hardware Runtime Target Reference** — Effect UUID
- **Actions** — Delete button

From this screen you can also create new bindings via a form at the top and use **MIDI Learn** (background listener on
`midi-raw-sniff` events).

### 3. Per-pedal dialog — `MidiBindingDialog.tsx`

Opened from individual effect pedals in the chain. A two-step wizard:

1. **StepParameterSelection** — Choose the parameter to map
2. **StepCcAssignment** — Use MIDI Learn (auto-capture) or manually enter channel/CC

Includes an "Active Bindings on this Pedal" section with inline delete.

## MIDI Learn flow

1. User clicks **Recognize button** in the binding dialog
2. Frontend enters `isLearning` state and listens for `midi-raw-sniff` events
3. Backend emits `midi-raw-sniff` with `(channel, cc_number)` whenever any CC message arrives
4. On capture, the dialog auto-fills channel + CC and exits learn mode
5. User confirms and saves the binding via `registerMidiBinding`

## Persistence

MIDI bindings are persisted alongside the amp configuration in `amp-config.json`:

- On startup, saved bindings are loaded and restored to the `MidiService` via `set_bindings()`
- Every `registerMidiBinding` / `removeMidiBinding` command persists the full binding list via
  `AmpConfigPersistenceService::persist_midi_bindings()`
- The persistence layer uses a trait-based repository pattern (`AmpConfigPersistenceTrait`) with a JSON file
  implementation

## Effect parameter bus

The `EffectParameterBus` (`services/effect_parameter_bus.rs`) provides a lock-free-friendly bridge between `MidiService`
and live effect chains:

- `AudioService` registers channels via `register_channel()`
- MIDI callback calls `set_param()` or `toggle_bypass()` without touching `AudioService` directly
- This avoids deadlocks between the Tauri command `Mutex<AudioService>` and the real-time MIDI thread

