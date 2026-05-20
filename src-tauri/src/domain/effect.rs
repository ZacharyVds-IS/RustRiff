use crate::domain::audio_processor::AudioProcessor;
use crate::domain::dto::effect::effect_dto::EffectDto;
use atomic_float::AtomicF32;
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, AtomicU32};
use std::sync::Arc;
use uuid::Uuid;

/// A trait defining the shared behavior for audio effects within the signal chain.
///
/// The `Effect` trait extends [`AudioProcessor`], providing metadata and state management
/// (such as enabling or bypassing the effect) in addition to the core signal processing
/// capabilities.
///
/// # Requirements
///
/// Types implementing `Effect` must also implement `AudioProcessor`, which provides the
/// primary [`process`](AudioProcessor::process) method used for manipulating audio samples.
pub trait Effect: AudioProcessor + Send + Sync {
    /// Returns the unique numeric identifier for this specific effect instance.
    fn id(&self) -> Uuid;

    /// Returns the human-readable name of the effect (e.g., "Overdrive", "Delay").
    fn name(&self) -> &str;

    /// Returns `true` if the effect is currently enabled and processing audio.
    ///
    /// If `false`, the effect should ideally be bypassed to save CPU or maintain
    /// signal transparency.
    fn is_active(&self) -> bool {
        self.active_flag()
            .load(std::sync::atomic::Ordering::Relaxed)
    }

    /// Sets whether the effect is active or bypassed.
    ///
    /// * `active` - `true` to enable the effect, `false` to bypass it.
    fn set_active(&self, active: bool) {
        self.active_flag()
            .store(active, std::sync::atomic::Ordering::Relaxed);
    }

    /// Returns a color code (hex) associated with this effect for UI representation.
    fn get_color(&self) -> String;

    /// Converts this effect into its serialisable [`EffectDto`] representation.
    ///
    /// Each concrete effect type returns the correct variant of the tagged union,
    /// carrying its own specific parameters alongside the shared fields.
    fn to_dto(&self) -> EffectDto;

    /// Returns the shared `Arc<AtomicBool>` that drives `process_if_active`.
    /// Command handlers write to it; the audio thread reads it lock-free.
    fn active_flag(&self) -> Arc<AtomicBool>;

    /// Returns named f32 parameter Arcs shared with the audio thread.
    /// Defaults to an empty map — override for effects with extra parameters.
    ///
    /// Implementing this is the **only** change required to make a new effect's
    /// parameters controllable from commands — no downcasting anywhere.
    fn f32_params(&self) -> HashMap<&'static str, Arc<AtomicF32>> {
        HashMap::new()
    }

    /// Returns named u32 parameter Arcs shared with the audio thread.
    /// Defaults to an empty map — override for effects with extra parameters.
    ///
    /// Implementing this is the **only** change required to make a new effect's
    /// parameters controllable from commands — no downcasting anywhere.
    fn u32_params(&self) -> HashMap<&'static str, Arc<AtomicU32>> {
        HashMap::new()
    }

    /// Processes a single audio sample only if the effect is currently active.
    ///
    /// This is the primary entry point for a signal chain. If [`is_active`](Self::is_active)
    /// returns `true`, the sample is passed to the underlying `process` method.
    /// Otherwise, the input sample is returned unchanged (unity gain bypass).
    ///
    /// # Parameters
    /// * `sample`: The input floating-point audio sample
    /// # Returns
    ///
    /// The processed sample if active, or the original sample if bypassed.
    fn process_if_active(&mut self, sample: f32) -> f32 {
        if self.is_active() {
            self.process(sample)
        } else {
            sample
        }
    }
}
