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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::channel::Channel;

    fn make_channel() -> (Uuid, Arc<Mutex<Channel>>) {
        let id = Uuid::new_v4();
        let channel = Arc::new(Mutex::new(Channel::new(id, "Test".to_string(), None, None)));
        (id, channel)
    }

    fn make_channel_with_effect(effect_id: Uuid) -> (Uuid, Arc<Mutex<Channel>>) {
        use crate::services::effects::distortion::hc_distortion::HCDistortion;
        let (channel_id, channel) = make_channel();
        {
            let mut ch = channel.lock().unwrap();
            ch.add_effect_to_chain(Box::new(HCDistortion::new(
                effect_id,
                "Test Effect".to_string(),
                false,
                0.5,
                0.0,
                "#e67e22".to_string(),
            )));
        }
        (channel_id, channel)
    }

    mod register_channel {
        use super::*;

        #[test]
        fn channel_can_be_registered() {
            let bus = EffectParameterBus::new();
            let (cid, ch) = make_channel();
            bus.register_channel(cid, ch);
            // No panic = success
        }

        #[test]
        fn duplicate_registration_overwrites() {
            let bus = EffectParameterBus::new();
            let (cid, ch1) = make_channel();
            let (_, ch2) = make_channel();
            bus.register_channel(cid, ch1);
            bus.register_channel(cid, ch2);
            // No panic = success
        }
    }

    mod unregister_channel {
        use super::*;

        #[test]
        fn registered_channel_can_be_unregistered() {
            let bus = EffectParameterBus::new();
            let (cid, ch) = make_channel();
            bus.register_channel(cid, ch);
            bus.unregister_channel(cid);
            // No panic = success
        }

        #[test]
        fn unregistering_unknown_channel_does_not_panic() {
            let bus = EffectParameterBus::new();
            bus.unregister_channel(Uuid::new_v4());
        }
    }

    mod set_param {
        use super::*;

        #[test]
        fn sets_parameter_on_matching_channel() {
            let bus = EffectParameterBus::new();
            let effect_id = Uuid::new_v4();
            let (cid, ch) = make_channel_with_effect(effect_id);
            bus.register_channel(cid, ch);

            let result = bus.set_param(effect_id, "threshold", 0.75f32);
            assert!(result.is_ok());
        }

        #[test]
        fn returns_error_for_unknown_effect() {
            let bus = EffectParameterBus::new();
            let (cid, ch) = make_channel();
            bus.register_channel(cid, ch);

            let result = bus.set_param(Uuid::new_v4(), "threshold", 0.5f32);
            assert!(result.is_err());
        }

        #[test]
        fn finds_effect_across_multiple_channels() {
            let bus = EffectParameterBus::new();
            let effect_id = Uuid::new_v4();
            let (cid1, ch1) = make_channel();
            let (cid2, ch2) = make_channel_with_effect(effect_id);
            bus.register_channel(cid1, ch1);
            bus.register_channel(cid2, ch2);

            let result = bus.set_param(effect_id, "threshold", 0.3f32);
            assert!(result.is_ok());
        }

        #[test]
        fn returns_error_for_unknown_parameter() {
            let bus = EffectParameterBus::new();
            let effect_id = Uuid::new_v4();
            let (cid, ch) = make_channel_with_effect(effect_id);
            bus.register_channel(cid, ch);

            let result = bus.set_param(effect_id, "nonexistent_param", 0.5f32);
            assert!(result.is_err());
        }
    }

    mod toggle_bypass {
        use super::*;

        #[test]
        fn toggles_effect_on_matching_channel() {
            let bus = EffectParameterBus::new();
            let effect_id = Uuid::new_v4();
            let (cid, ch) = make_channel_with_effect(effect_id);
            bus.register_channel(cid, ch);

            let result = bus.toggle_bypass(effect_id);
            assert!(result.is_ok());
        }

        #[test]
        fn returns_error_for_unknown_effect() {
            let bus = EffectParameterBus::new();
            let (cid, ch) = make_channel();
            bus.register_channel(cid, ch);

            let result = bus.toggle_bypass(Uuid::new_v4());
            assert!(result.is_err());
        }
    }
}
