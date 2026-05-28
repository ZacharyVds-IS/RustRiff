# MIDI support
Musical Instrumen Digital Interface is a universal standard in which musical devices, computers and software can communicate with each other.
MIDI doesn't send over audio, it sends data, information such as: what note is being placed/pressed, how long (timing) and how hard.

## MIDI in RustRiff
RustRiff relies on MIDI for controlling our various effects. For example: wah can now be mapped by an expression pedal, similar to turning the pedal on or off.

this adds compatibility for midsong changes, which is a common use case for guitarists.

## What data?
- Channels, take control on which channel specific things are transmitted allowed for full control preventing any cross-interference.
- CC (Control Change) this is a nubmer that tells RustRiff wchich knob, switch or pedal is being moved.
- Data value:(0 - 127) this is a value being sent int, RustRiff listens for the value on "expression pedals" any other binds are handled as toggles for ex. on/off button always sends value 127 but the software simply toggles its state based on the input.

