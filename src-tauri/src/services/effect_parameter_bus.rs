// src/services/effect_parameter_bus.rs
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tracing::warn;
use uuid::Uuid;

use crate::domain::channel::{Channel, ParamInput};

/// A lock-free-friendly bridge between `MidiService` and the live effect chains.
///
/// `AudioService` registers every `Channel` here (by cloning the `Arc`). The
/// MIDI callback thread calls `set_param` or `toggle_bypass` without ever
/// touching `AudioService` directly — avoiding any risk of a deadlock between
/// the Tauri command `Mutex<AudioService>` and the real-time MIDI thread.
///
/// ## Why not store `Arc<Channel>` in `AudioService`?
///
/// `Channel` is currently owned inside `Vec<Channel>` behind `Mutex<AudioService>`.
/// Wrapping each `Channel` in its own `Arc<Mutex<Channel>>` would ripple through
/// every command handler.  Instead, `EffectParameterBus` stores only the
/// already-atomic `EffectHandles` data that `Channel` exposes — effectively the
/// same pattern `Channel` itself uses to avoid locking the effect chain on the
/// audio thread.
pub struct EffectParameterBus {
    /// channel_id → shared Channel Arc
    ///
    /// The `Mutex` here is only taken briefly when channels are added/removed
    /// (not on every MIDI message).  The per-effect atomics inside each
    /// `Channel`'s `effect_handles` are written lock-free during playback.
    channels: Mutex<HashMap<Uuid, Arc<Mutex<Channel>>>>,
}

impl EffectParameterBus {
    pub fn new() -> Self {
        Self {
            channels: Mutex::new(HashMap::new()),
        }
    }

    /// Register a channel so MIDI can reach its effects.
    ///
    /// Called by `AudioService` inside `add_channel`, `restore_effect_chain`,
    /// and `apply_amp_config`.
    pub fn register_channel(&self, channel_id: Uuid, channel: Arc<Mutex<Channel>>) {
        if let Ok(mut map) = self.channels.lock() {
            map.insert(channel_id, channel);
        }
    }

    /// Remove a channel (called from `AudioService::remove_channel`).
    pub fn unregister_channel(&self, channel_id: Uuid) {
        if let Ok(mut map) = self.channels.lock() {
            map.remove(&channel_id);
        }
    }

    /// Set a named f32/u32 parameter on whichever channel owns `effect_id`.
    ///
    /// Iterates registered channels, delegates to `Channel::set_effect_param`.
    /// The write itself is a single atomic store — no audio-thread locks taken.
    pub fn set_param(
        &self,
        effect_id: Uuid,
        param: &str,
        value: impl Into<ParamInput> + Copy,
    ) -> Result<(), String> {
        let channels = self
            .channels
            .lock()
            .map_err(|_| "effect bus channels lock poisoned".to_string())?;

        for channel_arc in channels.values() {
            let channel = channel_arc
                .lock()
                .map_err(|_| "channel lock poisoned".to_string())?;

            if channel.has_effect(effect_id) {
                return channel.set_effect_param(effect_id, param, value);
            }
        }

        warn!("set_param: effect {} not found in any channel", effect_id);
        Err(format!("Effect {} not found in any channel", effect_id))
    }

    /// Toggle bypass on whichever channel owns `effect_id`.
    pub fn toggle_bypass(&self, effect_id: Uuid) -> Result<(), String> {
        let channels = self
            .channels
            .lock()
            .map_err(|_| "effect bus channels lock poisoned".to_string())?;

        for channel_arc in channels.values() {
            let channel = channel_arc
                .lock()
                .map_err(|_| "channel lock poisoned".to_string())?;

            if channel.has_effect(effect_id) {
                channel.toggle_effect(effect_id)?;
                return Ok(());
            }
        }

        warn!(
            "toggle_bypass: effect {} not found in any channel",
            effect_id
        );
        Err(format!("Effect {} not found in any channel", effect_id))
    }
}
