# Why Analyze Audio Visually?

When shaping guitar tone, our ears are the final judge, but our ears are not always enough on their own.

A visual analyzer helps by making hidden signal behavior obvious:

- where energy is concentrated (low end, mids, highs)
- when gain staging is pushing levels too hard
- how an effect chain changes frequency balance over time
- whether a tone problem is real in the signal, or just a monitoring illusion

In short, visual analysis is not a replacement for listening; it is a second source of truth that improves confidence and speed when dialing tones.

## Important limitation

A spectrum view shows energy distribution, not musical quality.

Two tones can look similar and feel very different to a player.
So the best workflow is always:

1. Listen first
2. Use visual data to confirm or challenge your assumption
3. Listen again after adjustment

---

# Spectrum Analyzer (Short Technical Overview)

In this project, the analyzer visualizes the **post-chain processed signal** (gain, tone stack, effects, channel volume, master volume).

The backend flow is:

1. Processed samples are written into a lock-free `SpectrumTap` ring buffer.
2. Snapshot samples are windowed and transformed by FFT.
3. Log-spaced bins are converted to dBFS values.
4. Frames are streamed to the frontend as `live-spectrum` events.

The frontend then renders these frames in the analyzer chart with light temporal smoothing to reduce jitter while preserving responsiveness.