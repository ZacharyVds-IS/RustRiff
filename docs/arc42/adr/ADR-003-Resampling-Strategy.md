# ADR-003: Automatic Resampling Strategy

**Status:** Accepted  
**Date:** May 2026  
**Context:** RustRiff - Cross-Platform Guitar Amplifier

## Problem

Input and output audio devices may operate at different sample rates. The DSP chain needs a consistent sample rate to process audio correctly, and we want to minimize CPU usage while maintaining audio quality.

## Alternatives Considered

| Alternative | Pros | Cons |
|---|---|---|
| **Automatic ResamplePolicy (Rubato)** | Handles all rate combinations, optimizes DSP execution rate, transparent to DSP processors | Adds latency from resampling, CPU overhead for resampler |
| Force matching sample rates | No resampling overhead, simplest | Requires user to manually match devices, may not be possible with all hardware |
| Fixed DSP sample rate | Predictable performance, simple | May require resampling in both directions, potential quality loss |
| Process at input rate, resample output only | DSP runs at input rate, simpler pipeline | DSP may run at unnecessarily high rate if input > output |

## Decision

Use custom **`ResamplePolicy`** with the **Rubato** crate for automatic sample rate conversion.

The policy is selected at loopback startup by comparing input and output sample rates:

| Condition | Policy | Behavior |
|---|---|---|
| input == output | `Bypass` | No resampling. Zero overhead. DSP runs at the common rate. |
| input > output | `PreDsp` | Downsample before DSP. DSP runs at the lower output rate (cheaper). |
| input < output | `PostDsp` | Upsample after DSP. DSP runs at the lower input rate (cheaper). |

The key insight: DSP always runs at the **lower** of the two rates, minimizing CPU usage.

- `PreDsp`: Input samples are downsampled first, then processed by DSP. Output is already at the correct rate.
- `PostDsp`: Input samples are processed by DSP first, then upsampled to match the output device rate.
- `Bypass`: No resampler is created. Samples flow directly through DSP.

The resampler uses a configurable chunk size (256 samples) balancing quality against latency.

## Consequences

- **Positive:** DSP always runs at the lowest possible rate, transparent to DSP processors (they don't need to know about sample rates), handles any input/output rate combination automatically
- **Negative:** Resampling adds some latency and CPU overhead, Rubato crate adds a dependency
- **Risk:** Poor resampling quality could introduce audio artifacts. Mitigated by using Rubato (high-quality resampler) and tuning the chunk size parameter.
