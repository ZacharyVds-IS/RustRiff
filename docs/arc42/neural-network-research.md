# Neural Networking in DSP

Neural networks can be used to run in audio signal processors.

Neural networks tend to capture the true feel of an amp, a full rig... Altough they cannot emulate time based effects
like delay, reverb...

## NAM (NN) vs IR

Nam can caputre full rigs (amp head + cab + combos), it can also handle amp heads themselves and pedals.

IR's capture speaker cab's, potentialy reverb and room acoustics.

> If an NAM capture of soley an amp head is used an IR profile is required for the final shaping.

## Existing solutions

- Neural DSP builds soft and hardware which use neural networks to provide a given tone tot your setup @ home. Altough
  their solutions are not free available.

- NAM (neural amp modeler) is an open source project by Tone 3000 which has a large community around them. They also
  provide a core package in C++ which we could use from RUST if we utilize bindgen into that flow for succesfull
  overflow. This will allow us to support but also utilize the big community that NAM has around it including existing
  models.

- DIY a last option for us would be to train our own models. We could take an input signal and output signal and train a
  neural network between them. Altough this would be a nice learning experience, it would also mean we can only emulate
  our own rigs and not use just straight up anything.

## How can RustRiff implement this?

1. Support NAM using C++ bindgen. NAM provides a core repository written in
   C++ [NAM Core](https://github.com/sdatkinson/NeuralAmpModelerCore). we could use Bindgen to generate Rust bindings on
   this core library and implement the functionality using these bindings.
   We essentialy just use the C++ package in our Rust environment.

> Important note to make is that Bindgen does not work flawlesly on C++ classes as it''s designed for regular C. Writing
> a small C-compatible wrapper interface inside the C++ layer should be used to workaround this fact.

2. Rewrite [NAM Core](https://github.com/sdatkinson/NeuralAmpModelerCore) in Rust. Since nam (and core) are open source
   and released under MIT license we could rewrite the Original Package in Rust making it run natively in Rust.

> The above two options come with the additional benefit of a full community with trained ready-models. Other big brands
> also use NAM compatibility like valeton, Hotone, Neural AMp... (full
> list [here](https://www.tone3000.com/guides/neural-amp-modeler))

> Relying on NAM doesn't mean we cannot introduce our own models or gear. Since its open source they support the
> creation of custom models seemlessly. More info [here](https://www.tone3000.com/capture)

3. Train and use our own models. Training models is a bit of rabithole but in general this includes: feeding a specific
   signal through the amp (for ex sine sweep) -> then knob sweeps (capturing knobs at various positions) are captured as
   well, they record the dry input and the wet output signal. Basicly capturing lots of data about the physical
   hardware's performance. After which a network architecture is decided upon, the big two are LSTM/GRU (recurrent
   neural networks) and CNN's (onvolutional Neural nets) specifically (TCN temporal convolution networks). These look
   at "windows" of audio over time to predict the next sample. Then comes an actual training step the model is given raw
   input signal, it makes a guess at what the physical amp would do to it, outputting its own new version. Then the "
   loss" can be calculated ex. ESR(Error-To-Signal ratio) measures raw sample-by-sample dif. or MRSTFT (Multi Resolution
   Short Time Fourier Transform Loss) mesures the dif in the frequency spectrum over time. after which a backpropagation
   pass is made to improve upon the model. After which a model will require optimization for good realtime use.

4. Rely on the weights from NAM but run them in a general runtime like ONNX. Each .Nma file also carries just regular
   weights, we could strip them and simly use those weights inside of an existing runtime. Altough the more detailed
   factors of nam would be lost. Ex. samplerate.

6. Use [RTNeural](https://github.com/jatinchowdhury18/RTNeural) (C++ library) its a lightweight neural network
   inferencing engine written in C++. using bindgen similarly to what we would do with namcore we could use this in our
   Rust Enviromnment and then use the weights from the NAM models or custom models as described above.

7. [ONNX](https://onnx.ai/) Based on Lars' chessbot projet, we could use ONNX since this provides a NN runtime in the
   rust environment. Using this we could use custom made models or use NAM models (weights) after converting these over
   into the ONNX format.

# Implementation Matrix: Neural Networking in RustRiff

This document evaluates the prospective approaches for implementing Neural Network-based digital signal processing (DSP)
within RustRiff, comparing integration complexity, performance, and community leverage.

| Option | Approach                                                                                      | Pros                                                                                                                                                                                                                                                                                                                                                                                                                 | Cons                                                                                                                                                                                                                                                                                                                                                                                                                    |
|:-------|:----------------------------------------------------------------------------------------------|:---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|:------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| **1**  | **Support NAM using C++ Bindgen**<br>*(Wrap & bind sdatkinson/NeuralAmpModelerCore)*          | <ul><li>Instant access to NAM's massive community ecosystem of thousands of free pre-trained models.</li><li>Maintains perfect parity with NAM's DSP code updates and samplerate handling.</li></ul>                                                                                                                                                                                                                 | <ul><li>`bindgen` struggles with complex C++ features; requires maintaining a manual C-compatible wrapper layer (`extern "C"`).</li><li>Introduces cross-compilation friction and reliance on a C++ compiler toolchain within the Rust workflow.</li><li>Potential memory safety hazards at the FFI boundary if pointer/buffer management is incorrect.</li><li>Makes debugging realy cluttered / hard to follow</li></ul>                                                       |
| **2**  | **Rewrite NAM Core natively in Rust**<br>*(Port original architecture to pure Rust)*          | <ul><li>100% memory-safe, idiomatic Rust codebase with zero FFI overhead or toolchain friction.</li><li>Enables native optimization (e.g., SIMD auto-vectorization via `core::arch` or `ndarray`).</li><li>Retains full compatibility with the existing `.nam` model ecosystem and architecture definitions (LSTM/WaveNet).</li><li>Could be released on it's own as the rust counterpart for other users.</li></ul> | <ul><li>High upfront development effort to accurately rewrite and validate complex DSP logic and mathematical structures.</li><li>Risk of subtle bugs or performance regressions relative to the heavily optimized C++ core.</li><li>Requires ongoing maintenance to manually port upstream bugfixes or feature updates from the original repository.</li></ul>                                                         |
| **3**  | **Train and Use Custom Models**<br>*(LSTM/GRU/TCN trained from scratch)*                      | <ul><li>Complete intellectual freedom and independence from third-party architectures and licensing constraints.</li><li>Deep, practical learning experience covering audio data acquisition, loss functions (ESR, MRSTFT), and custom network design.</li><li>Ability to innovate on architecture variations optimized specifically for RustRiff's performance constraints.</li></ul>                               | <ul><li>Massive "rabbit hole" requiring dedicated machine learning expertise, training pipelines, and high-compute hardware.</li><li>Isolates the project from the NAM community; users cannot load third-party rigs, limiting RustRiff to models trained explicitly by the team.</li><li>A massive amount of manual hardware profiling (sine sweeps, knob sweeps) is required to build a functional library.</li></ul> |
| **4**  | **Extract NAM Weights into Generic Runtime**<br>*(Parse `.nam` JSON/Protobuf files directly)* | <ul><li>Avoids compilation of C++ dependencies while still reading existing models.</li><li>Decouples the neural network architecture from specific framework bindings by evaluating raw weight matrices natively.</li></ul>                                                                                                                                                                                         | <ul><li>Loses specialized NAM logic, metadata parsing, and sample-rate adjustment heuristics embedded inside NAM Core.</li><li>High risk of structural mismatch if NAM introduces new architecture variants or formatting changes.</li><li>Requires writing custom execution blocks for the network layers.</li></ul>                                                                                                   |
| **6**  | **Utilize RTNeural via Bindgen**<br>*(Lightweight C++ inference engine)*                      | <ul><li>RTNeural is highly optimized specifically for real-time audio DSP execution and handles common layers (LSTM, GRU, Dense) flawlessly.</li><li>Saves development time relative to creating a new inference backend.</li></ul>                                                                                                                                                                                  | <ul><li>Shares the same FFI/Bindgen drawbacks as Option 1, requiring a C-wrapper interface.</li><li>Requires a translation pipeline to map NAM weights or custom architectures into the specific structural formats expected by RTNeural.</li></ul>                                                                                                                                                                     |
| **7**  | **Deploy ONNX Runtime inside Rust**<br>*(Utilize `ort` or native ONNX bindings)*              | <ul><li>Leverages a robust, production-grade neural network engine with cross-platform optimization (CPU execution providers, CoreML, DirectML).</li><li>Allows team members to use standard ML tools (PyTorch, TensorFlow) to train models and export via `onnx`.</li><li>Prior precedent established in the ecosystem (e.g., Lars' chessbot project).</li></ul>                                                    | <ul><li>ONNX Runtime binaries are large, significantly expanding the compiled binary size of RustRiff.</li><li>Not inherently optimized for sample-by-sample stateful audio tracking (like hidden states in real-time LSTM/GRU execution), potentially causing latency overhead.</li><li>Requires an external conversion utility to translate existing `.nam` files into the standard `.onnx` schema.</li></ul>         |

***

### Key Technical Considerations for Selection

* **Community Leverage:** Options **1, 2, 5, and 6** allow integration with existing model ecosystems, preventing the "
  cold-start" problem where RustRiff has no accessible gear models.
* **Real-time Performance:** Audio buffers require execution within strict deadlines (e.g., < 2-5ms). Options **2 (
  Native Rust)** and **5 (RTNeural)** provide specialized optimizations tailored for low-overhead inner-loop execution,
  whereas general engines like **6 (ONNX)** must be carefully tuned to prevent garbage collection or buffer copy stalls.