# Gain

Gain controls how hard the input signal is driven at the start of the chain.
In RustRiff, gain is applied early (input stage), before master volume.

## Where gain sits in the chain

`Input device -> Gain -> Tone shaping -> Master volume -> Output device`

- Higher gain means a stronger input signal.
- Lower gain keeps the signal cleaner and gives more headroom.

## The algorithm

Gain can be implemented as a simple multiplier on the input audio samples, but if we were to alter the gain in this way
while playing, we would get a very abrupt change in the audio output. To avoid this, we use a smoothing algorithm to
gradually transition between gain levels.

This smoothing uses the following formula:
$$
g[n] = g[n-1] + \alpha (g_{target}[n] - g[n-1])
$$

Where $g[n]$ is the current gain at sample $n$, $g[n-1]$ is the gain at the previous sample, $g_{target}[n]$ is the
target gain level we want to reach and $\alpha$ is the smoothing factor (0 < $\alpha$ < 1) that determines how quickly
the gain transitions to the target level. We have set $\alpha$ to 0.001, which means the gain will change very slowly,
creating a smooth transition without abrupt changes in the audio output.

Finally, the output audio sample is calculated by multiplying the input sample $x[n]$ by the current gain $g[n]$:
$$
y[n] = x[n] \cdot g[n]
$$