//! Infrastructure adapters that integrate the domain/services with external audio APIs.

#[cfg(feature = "audio-backend")]
pub mod audio_handler;
pub mod file_loader;
#[cfg(feature = "audio-backend")]
pub mod midi_handler;
pub mod midi_parser;
pub mod persistence;
