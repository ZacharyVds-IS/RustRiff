# Master Volume

Master volume is the final volume control later in the chain.

It behaves similarly to Gain (both are level controls), but they operate at different points.

## Where master volume sits in the chain

`Input device -> Gain -> Tone shaping -> Master volume -> Output device`

## Why this distinction matters

If gain and master volume are set to the same numeric value, the sound can still feel different from changing only one
of them:

- Raising gain changes the upstream level feeding the rest of the chain.
- Raising master volume mainly changes final loudness.