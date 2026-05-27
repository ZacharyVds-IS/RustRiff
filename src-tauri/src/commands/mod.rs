//! Tauri command handlers for the Guitar Amplifier application.
//!
//! This module contains all the command handlers that are exposed to the frontend
//! via Tauri's IPC layer. Each submodule groups related commands:
//!
//! - [`loopback`] — Audio loopback control
//! - [`default_controls`] — Gain, master volume, and tone stack equalizer configuration
//! - [`settings`] — Input/output device selection and enumeration

pub mod analyzer;
pub mod channels;
pub mod default_controls;
pub mod effect_commands;
pub mod helpers;
pub mod latency_testing;
pub mod loopback;
pub mod settings;
pub mod tuner;
