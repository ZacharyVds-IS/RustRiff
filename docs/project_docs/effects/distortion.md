# Distortion
The distortion effect lowers the clipping roof of the signal. This causes the tone to sound more "fuzzy" of "gritty".
There are 2 main ways of handling this process: **Hard Clipping** and **Soft Clipping**

## Hard Clipping
Hard clipping cuts the signal to the clipping roof. This results in the wave having a very harsh cut.
This results in the distortion sounding more aggressive and gritty.

## Soft Clipping
Soft clipping smoothes the transition to the roof so that the wave isn't cut off. 
This give the audio a more warm and compressed sound.

The function we use to smooth out the signal is defined as:

$$f(x)=\frac{x}{(1+|x|^n)^{\frac{1}{n}}}$$

Where $f(x)$ is the output or wet sample, $x$ is the ingoing/dry sample and $n$ is the smoothness parameter set between 1 and 10. 
1 being maximum smoothing and 10 minimal. If we don't limit $n$, then $n \rightarrow \infty$ would create Hard Clipping Distortion.

Because this function smoothed towards -1 and 1, 
we need to normalize the ingoing sample by dividing it by the configurable clipping limit and denormalizing the outgoing sample to get back to the desired amplitude.
