use crate::domain::dto::tone_stack_dto::ToneStackDto;
use crate::domain::effect::Effect;
use crate::domain::tone_stack::ToneStack;
use crate::services::effects::distortion::hc_distortion::HCDistortion;
use atomic_float::AtomicF32;
use std::collections::HashMap;
use std::mem::take;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::channel;
use std::sync::{Arc, Mutex};
use tracing::{error, info};

/// Atomic handles retained by `Channel` after the effect chain is moved to the
/// audio worker thread.  Commands write through these Arcs; the audio thread
/// reads from the same Arcs on every sample — completely lock-free.
struct EffectHandles {
    is_active: Arc<AtomicBool>,
    /// Named f32 parameters (e.g. `"threshold"`). The Effect trait's
    /// `f32_params()` method populates this, so no downcasting is needed.
    f32_params: HashMap<&'static str, Arc<AtomicF32>>,
}

/// Represents an audio channel with atomic gain, master volume, and tone stack parameters.
///
/// `Channel` uses [`AtomicF32`] for lock-free updates of audio parameters from
/// the UI thread while the audio processing thread reads them without waiting.
/// This enables low-latency parameter changes without interrupting audio playback.
///
/// The tone stack provides equalization controls for bass (low frequencies), middle (mid-range frequencies),
/// and treble (high frequencies), allowing fine-tuning of the audio signal's frequency response.
/// These parameters are also updated atomically for low-latency changes.
///
/// Effect chain is the chain of effects where the signal is put through. Effects are applied in the order they are added to the chain.
/// The `Channel` struct provides methods to add and remove effects from the chain, allowing dynamic modification of the audio processing pipeline.
///
/// Next effect_id is the unique identifier given to the next created effect in the chain.
///
/// Gain and volume are validated to be positive values (> 0.0); attempting to
/// set a negative or zero value will panic.
///
/// Tone stack values are validated to be between 0.0 and 1.0; attempting to set a value outside this range will panic.
pub struct Channel {
    id: u32,
    name: String,
    gain: Arc<AtomicF32>,
    tone_stack: Arc<ToneStack>,
    volume: Arc<AtomicF32>,
    effect_chain: Arc<Mutex<Vec<Box<dyn Effect>>>>,
    /// Retained per-effect Arc handles indexed by effect id.
    /// Stays populated even after `take_effect_chain` moves the effects to the audio thread.
    effect_handles: HashMap<u32, EffectHandles>,
    next_effect_id: u32,
}

impl Channel {
    /// Creates a new `Channel` with the given name and optional gain/master volume values.
    ///
    /// If `gain` or `master_volume` are not provided, they default to `1.0`.
    /// The tone stack parameters (bass, middle, treble) are initialized to `1.0`.
    /// The `effect chain` is initialized as an empty vector, and the next effect ID starts at `0`.
    ///
    /// # Arguments
    ///
    /// * `name` - A human-readable name for the channel (e.g., "Main", "Overdrive").
    /// * `gain` - Optional initial gain value. Defaults to `1.0` if `None`.
    /// * `master_volume` - Optional initial master volume value. Defaults to `1.0` if `None`.
    pub fn new(id: u32, name: String, gain: Option<f32>, volume: Option<f32>) -> Self {
        let gain = gain.unwrap_or(1.0);
        let volume = volume.unwrap_or(1.0);

        Self {
            id,
            name,
            gain: Arc::new(AtomicF32::new(gain)),
            tone_stack: Arc::new(ToneStack::new()),
            volume: Arc::new(AtomicF32::new(volume)),
            effect_chain: Arc::new(Mutex::new(Vec::new())),
            effect_handles: HashMap::new(),
            next_effect_id: 0,
        }
    }

    // ── Gain ─────────────────────────────────────────────────────────────────

    /// Sets the gain value for this channel.
    ///
    /// The gain value is atomically updated and will be read by the audio processing
    /// thread on the next sample cycle.
    ///
    /// # Arguments
    ///
    /// * `gain` - The new gain value. Must be positive (> 0.0).
    ///
    /// # Panics
    ///
    /// Panics if `gain` is negative or zero.
    pub fn set_gain(&self, gain: f32) {
        if gain.is_sign_positive() {
            self.gain.store(gain, Ordering::Relaxed);
        } else {
            error!("Gain must be a positive number");
            panic!("Gain must be positive");
        }
    }

    /// Returns a cloned [`Arc`] to the atomic gain value.
    ///
    /// Allows independent threads to share and read/write the gain parameter
    /// without contention.
    pub fn gain(&self) -> Arc<AtomicF32> {
        Arc::clone(&self.gain)
    }

    // ── Tone stack ────────────────────────────────────────────────────────────

    /// Sets the tone stack parameters from a [`ToneStackDto`].
    ///
    /// The bass, middle, and treble values in the DTO should be between 0.0 and 1.0.
    ///
    /// # Arguments
    ///
    /// * `tone_stack` - The tone stack data transfer object containing the new values.
    ///
    /// # Panics
    ///
    /// Panics if any value is outside the valid range.
    pub fn set_tone_stack(&self, tone_stack: ToneStackDto) {
        self.tone_stack.set_bass(tone_stack.bass);
        self.tone_stack.set_middle(tone_stack.middle);
        self.tone_stack.set_treble(tone_stack.treble);
    }

    /// Sets the bass level for the tone stack.
    ///
    /// The input value is expected to be in the range 0-100 and is internally scaled to 0-1.
    ///
    /// # Arguments
    ///
    /// * `bass` - The bass level (0-100).
    ///
    /// # Panics
    ///
    /// Panics if the scaled value is not between 0.0 and 1.0.
    pub fn set_bass(&self, bass: f32) {
        self.tone_stack.set_bass(bass / 100.0);
    }

    /// Sets the middle level for the tone stack.
    ///
    /// The input value is expected to be in the range 0-100 and is internally scaled to 0-1.
    ///
    /// # Arguments
    ///
    /// * `middle` - The middle level (0-100).
    ///
    /// # Panics
    ///
    /// Panics if the scaled value is not between 0.0 and 1.0.
    pub fn set_middle(&self, middle: f32) {
        self.tone_stack.set_middle(middle / 100.0);
    }

    /// Sets the treble level for the tone stack.
    ///
    /// The input value is expected to be in the range 0-100 and is internally scaled to 0-1.
    ///
    /// # Arguments
    ///
    /// * `treble` - The treble level (0-100).
    ///
    /// # Panics
    ///
    /// Panics if the scaled value is not between 0.0 and 1.0.
    pub fn set_treble(&self, treble: f32) {
        self.tone_stack.set_treble(treble / 100.0);
    }

    /// Returns a cloned [`Arc`] to the tone stack.
    ///
    /// Allows independent threads to access the tone stack parameters for audio processing.
    pub fn tone_stack(&self) -> Arc<ToneStack> {
        Arc::clone(&self.tone_stack)
    }

    // ── Volume ────────────────────────────────────────────────────────────────

    /// Sets the volume for this channel.
    ///
    /// #Arguments
    ///
    /// * `volume` - The volume level (must be positive)
    ///
    /// # Panics
    ///
    /// Panics if the volume is negative.
    pub fn set_volume(&self, volume: f32) {
        if volume.is_sign_positive() {
            self.volume.store(volume, Ordering::Relaxed);
        } else {
            error!("Volume must be a positive number");
            panic!("Volume must be positive");
        }
    }

    /// Returns a cloned [`Arc`] to the atomic volume value.
    ///
    /// Allows independent threads to share and read/write the volume parameter without contention.
    pub fn volume(&self) -> Arc<AtomicF32> {
        Arc::clone(&self.volume)
    }

    // ── Metadata ──────────────────────────────────────────────────────────────

    /// Sets the name of the Channel
    ///
    /// # Arguments
    ///
    /// * `name` - The name
    pub fn set_name(&mut self, name: String) {
        self.name = name;
    }

    /// Returns the name of the channel.
    pub fn name(&self) -> &String {
        &self.name
    }

    /// Returns the unique identifier of the channel.
    pub fn id(&self) -> u32 {
        self.id
    }

    // ── Effect chain ──────────────────────────────────────────────────────────

    /// Returns an `Arc<Mutex<Vec<Box<dyn Effect>>>>` representing the effect chain for this channel.
    pub fn effect_chain(&self) -> Arc<Mutex<Vec<Box<dyn Effect>>>> {
        Arc::clone(&self.effect_chain)
    }

    /// Adds an effect, capturing its shared atomic handles so commands can reach
    /// them after the chain has been moved to the audio thread.
    ///
    /// No downcasting — every effect self-reports its parameters via
    /// [`Effect::f32_params`](crate::domain::effect::Effect::f32_params).
    pub fn add_effect_to_chain(&mut self, effect: Box<dyn Effect>) {
        info!(
            "Added effect '{}' (id={}) to chain",
            effect.name(),
            effect.id()
        );

        self.effect_handles.insert(
            effect.id(),
            EffectHandles {
                is_active: effect.active_flag(),
                f32_params: effect.f32_params(),
            },
        );

        if let Ok(mut chain) = self.effect_chain.lock() {
            chain.push(effect);
            self.next_effect_id += 1;
        } else {
            error!("Failed to lock effect chain for adding effect");
        }
    }

    /// Removes an effect from the channel's effect chain by its unique identifier.
    ///
    /// If the effect is found and removed, an informational message is logged. If the effect is not found, an error message is logged.
    ///
    /// # Arguments
    ///
    /// * `effect_id` - The unique identifier of the effect to remove from the chain
    pub fn remove_effect_from_chain(&mut self, effect_id: u32) {
        if let Ok(mut chain) = self.effect_chain.lock() {
            if let Some(pos) = chain.iter().position(|e| e.id() == effect_id) {
                info!("Removed effect: {} from chain", chain[pos].name());
                chain.remove(pos);
                self.effect_handles.remove(&effect_id);
            } else {
                error!("Effect with id {} not found in chain", effect_id);
            }
        }
    }

    /// Returns the next available unique identifier for an effect in this channel's effect chain.
    pub fn next_effect_id(&self) -> u32 {
        self.next_effect_id
    }

    // ── Effect controls (written from command handlers) ───────────────────────

    /// Toggles the active state of an effect.
    ///
    /// Enables or disables audio processing for a specific effect in this channel's
    /// effect chain. The change takes effect immediately on the audio thread (lock-free).
    ///
    /// # Arguments
    /// * `effect_id` — Unique identifier of the effect to toggle
    ///
    /// # Returns
    /// * `Ok(bool)` — The new active state (`true` = now active, `false` = now bypassed)
    /// * `Err(String)` — Error message if effect ID not found in this channel
    pub fn toggle_effect(&self, effect_id: u32) -> Result<bool, String> {
        let h = self
            .effect_handles
            .get(&effect_id)
            .ok_or_else(|| format!("No effect with id {effect_id}"))?;
        let next = !h.is_active.load(Ordering::Relaxed);
        h.is_active.store(next, Ordering::Relaxed);
        info!(
            "Effect {effect_id} → {}",
            if next { "active" } else { "bypassed" }
        );
        Ok(next)
    }

    /// # Sets a Named Float32 Parameter on an Effect
    /// Generic parameter update mechanism for effect settings. Parameters are identified
    /// by string names and stored as lock-free atomics (`Arc<AtomicF32>`).
    ///
    /// ## Lock-Free Operation
    ///
    /// Uses `Ordering::Relaxed` atomic store — no synchronisation overhead:
    /// - Write happens immediately on the calling thread
    /// - Audio thread reads the updated value on next sample
    /// - No locks or condition variables
    ///
    /// ## Parameter Discovery
    ///
    /// Parameters are exposed via `Effect::f32_params()` which returns a HashMap.
    ///
    /// # Arguments
    /// * `effect_id` — ID of the effect to modify
    /// * `param` — Parameter name string (e.g., `"threshold"`, `"level"`)
    /// * `value` — New parameter value as `f32`
    ///
    /// # Returns
    /// * `Ok(())` — Parameter updated successfully
    /// * `Err(String)` — Error if:
    ///   - Effect with given ID not found
    ///   - Parameter name not recognised by the effect
    pub fn set_effect_param(&self, effect_id: u32, param: &str, value: f32) -> Result<(), String> {
        let h = self
            .effect_handles
            .get(&effect_id)
            .ok_or_else(|| format!("No effect with id {effect_id}"))?;
        let arc = h
            .f32_params
            .get(param)
            .ok_or_else(|| format!("Effect {effect_id} has no param '{param}'"))?;
        arc.store(value, Ordering::Relaxed);
        info!("Effect {effect_id} param '{param}' → {value:.4}");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod success_path {
        use super::*;

        #[test]
        fn gain_set_to_positive_value_should_succeed() {
            let channel = Channel::new(1, "Test".to_string(), None, None);
            channel.set_gain(0.5);
            assert_eq!(channel.gain().load(Ordering::Relaxed), 0.5);
        }

        #[test]
        fn volume_set_to_positive_value_should_succeed() {
            let channel = Channel::new(1, "Test".to_string(), None, None);
            channel.set_volume(0.5);
            assert_eq!(channel.volume().load(Ordering::Relaxed), 0.5);
        }

        #[test]
        fn toggle_effect_flips_active_state() {
            let channel = Channel::new(0, "Test".to_string(), None, None);
            let was = channel.effect_handles[&6].is_active.load(Ordering::Relaxed);
            let next = channel.toggle_effect(6).unwrap();
            assert_eq!(next, !was);
        }

        #[test]
        fn set_effect_param_updates_threshold() {
            let channel = Channel::new(0, "Test".to_string(), None, None);
            channel.set_effect_param(6, "threshold", 0.3).unwrap();
            let v = channel.effect_handles[&6].f32_params["threshold"].load(Ordering::Relaxed);
            assert!((v - 0.3).abs() < 1e-6);
        }

        #[test]
        fn adding_effect_to_effect_chain_should_add_an_effect_to_effect_chain() {
            let mut channel = Channel::new(1, "Test".to_string(), None, None);
            let effect_id = channel.next_effect_id();

            channel.add_effect_to_chain(Box::new(HCDistortion::new(
                effect_id,
                "Test Effect".to_string(),
                false,
                0.5,
                0.0,
                "#e67e22".to_string(),
            )));

            let chain = channel.effect_chain.lock().unwrap();
            assert_eq!(chain.len(), 1);
        }

        #[test]
        fn removing_effect_from_effect_chain_should_remove_an_effect_from_effect_chain() {
            let mut channel = Channel::new(1, "Test".to_string(), None, None);
            let effect_id = channel.next_effect_id();
            let effect = Box::new(HCDistortion::new(
                effect_id,
                "Test Effect".to_string(),
                false,
                0.5,
                0.0,
                "#e67e22".to_string(),
            ));

            channel.add_effect_to_chain(effect);

            {
                let chain_before = channel.effect_chain.lock().unwrap();
                assert_eq!(chain_before.len(), 1);
            }

            channel.remove_effect_from_chain(effect_id);

            let chain_after = channel.effect_chain.lock().unwrap();
            assert_eq!(chain_after.len(), 0);
            assert!(!channel.effect_handles.contains_key(&effect_id));
        }
    }

    mod failure_path {
        use super::*;

        #[test]
        #[should_panic(expected = "Gain must be positive")]
        fn gain_set_to_negative_value_should_panic() {
            let channel = Channel::new(1, "Test".to_string(), None, None);
            channel.set_gain(-0.5);
        }

        #[test]
        #[should_panic(expected = "Volume must be positive")]
        fn volume_set_to_negative_value_should_panic() {
            let channel = Channel::new(1, "Test".to_string(), None, None);
            channel.set_volume(-0.5);
        }

        #[test]
        fn toggle_unknown_effect_returns_err() {
            let channel = Channel::new(1, "Test".to_string(), None, None);
            assert!(channel.toggle_effect(999).is_err());
        }

        #[test]
        fn removing_invalid_effect_id_should_not_remove_any_effect() {
            let mut channel = Channel::new(1, "Test".to_string(), None, None);
            let effect_id = channel.next_effect_id();
            let effect = Box::new(HCDistortion::new(
                effect_id,
                "Test Effect".to_string(),
                false,
                0.5,
                0.0,
                "#e67e22".to_string(),
            ));

            channel.add_effect_to_chain(effect);

            let len_before = channel.effect_chain.lock().unwrap().len();
            channel.remove_effect_from_chain(effect_id + 1);

            let len_after = channel.effect_chain.lock().unwrap().len();
            assert_eq!(len_before, len_after);
        }
    }
}
