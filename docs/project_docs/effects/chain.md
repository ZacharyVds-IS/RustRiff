# Chain
The effect chain is the actual chain in which events are being processed. 
The order of effects could impact the final result since the effect is being passed already processed audio by the previous effect.

For example:
```
Equalizer -> Distortion
```
will sound different then 
```
Distortion -> Equalizer
```
The first sample the distortion will be applied to a pre equalized sample. While the second example will distory the original input audio after which it gets equalized.