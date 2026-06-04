# Tone Stack

Our amp's tone stack consits of the following components:

- `Bass`: Controls the low frequencies of the sound.
- `Mid`: Controls the mid frequencies of the sound.
- `Treble`: Controls the high frequencies of the sound.

## Does tone stack boost the audio?

An important find about tone stack is the fact that this in fact does not boost the audio.
It rather attenuates specific audio frequencies. Meaning that if the selected value is at 0 the bass will be fully
filtered out.
Which in turn means that when set to its maximum value you end up with the same frequency as the input signal.

## Selected Frequencies

Bass for example doesn't have a verry strict frequency. This means we can configure the tone stack as desired ourselves.
Our frequency range's are :

- `Bass`: 0 - 180 Hz
- `Mid`: Peaking at 1200 Hz
- `Treble`: 2400 Hz - 20000 Hz

## Tone Stack Implementation

Our Tone Stack implementation relies on a modular, three-layered architecture to manage real-time audio equalization. At
the highest level, `tone_stack_processor.rs` coordinates the entire chain, passing the audio sample through separate bass,
middle, and treble equalizers in sequence. Each of these frequency ranges is handled by `range_eq.rs`, which translates
simple user percentage settings into decibel gains and maps them to the appropriate filter types. At the foundation,
`biquad.rs` executes the low-level digital signal processing, running a second-order IIR filter algorithm to smoothly
shape the audio stream sample by sample.

The IIR algorithm used by `biquad.rs` comes from [The Audio Cookbook](https://webaudio.github.io/Audio-EQ-Cookbook/audio-eq-cookbook.html). We use the Direct 1 form of the biquad equation.
$$$
y[n] = \left(\frac{b_0}{a_0}\right)x[n] + \left(\frac{b_1}{a_0}\right)x[n-1] + \left(\frac{b_2}{a_0}\right)x[n-2] - \left(\frac{a_1}{a_0}\right)y[n-1] - \left(\frac{a_2}{a_0}\right)y[n-2]
$$$
We use the following filter types for each frequency range:
- `Bass`: Low Shelf
- `Mid`: Peaking
- `Treble`: High Shelf