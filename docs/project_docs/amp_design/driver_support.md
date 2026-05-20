# Audio Drivers

RustRiff uses CPAL under the hood for its audio input and output.

CPAL uses a platform-specific backend to access the audio hardware. By default these are:

| Platform | Backend   |
|----------|-----------|
| Windows  | WASAPI    |
| Linux    | ALSA      |
| macOS    | CoreAudio |

## Audio Driver Latency
### Windows
On windows WASAPI is active as the default audio driver within the CPAL library.

WASAPI or Windows Audio Session API has 2 modes in which it can operate. The first being Shared Mode, within this mode multiple apps can output sound simultaneaously. Windows will then take care of all audio stream together, applies volume controls, and resamples the audio to match your system's default bit depth and sample rate.
The second mode is Exclusive Mode, in this mode a single media player or DAW takes complete, exclusive control of the audio device. The windows mixer is entirely bypassed. 

This means that the mode in which WASAPI operates has a significant impact on the latency of the audio output. In Shared Mode, the latency is typically higher due to the additional processing performed by the Windows mixer. In Exclusive Mode, the latency can be significantly reduced as the audio stream is sent directly to the hardware without any intermediate processing.

The main issue with WASAPI exclusive mode is that it currently is not being supported through the CPAL package (which RustRiff uses for audio input and output).

#### Low Latent Windows Audio Drivers
Since we consider our user to have a high likelyness to use an audio interface, we can assume that they will have access to a low latency audio driver. The most common low latency audio drivers for Windows outside of the native windows scope is ASIO.
Luckely we can enable ASIO support in cpal by enabling the `cpal/asio` feature flag. This will allow users to use ASIO drivers for low latency audio output on Windows.

ASIO (Audio Stream Input/Output) works similar to windows WASAPI exclusive mode, in that it allows applications to bypass the Windows mixer and communicate directly with the audio hardware.
This itself results in significantly reduced latency compared to the default WASAPI shared mode.