use crate::domain::audio_processor::AudioProcessor;
use crate::domain::dto::effect::effect_dto::EffectDto;
use crate::domain::dto::effect::hcdistortion_dto::HcDistortionDto;
use crate::domain::effect::Effect;
use crate::services::processors::gain::gain_processor::GainProcessor;
use atomic_float::AtomicF32;
use rustfft::num_complex::ComplexFloat;
use rustfft::num_traits::pow;
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

pub struct SCDistortion {
    id: u32,
    name: String,
    is_active: Arc<AtomicBool>,
    /// Clip level in `(0.0, 1.0]`. Lower = heavier distortion.
    /// Shared with command infrastructure via [`f32_params`](Self::f32_params).
    limit: Arc<AtomicF32>,
    /// Internal gain atomic shared with `level_gain`. Stores gain in range `[1.0, 2.0]`.
    /// Accessed externally via normalised [`level`](Self::level) method.
    level: Arc<AtomicF32>,
    /// GainProcessor that applies smoothed level boost after hard clipping.
    /// Reads gain value from `level` atomic lock-free on each sample.
    level_gain: GainProcessor,
    /// Smoothing factor `[1.0,10.0]`. Lower = more smoothing
    /// Shared with command infrastructure via [`f32_params`](Self::f32_params).
    smoothing: Arc<AtomicF32>,
    /// UI chassis colour (hex string, e.g. `"#e67e22"`).
    color: String,
}

impl SCDistortion {
    pub fn new(
        id: u32,
        name: String,
        is_active: bool,
        threshold: f32,
        level: f32,
        smoothing: f32,
        color: String,
    ) -> Self {
        let gain_value = 1.0 + level.clamp(0.0, 1.0); // map [0,1] → [1,2]
        let level_arc = Arc::new(AtomicF32::new(gain_value));
        let level_gain = GainProcessor::new(Arc::clone(&level_arc));
        let smoothing_arc = Arc::new(AtomicF32::new(smoothing));
        Self {
            id,
            name,
            is_active: Arc::new(AtomicBool::new(is_active)),
            limit: Arc::new(AtomicF32::new(threshold.clamp(0.001, 1.0))),
            level: level_arc,
            level_gain,
            smoothing: smoothing_arc,
            color,
        }
    }

    /// Returns the current clipping threshold in range `(0.0, 1.0]`.
    ///
    /// # Returns
    ///
    /// The threshold value; lower values produce heavier clipping.
    pub fn threshold(&self) -> f32 {
        self.limit.load(Ordering::Relaxed)
    }

    /// Sets the clipping threshold. Value is clamped to `[0.001, 1.0]`.
    ///
    /// The change takes effect on the very next audio sample — no synchronisation needed.
    ///
    /// # Parameters
    ///
    /// * `threshold` — New clipping level in `(0.0, 1.0]`
    pub fn set_threshold(&self, threshold: f32) {
        self.limit
            .store(threshold.clamp(0.001, 1.0), Ordering::Relaxed);
    }

    /// Returns the normalised output level in range `[0.0, 1.0]`.
    ///
    /// Internally the gain is stored as `[1.0, 2.0]`; this method reverses that mapping
    /// to give the external normalised value.
    ///
    /// # Returns
    ///
    /// Normalised level: `0.0` = no boost (unity gain), `1.0` = ×2.0 boost
    pub fn level(&self) -> f32 {
        (self.level.load(Ordering::Relaxed) - 1.0).clamp(0.0, 1.0)
    }

    /// Sets the output level from a normalised value `[0.0, 1.0]`.
    ///
    /// Internally maps to gain `[1.0, 2.0]` and stores it in the atomic.
    /// The change takes effect on the very next audio sample — no synchronisation needed.
    ///
    /// # Parameters
    ///
    /// * `level` — Normalised level in `[0.0, 1.0]`. Will be clamped.
    pub fn set_level(&self, level: f32) {
        self.level
            .store(1.0 + level.clamp(0.0, 1.0), Ordering::Relaxed);
    }
}

impl AudioProcessor for SCDistortion {
    fn process(&mut self, sample: f32) -> f32 {
        let limit = self.limit.load(Ordering::Relaxed);
        let smoothing = self.smoothing.load(Ordering::Relaxed);
        let abs_sample= sample.abs();
        let distorted = (limit * sample) / pow(1.0 + pow(abs_sample, smoothing) ,1.0/smoothing);
        self.level_gain.process(distorted)
    }
}

impl Effect for SCDistortion {
    fn id(&self) -> u32 {
        self.id
    }
    fn name(&self) -> &str {
        &self.name
    }
    fn get_color(&self) -> String {
        self.color.clone()
    }
    fn active_flag(&self) -> Arc<AtomicBool> {
        Arc::clone(&self.is_active)
    }

    /// Returns a map of named f32 parameters for command infrastructure.
    ///
    /// This enables the generic command dispatcher to update effect parameters
    /// without needing to know about specific effect types.
    ///
    /// # Returns
    ///
    /// HashMap with keys:
    /// * `"threshold"` — points to `limit` atomic; external code can write new thresholds
    /// * `"level"` — points to internal gain atomic `[1.0, 2.0]`
    ///
    /// # Note
    ///
    /// The `"level"` key stores the raw gain value. Command handlers should convert
    /// the external normalised `[0, 1]` range to internal gain `[1, 2]` before writing.
    fn f32_params(&self) -> HashMap<&'static str, Arc<AtomicF32>> {
        let mut map = HashMap::new();
        map.insert("threshold", Arc::clone(&self.limit));
        // "level" stores the internal gain [1, 2]; the command converts [0,1] before writing.
        map.insert("level", Arc::clone(&self.level));
        map.insert("smoothing", Arc::clone(&self.smoothing));
        map
    }

    /// Converts this effect into its serialisable DTO representation.
    ///
    /// Called when sending effect state to the frontend or external clients.
    ///
    /// # Returns
    ///
    /// [`EffectDto::HCDistortion`] with all current parameters
    fn to_dto(&self) -> EffectDto {
        //TODO: Convert to SCDistortionDto
        EffectDto::HCDistortion(HcDistortionDto {
            id: self.id,
            name: self.name.clone(),
            is_active: self.is_active.load(Ordering::Relaxed),
            color: self.color.clone(),
            threshold: self.limit.load(Ordering::Relaxed),
            level: self.level(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn distortion(threshold: f32) -> SCDistortion {
        SCDistortion::new(
            0,
            "SC".to_string(),
            true,
            threshold,
            0.0,
            1.0,
            "#e67e22".to_string(),
        )
    }

    mod success_path {}

    mod failure_path {}
}
