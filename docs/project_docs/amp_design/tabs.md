# Tabs
Tablatures are a way musicians can easily see what notes to play on their instrument. They are a form of musical notation that uses numbers and symbols to represent the strings and frets of a guitar, bass, or other stringed instrument.
We wanted to integrate a way for guitarists to load in a tab in our app and be able to play along with it. This would allow users to practice their timing and rhythm while playing along with a song.

## AlphaTab
To achieve this, we integrated the [AlphaTab](https://www.alphatab.net/) library into our app. AlphaTab is a powerful JavaScript library that can parse and render guitar tablature in a web application. It supports a wide range of tab formats and provides a user-friendly interface for displaying tabs.

It supports the following file formats:
- Guitar Pro 3-5 files `.gp3`, `.gp4`, `.gp5`
- Guitar Pro 6 files `.gpx`
- Guitar Pro 7 files `.gp`
- MusicXML files `.xml`
- CapXML files `.capx`
- alphaTex files 