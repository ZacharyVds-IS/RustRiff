use crate::domain::dto::tone_stack_dto::ToneStackDto;
use crate::domain::effect::Effect;
use crate::domain::tone_stack::ToneStack;
use crate::services::effects::distortion::hc_distortion::HCDistortion;
use crate::services::effects::flip_effect::FlipEffect;
use crate::services::processors::gain::gain_processor::GainProcessor;
use atomic_float::AtomicF32;
use std::mem::take;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use tracing::{error, info};

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
/// Gain and master volume are validated to be positive values (> 0.0); attempting to
/// set a negative or zero value will panic.
///
/// Tone stack values are validated to be between 0.0 and 1.0; attempting to set a value outside this range will panic.
pub struct Channel {
    id: u32,
    name: String,
    gain: Arc<AtomicF32>,
    tone_stack: Arc<ToneStack>,
    volume: Arc<AtomicF32>,
    effect_chain: Vec<Box<dyn Effect>>,
}

impl Channel {
    /// Creates a new `Channel` with the given name and optional gain/master volume values.
    ///
    /// If `gain` or `master_volume` are not provided, they default to `1.0`.
    /// The tone stack parameters (bass, middle, treble) are initialized to `1.0`.
    ///
    /// # Arguments
    ///
    /// * `name` - A human-readable name for the channel (e.g., "Main", "Overdrive").
    /// * `gain` - Optional initial gain value. Defaults to `1.0` if `None`.
    /// * `master_volume` - Optional initial master volume value. Defaults to `1.0` if `None`.
    pub fn new(id: u32, name: String, gain: Option<f32>, volume: Option<f32>) -> Self {
        let gain = gain.unwrap_or(1.0);
        let volume = volume.unwrap_or(1.0);

        let mut channel = Self {
            id,
            name,
            gain: Arc::new(AtomicF32::new(gain)),
            tone_stack: Arc::new(ToneStack::new()),
            volume: Arc::new(AtomicF32::new(volume)),
            effect_chain: Vec::new(),
        };

        //this is temp to test effects in the chain UI
        if id == 0 {
            channel.add_effect_to_chain(Box::new(FlipEffect::new(
                5,
                "Flipper".to_string(),
                "#21CC00".to_string(),
            )));
            channel.add_effect_to_chain(Box::new(HCDistortion::new(
                6,
                "Distortion".to_string(),
                false,
                0.5,
                "#e62cdc".to_string(),
            )));
        }
        channel
    }

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

    /// Sets the name of the Channel
    ///
    /// # Arguments
    ///
    /// * `name` - The name
    pub fn set_name(&mut self, name: String) {
        self.name = name;
    }

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

    /// Returns a cloned [`Arc`] to the atomic gain value.
    ///
    /// Allows independent threads to share and read/write the gain parameter
    /// without contention.
    pub fn gain(&self) -> Arc<AtomicF32> {
        Arc::clone(&self.gain)
    }

    /// Returns a cloned [`Arc`] to the tone stack.
    ///
    /// Allows independent threads to access the tone stack parameters for audio processing.
    pub fn tone_stack(&self) -> Arc<ToneStack> {
        Arc::clone(&self.tone_stack)
    }

    /// Returns the name of the channel.
    pub fn name(&self) -> &String {
        &self.name
    }

    /// Returns a cloned [`Arc`] to the atomic volume value.
    ///
    /// Allows independent threads to share and read/write the volume parameter without contention.
    pub fn volume(&self) -> Arc<AtomicF32> {
        Arc::clone(&self.volume)
    }

    /// Returns the unique identifier of the channel.
    pub fn id(&self) -> u32 {
        self.id
    }

    /// Returns a reference to the effect chain for this channel.
    pub fn effect_chain(&self) -> &[Box<dyn Effect>] {
        &self.effect_chain
    }

    /// Takes ownership of the effect chain, replacing it with an empty vector.
    /// This is useful for transferring the chain to another component without cloning.
    pub fn take_effect_chain(&mut self) -> Vec<Box<dyn Effect>> {
        take(&mut self.effect_chain)
    }

    /// Adds an effect to the end of the channel's effect chain.
    ///
    /// # Arguments
    ///
    /// * `effect` - The effect to add to the chain.  Must implement the `Effect` trait.
    pub fn add_effect_to_chain(&mut self, effect: Box<dyn Effect>) {
        info!("Added effect: {} to chain", effect.name());
        self.effect_chain.push(effect);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(test)]
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
    }

    #[cfg(test)]
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
    }
}
