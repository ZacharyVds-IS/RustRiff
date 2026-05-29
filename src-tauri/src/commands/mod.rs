//! Tauri command handlers for the Guitar Amplifier application.
//!
//! This module contains all the command handlers that are exposed to the frontend
//! via Tauri's IPC layer. Each submodule groups related commands:
//!
//! - [`loopback`] — Audio loopback control
//! - [`default_controls`] — Gain, master volume, and tone stack equalizer configuration
//! - [`settings`] — Input/output device selection and enumeration

#[cfg(feature = "audio-backend")]
pub mod analyzer;
#[cfg(feature = "audio-backend")]
pub mod channels;
#[cfg(feature = "audio-backend")]
pub mod default_controls;
#[cfg(feature = "audio-backend")]
pub mod effect_commands;
#[cfg(feature = "audio-backend")]
pub mod helpers;
#[cfg(feature = "audio-backend")]
pub mod latency_testing;
#[cfg(feature = "audio-backend")]
pub mod loopback;
#[cfg(feature = "audio-backend")]
pub mod midi;
#[cfg(feature = "audio-backend")]
pub mod settings;
