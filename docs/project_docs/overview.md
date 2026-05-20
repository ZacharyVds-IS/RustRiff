# Overview
RustRiff is a desktop guitar amplifier app built with:

- React (TypeScript) for the UI.
- Rust for backend logic and audio processing.

## What this project does
RustRiff enables your computer as a virtual guitar amplifier. To ensure a good overall experience you should make use of an audio interface like a FocusRite Scarleet Solo or similar. 


## Why Rust
The most important part of a guitar amplifier isn't necessarily the sound. It's the feeling it gives the guitarist. 
To achieve this feeling the amplifier needs to be responsive and have low latency.

In the analysis phase of the project, we compared Rust to C++ and Go and found the follwing:
- Rust and C++ have similar performance characteristics, while Go introduces a lot of latency due to an abstraction ontop of the audio device layer.
- Rust has good package support in the form of so called "crates".

This sounds like Rust and C++ would have a very similar experience. But since C++ is the industry standard for audio processing you end up quickly within the JUCE framework which takes away a lot of the actual coding.
Instead we opted for the non-industry standard Rust which allows us to have a greater learning experience.

## Documentation map
- Read `Project Structure` for a folder-level guide.
- Read `Development Workflow` for local setup and doc generation.
- Use API references for symbol-level details:
  - [Frontend API (TypeDoc)](/frontend/index.html)
  - [Backend API (Rustdoc)](/backend/doc/rustriff_lib/index.html)

