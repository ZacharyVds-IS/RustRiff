# Cabinet Simulation (Impulse Response)
Impulse Response (IR) is a digital snapshot of how a speaker's cabinet, room or space responds to sound.

In simple terms: If I record sound in my bathroom and make someone gues what room I was in, most people will gues the bathroom correctly. 
This is because they recognize the impulse response of this room.

## IR in digital signal processing.
An impulse response is typically shipped in a .wav file and contains a simple snapshot of a speakers cabinets.
Once we read that file in we will find a simple audio buffer containing values.

Contents of a IR file look something like this:
```Rust
    const ECHO_IR: [f32; 6] = [
    1.0,   // direct sound
    0.0,
    0.0,
    0.35,  // first echo
    0.0,
    0.15,  // second echo
];
```
## How can we now use this to simulate a cabinet?
Simply put using IR is a "blending" operation. We take our dry guitar signal and for every single sample, it gets multiplied by the entire IR snapshot. This applies the tonal characteristics (EQ) and the reflections (reverb/echo) of the cabinet to your sound.

### Applying IR: Convolution
In DSP we calculate the output by sliding the IR over our input signal. For every moment in time, the output is the sum of the current input and previous inputs, each weighted by the values in the IR buffer.

Convolution Formula:
$$(x * h)[n] = \sum_{k=0}^{M-1} x[n-k] \cdot h[k]$$

where:
n: The current sample index.
M: the length of your IR (6 in the example above).
h[k]: The k-th value of the IR buffer.

### Limitations of Convolution
The above example works fine with small buffers. But in DSP IR buffers easily go up to 2048 and more. 
This maps to a verry cpu intensive operation.

### A solution to this problem
To solve this problem we can make use of Fast Fourier Transform (FFT).

FFT is a mathemtical bridge. It takes a signal from the Time Domain (amplitude over time) and converts it into the Frequency Domain (magnitude and phase over frequency).
Simply put this means that instead of the complex sliding and summing math. We can simply multiply the individual frequency components together.

### FFT workflow
1. Take the dry guitar signal and convert them into its frequency components.
2. apply FFT to the ir: Convert your Cabinet IR into its frequency components (this only needs to be done once).
3. Multiple: Multiply the two frequency graphs together. If the IR has a huge dip at 500Hz, the multiplication will "carve" that dip into your guitar signal.
4. Inverse FFT (IFFT): Convert the result back into the Time Domain so our speakers can use it.

## Quick comparison between FFT and regular convolution.
| Method | Complexity | Use Case | Latency|
|--------|------------|----------|--------|
| Regular Convolution | O(n2)      | Short IRs (up to 256 samples) | Low |
| FFT-based Convolution | O(N log N) | Long IRs (256 samples and above) | Higher (due to block processing) |

## Conclusion
In modern audio engineering, Impulse Response are the gold standard for capturing the soul of hardware. Whether it's the woody resonance of a vintage 4X12 cab or the lush reflectiosn of a stone cathedral, IRs allow us to transport those physical spaces into our workstations.

The "sliding math" of Time-Domain Convolution is conceptually simple and perfect for low-latency tasks, the FFT is what makes professional cabinet simulation viable, allowing us to process complex, high-resolution snapshots without crushing the CPU.