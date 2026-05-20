# Gain
Gain controls how hard the input signal is driven at the start of the chain.
In RustRiff, gain is applied early (input stage), before master volume.

## Where gain sits in the chain
`Input device -> Gain -> Tone shaping -> Master volume -> Output device`
- Higher gain means a stronger input signal.
- Lower gain keeps the signal cleaner and gives more headroom.
