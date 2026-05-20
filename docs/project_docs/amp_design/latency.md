# Latency

Latency is the time it takes for an audio signal to go in, be processed, and come back out.

In practice, latency is not just one thing. In this project we look at latency in a few separate ways so it is easier to understand what part of the system causes delay.

## Types of latency

- **Input / output latency**  
  The delay caused by the audio system itself, such as the audio driver, hardware, and system buffering.

- **Algorithmic latency**  
  The delay added by an audio effect because the algorithm needs to keep or inspect samples before producing output.

- **Buffer latency**  
  The delay caused by collecting samples in a buffer before they are processed or played back.

## How we measure algorithmic latency

In our backend, algorithmic latency is measured in `src-tauri/src/services/audio_latency_service.rs` and returned per processor in the DSP chain.

We first express latency in **samples**, and then convert that to **milliseconds** using the current output sample rate.

The conversion is:

`latency_ms = latency_samples / sample_rate * 1000`

The values are exposed to the frontend through the latency command module and then shown in `src/components/EffectControls.tsx`.

### What this means for our current effects

For the current DSP chain:
- Gain
- Tone Stack
- Master Volume

the measured algorithmic latency is currently **0 samples**.
That means these processors do change the sound, but they do **not** intentionally delay it by storing audio and playing it later.
In other words, they do work on the signal immediately, sample by sample.

## How we measure buffer latency

Buffer latency is measured separately in `src-tauri/src/services/audio_latency_service.rs` through `measure_buffer_latency`.

We estimate:
- **input buffer latency**
- **output buffer latency**
- **total buffer latency**

The calculation is based on:
- buffer size in frames
- sample rate

The formula is:

`buffer_latency_ms = buffer_frames / sample_rate * 1000`

So if the system uses a larger buffer, latency goes up.  
If the sample rate is higher, latency for the same buffer size goes down.

When CPAL reports `BufferSize::Default`, we cannot read an exact frame count from the configuration. In that case we use a fallback value of **256 frames** so the UI can still show a practical estimate.

## Why buffer latency is called an estimate

The buffer latency shown in the UI is an **estimated buffer latency**, not a full real-world round-trip measurement.

This is because it is based on the configured stream buffer sizes and sample rates.
It does **not** include every possible delay in the real audio path, such as:

- hardware converter delay
- operating system scheduling delay
- driver safety offsets
- extra internal buffering in the audio stack
- resampler chunking or any other processing-stage buffering

So this metric is useful for understanding the buffering cost of the current digital setup, but it should not be treated as a perfect measurement of total real-world latency.

## How this is shown in the UI

In `src/components/EffectControls.tsx` we display:
- **algorithmic latency** per processor
- **estimated buffer latency** for the current system configuration

This makes it easier to distinguish between:
- delay added by an effect itself
- delay added by the audio system configuration

## Summary

In short:

- **Algorithmic latency** tells us whether an effect itself adds delay.
- **Buffer latency** tells us how much delay is introduced by buffering audio before processing or playback.
- Our current Gain, Tone Stack, and Master Volume have **0 samples of algorithmic latency**.
- The buffer latency shown in the UI is an **estimate** based on buffer size and sample rate.
