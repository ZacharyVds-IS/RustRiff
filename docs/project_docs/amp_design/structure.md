# Amplifier Structure
This page explains how RustRiff maps to a physical guitar amplifier and where each setting is stored in the app.
## Signal chain (physical amp vs RustRiff)
In a traditional rig, the path is usually:

`Guitar -> Input stage (gain) -> Preamp tone shaping -> Power amp (master volume) -> Speaker/cab`

In RustRiff, the same idea maps to:

`Input device -> Gain processor -> tone stack -> Effect Chain ->  Master volume -> Output device`

- **Input device**: Your interface or microphone source selected in `Settings`.
- **Gain**: Controls input amplification before effect application.
- **Tone stack (Low/Mid/Bass)**: A basic equalizer of our Low, Mid and Bass controls.
- **Effect chain**: The series of effects you have enabled (distortion,delay,reverb, etc).
- **Master volume**: Final volume control for the output stage.
- **Output device**: Where sound is sent (headphones, interface output, speakers).

## What each setting means
### `is_active` (On/Off switch)
- Represents whether the loopback/audio engine is active.
- **Physical analogy**: amp power/standby behavior.

### `gain`
- Controls input signal amplification.
- Higher values can drive a stronger, potentially more saturated signal depending on processing.

### `master_volume`
- Controls final output level after upstream processing.
- **Physical analogy**: power amp/master knob.
### Input/Output device IDs
- Store the selected hardware routing target by device id.
- Used to hot-swap the active source/sink at runtime.
