use crate::domain::audio_processor::AudioProcessor;
use crate::domain::dto::effect::effect_dto::EffectDto;
use crate::domain::dto::effect::scdistortion_dto::ScDistortionDto;
use crate::domain::effect::Effect;
use crate::services::processors::gain::gain_processor::GainProcessor;
use atomic_float::AtomicF32;
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

pub struct SCDistortion {
    id: u32,
    name: String,
    is_active: Arc<AtomicBool>,
    /// Clip level in `[0.0, 1.0]`. Lower = heavier distortion.
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
        Self {
            id,
            name,
            is_active: Arc::new(AtomicBool::new(is_active)),
            limit: Arc::new(AtomicF32::new(threshold.clamp(0.001, 1.0))),
            level: level_arc,
            level_gain,
            smoothing: Arc::new(AtomicF32::new(smoothing.clamp(1.0, 10.0))),
            color,
        }
    }

    /// Returns the current clipping threshold in range `[0.0, 1.0]`.
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

    pub fn smoothing(&self) -> f32 {
        self.smoothing.load(Ordering::Relaxed)
    }

    pub fn set_smoothing(&self, smoothing: f32) {
        self.smoothing
            .store(smoothing.clamp(1.0, 10.0), Ordering::Relaxed);
    }
}

impl AudioProcessor for SCDistortion {
    /// Processes a single audio sample through soft clipping and level boost.
    ///
    /// # Algorithm
    ///
    /// 1. **Load clipping threshold & the smoothing factor** atomically (lock-free)
    /// 2. **Normalize the sample** to the clipping threshold. Needs to be done because smoothing algorithm smooths towards 1.0 and -1.0
    /// 3. **Smooth the curve** towards the limit
    /// 4. **Denormalize the sample** to get back to the desired amplitude
    /// 5. **Apply gain boost** via the [`GainProcessor`] with smoothed transitions
    ///
    /// # Parameters
    ///
    /// * `sample` — audio sample, typically `-1.0` to `1.0`
    ///
    /// # Returns
    /// Processed sample: clipped, smoothed and boosted by the level knob
    fn process(&mut self, sample: f32) -> f32 {
        let limit = self.limit.load(Ordering::Relaxed);
        let smoothing = self.smoothing.load(Ordering::Relaxed);
        let normalized_sample = sample / limit;
        let abs_normalized_sample = normalized_sample.abs();
        let smoothed =
            normalized_sample / (1.0 + abs_normalized_sample.powf(smoothing)).powf(1.0 / smoothing);
        let denormalized_sample = smoothed * limit;
        self.level_gain.process(denormalized_sample)
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
    /// Converts this effect into its serialisable DTO representation.
    ///
    /// Called when sending effect state to the frontend or external clients.
    ///
    /// # Returns
    ///
    /// [`EffectDto::SCDistortion`] with all current parameters
    fn to_dto(&self) -> EffectDto {
        EffectDto::SCDistortion(ScDistortionDto {
            id: self.id,
            name: self.name.clone(),
            is_active: self.is_active.load(Ordering::Relaxed),
            color: self.color.clone(),
            threshold: self.limit.load(Ordering::Relaxed),
            level: self.level(),
            smoothing: self.smoothing.load(Ordering::Relaxed),
        })
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
}

#[cfg(test)]
mod tests {
    use super::*;

    fn distortion(threshold: f32, smoothing: f32) -> SCDistortion {
        SCDistortion::new(
            0,
            "SC".to_string(),
            true,
            threshold,
            0.0,
            smoothing,
            "#e67e22".to_string(),
        )
    }

    mod success_path {
        use super::*;

        #[test]
        fn sample_within_threshold_is_slightly_compressed() {
            let mut fx = distortion(1.0, 1.0);
            for _ in 0..10_000 {
                fx.process(0.0);
            }

            let input = 0.1;
            let output = fx.process(input);

            assert!(output < input);
            assert!((output - input).abs() < 0.01);
        }

        #[test]
        fn sample_is_pushed_towards_limit() {
            let limit = 0.5;
            let mut fx = distortion(limit, 5.0);
            for _ in 0..10_000 {
                fx.process(0.0);
            }

            let out = fx.process(100.0);

            assert!(out <= limit);
            assert!((out - limit).abs() < 0.01);
        }

        #[test]
        fn smoothing_parameter_affects_curve() {
            let mut soft_fx = distortion(1.0, 1.0); // n=1 (Very soft)
            let mut hard_fx = distortion(1.0, 10.0); // n=10 (Harder)

            for _ in 0..10_000 {
                soft_fx.process(0.0);
                hard_fx.process(0.0);
            }

            let input = 0.5;
            let soft_out = soft_fx.process(input);
            let hard_out = hard_fx.process(input);

            assert!(soft_out < hard_out);
        }

        #[test]
        fn process_if_active_passes_through_when_inactive() {
            let mut fx = distortion(0.5, 10.0);
            fx.set_active(false);
            assert_eq!(fx.process_if_active(0.9), 0.9);
        }

        #[test]
        fn set_threshold_updates_clip_level() {
            let mut fx = distortion(0.8, 10.0);

            fx.set_threshold(0.3);
            assert!((fx.threshold() - 0.3).abs() < 1e-6);

            for _ in 0..10_000 {
                fx.process(0.0);
            }

            let output = fx.process(0.9);

            assert!(output < 0.9);
            assert!(
                (output - 0.3).abs() < 0.05,
                "Expected output to be near 0.3, got {}",
                output
            );

            let massive_input = 100.0;
            let limited_output = fx.process(massive_input);
            assert!(limited_output <= 0.30001);
        }

        #[test]
        fn level_boost_doubles_output_at_max() {
            let mut fx = SCDistortion::new(
                0,
                "HC".to_string(),
                true,
                1.0,
                1.0,
                10.0,
                "#e67e22".to_string(),
            );
            // Converge gain processor to ×2.0
            for _ in 0..20_000 {
                fx.process(0.0);
            }
            let out = fx.process(0.3);
            assert!((out - 0.6).abs() < 0.01, "expected ≈0.6, got {out}");
        }

        #[test]
        fn level_unity_at_zero() {
            let mut fx = distortion(1.0, 10.0); // level=0.0
            for _ in 0..10_000 {
                fx.process(0.0);
            }
            let out = fx.process(0.4);
            assert!((out - 0.4).abs() < 0.01, "expected ≈0.4, got {out}");
        }
    }

    mod failure_path {
        use super::*;
        #[test]
        fn threshold_above_one_is_clamped_to_one() {
            let fx = distortion(2.0, 1.0);
            assert_eq!(fx.threshold(), 1.0);
        }

        #[test]
        fn threshold_of_zero_is_clamped_to_minimum() {
            let fx = distortion(0.0, 1.0);
            assert!(fx.threshold() > 0.0);
        }

        #[test]
        fn smoothing_above_ten_is_clamped_to_ten() {
            let fx = distortion(1.0, 11.0);
            assert_eq!(fx.smoothing(), 10.0);
        }

        #[test]
        fn smoothing_of_zero_is_clamped_to_minimum() {
            let fx = distortion(1.0, 0.0);
            assert!(fx.smoothing() > 0.0);
        }
    }
}
