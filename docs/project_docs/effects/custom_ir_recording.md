# Recording Impulse Response
An impulse response is a simple audio (wav file) containing the response of a specific environment to an impulse.

These can be easily recorded yourself.

## Recording an impulse response yourself.
The easiest way to record an impulse response yourself is to simply record a clap inside the desired environment.

1. Go to your desired location.
2. Setup a microphone and a recording device (e.g. a laptop and a microphone).
3. Start the recording and clap
4. Stop recording and save the file.
5. Trim the file so the response is at the very beginning of the file.
> If not trimmed, the impulse response will be delayed by the amount of silence at the beginning of the file.
6. Save the file as a wav file and use it as an impulse response in the project.

## Professional recordings
The method above describes a DIY approach to recording impulse response.

This DIY method isn't ideal since the clap is mainly in the mid tones. Ideally we want to know the reaction for all frequencies.

On a professional level for recording an amp's impulse response for ex.
1. is to place a microphone in front of the speaker cabinet (like required for normal recording).
2. Then generate a test signal (typically a sine sweep from 20Hz to 20kHz).
3. Send this signal through the speaker and record what comes out again.
4. Use software to deconvolve or mathematically compare the original test signal with the recorded signal. Which will extract the unique sonic fingerprint of the speaker.
5. Trim the audio file to start exactly when the impulse begins and save it as a .wav file.

> For our custom recorded content we have used the simpler "clap" method, which is sufficient for demonstration purposes.
