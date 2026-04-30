use crate::domain::audio_processor::AudioProcessor;

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
pub trait Effect: AudioProcessor {

    /// Returns the unique numeric identifier for this specific effect instance.
    fn id(&self) -> u32;

    /// Returns the human-readable name of the effect (e.g., "Overdrive", "Delay").
    fn name(&self) -> &str;

    /// Returns `true` if the effect is currently enabled and processing audio.
    ///
    /// If `false`, the effect should ideally be bypassed to save CPU or maintain
    /// signal transparency.
    fn is_active(&self) -> bool;

    /// Sets whether the effect is active or bypassed.
    ///
    /// * `active` - `true` to enable the effect, `false` to bypass it.
    fn set_active(&mut self, active: bool);

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