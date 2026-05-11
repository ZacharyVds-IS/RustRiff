use crate::domain::audio_processor::AudioProcessor;
use crate::domain::dto::effect::effect_dto::EffectDto;
use crate::domain::dto::effect::hcdistortion_dto::HcDistortionDto;
use crate::domain::effect::Effect;
use crate::services::processors::gain::gain_processor::GainProcessor;
use atomic_float::AtomicF32;
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

/// # Hard-Clipping Distortion Effect
///
/// `HCDistortion` implements a classic hard-clipping distortion pedal with two
/// controllable parameters: **Drive** (clipping threshold) and **Level** (output boost).
///
/// ## Signal Chain
///
/// The processing happens in two stages:
///
/// 1. **Hard Clipping**
///    - Any sample whose absolute value exceeds the `threshold` is clamped to `±threshold`
///    - This produces the characteristic flat-top waveform of hard clipping distortion
///    - Lower thresholds produce heavier distortion (more clipping)
///    - Higher thresholds produce lighter distortion (less clipping)
///
/// 2. **Output Level Boost** (via [`GainProcessor`])
///    - After clipping, the signal passes through a [`GainProcessor`]
///    - The gain is controlled by a normalised `level` parameter `[0.0, 1.0]`
///    - Maps to a linear gain range of `1.0` (unity) to `2.0` (double amplitude)
///    - Uses smoothed transitions (one-pole filter) to avoid clicks and pops
///
/// ## Parameter Ranges
///
/// | Parameter  | Range      | UI Display | Effect |
/// |-----------|----------|------------|--------|
/// | `threshold` | `(0.0, 1.0]` | Drive 0–100% | Lower = heavier distortion |
/// | `level`  | `[0.0, 1.0]`   | Level 0–100% | 0 = no boost, 1.0 = ×2 boost |
///
/// ## Thread-Safe Atomic Updates
///
/// All mutable parameters are stored as lock-free atomics:
/// - `is_active`: [`Arc<AtomicBool>`] — enable/bypass the effect
/// - `limit`: [`Arc<AtomicF32>`] — clipping threshold (shared with [`f32_params`](Self::f32_params))
/// - `level`: [`Arc<AtomicF32>`] — internal gain value `[1.0, 2.0]` (shared with [`GainProcessor`])
///
/// This allows the audio thread to read parameter changes from command handlers
/// without any locks or synchronisation overhead.
pub struct HCDistortion {
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
    /// UI chassis colour (hex string, e.g. `"#e67e22"`).
    color: String,
}

impl HCDistortion {
    /// Creates a new `HCDistortion` effect.
    ///
    /// # Parameters
    ///
    /// * `id` — Unique identifier for this effect instance
    /// * `name` — Human-readable name (e.g., "Distortion")
    /// * `is_active` — Whether the effect is initially enabled
    /// * `threshold` — Clip level in `(0.0, 1.0]`. Will be clamped to `[0.001, 1.0]`.
    ///   Lower values produce heavier distortion.
    /// * `level` — Initial output boost in `[0.0, 1.0]`. Will be clamped to `[0.0, 1.0]`.
    ///   Maps internally to gain `[1.0, 2.0]`.
    /// * `color` — Hex colour string for UI pedal chassis (e.g., `"#e67e22"`)
    pub fn new(
        id: u32,
        name: String,
        is_active: bool,
        threshold: f32,
        level: f32,
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

impl AudioProcessor for HCDistortion {
    /// Processes a single audio sample through hard clipping and level boost.
    ///
    /// # Algorithm
    ///
    /// 1. **Load clipping threshold** atomically (lock-free)
    /// 2. **Clamp sample** to `[-threshold, threshold]` (hard clipping)
    /// 3. **Apply gain boost** via the [`GainProcessor`] with smoothed transitions
    ///
    /// # Parameters
    ///
    /// * `sample` — Normalised audio sample, typically `-1.0` to `1.0`
    ///
    /// # Returns
    /// Processed sample: clipped and boosted by the level knob
    fn process(&mut self, sample: f32) -> f32 {
        let limit = self.limit.load(Ordering::Relaxed);
        let clipped = sample.clamp(-limit, limit);
        self.level_gain.process(clipped)
    }
}

impl Effect for HCDistortion {
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
        EffectDto::HCDistortion(HcDistortionDto {
            id: self.id,
            name: self.name.clone(),
            is_active: self.is_active.load(Ordering::Relaxed),
            color: self.color.clone(),
            threshold: self.limit.load(Ordering::Relaxed),
            level: self.level(),
        })
    }

    /// Processes a sample only if the effect is currently active.
    ///
    /// If inactive (bypassed), the sample is returned unchanged.
    ///
    /// # Parameters
    ///
    /// * `sample` — Input audio sample
    ///
    /// # Returns
    ///
    /// Processed sample if active, otherwise the input unchanged (unity bypass)
    fn process_if_active(&mut self, sample: f32) -> f32 {
        if self.is_active() {
            self.process(sample)
        } else {
            sample
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn distortion(threshold: f32) -> HCDistortion {
        HCDistortion::new(
            0,
            "HC".to_string(),
            true,
            threshold,
            0.0,
            "#e67e22".to_string(),
        )
    }

    mod success_path {
        use super::*;

        #[test]
        fn sample_within_threshold_is_unchanged() {
            let mut fx = distortion(0.5);
            // With level=0.0 the gain processor targets 1.0; after many samples it converges.
            // For a quick unit check, drive it to steady-state first.
            for _ in 0..10_000 {
                fx.process(0.0);
            }
            assert!((fx.process(0.3) - 0.3).abs() < 1e-3);
            assert!((fx.process(-0.3) - (-0.3)).abs() < 1e-3);
        }

        #[test]
        fn sample_above_threshold_is_clipped() {
            let mut fx = distortion(0.5);
            for _ in 0..10_000 {
                fx.process(0.0);
            }
            assert!((fx.process(0.9) - 0.5).abs() < 1e-3);
        }

        #[test]
        fn process_if_active_clips_when_active() {
            let mut fx = distortion(0.5);
            for _ in 0..10_000 {
                fx.process(0.0);
            }
            assert!((fx.process_if_active(0.9) - 0.5).abs() < 1e-3);
        }

        #[test]
        fn process_if_active_passes_through_when_inactive() {
            let mut fx = distortion(0.5);
            fx.set_active(false);
            assert_eq!(fx.process_if_active(0.9), 0.9);
        }

        #[test]
        fn set_threshold_updates_clip_level() {
            let mut fx = distortion(0.8);
            fx.set_threshold(0.3);
            assert!((fx.threshold() - 0.3).abs() < 1e-6);
            for _ in 0..10_000 {
                fx.process(0.0);
            }
            assert!((fx.process(0.9) - 0.3).abs() < 1e-3);
        }

        #[test]
        fn level_boost_doubles_output_at_max() {
            let mut fx =
                HCDistortion::new(0, "HC".to_string(), true, 1.0, 1.0, "#e67e22".to_string());
            // Converge gain processor to ×2.0
            for _ in 0..20_000 {
                fx.process(0.0);
            }
            let out = fx.process(0.3);
            assert!((out - 0.6).abs() < 0.01, "expected ≈0.6, got {out}");
        }

        #[test]
        fn level_unity_at_zero() {
            let mut fx = distortion(1.0); // level=0.0
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
            let fx = distortion(2.0);
            assert_eq!(fx.threshold(), 1.0);
        }

        #[test]
        fn threshold_of_zero_is_clamped_to_minimum() {
            let fx = distortion(0.0);
            assert!(fx.threshold() > 0.0);
        }
    }
}
