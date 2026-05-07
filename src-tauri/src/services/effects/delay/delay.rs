use crate::domain::audio_processor::AudioProcessor;
use crate::domain::dto::effect::delay_dto::DelayDto;
use crate::domain::dto::effect::effect_dto::EffectDto;
use crate::domain::effect::Effect;
use atomic_float::AtomicF32;
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::Arc;

pub struct Delay {
    id: u32,
    name: String,
    is_active: Arc<AtomicBool>,
    color: String,
    delay_time: Arc<AtomicU32>, //20ms - 800ms
    level: Arc<AtomicF32>,      //0.0-0.95
    delay_buffer: Vec<f32>,
    write_pos: usize,
    sample_rate: u32,
    delay_in_samples: usize,
    last_feedback_output: f32
}

const MAX_DELAY_TIME_FLOAT: f32 = 800.0;

impl Delay {
    pub fn new(
        id: u32,
        name: String,
        is_active: bool,
        color: String,
        sample_rate: u32,
        delay_time: u32,
        level: f32,
    ) -> Self {
        let level_arc = Arc::new(AtomicF32::new(level.clamp(0.0, 0.95)));
        let delay_time_arc = Arc::new(AtomicU32::new(delay_time.clamp(20, 800)));

        let max_samples = (MAX_DELAY_TIME_FLOAT * sample_rate as f32 / 1000.0) as usize;
        let delay_buffer = vec![0.0; max_samples + 1];

        let mut instance = Self {
            id,
            name,
            is_active: Arc::new(AtomicBool::new(is_active)),
            color,
            delay_time: delay_time_arc,
            level: level_arc,
            delay_buffer,
            write_pos: 0,
            sample_rate,
            delay_in_samples: 0,
            last_feedback_output: 0.0,
        };

        instance.calc_delay_in_samples();
        instance
    }

    ///Calculates delay in samples with the given delay time and sample rate.
    ///
    /// This also sets the delay in samples to the calculated value
    fn calc_delay_in_samples(&mut self) {
        self.delay_in_samples = (self.delay_time.load(Ordering::Relaxed) as f32
            * self.sample_rate as f32
            / 1000.0) as usize;
    }

    // GETTERS
    pub fn delay_time(&self) -> &Arc<AtomicU32> {
        &self.delay_time
    }

    pub fn level(&self) -> &Arc<AtomicF32> {
        &self.level
    }

    pub fn delay_buffer(&self) -> &Vec<f32> {
        &self.delay_buffer
    }

    pub fn write_pos(&self) -> usize {
        self.write_pos
    }

    pub fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    // SETTERS
    /// Sets the delay time (ms) to the new value clamped between [20,800]
    ///
    /// Calls `Delay::calc_delay_in_samples` to recalculate the delay in samples
    pub fn set_delay_time(&mut self, delay_time: u32) {
        self.delay_time
            .store(delay_time.clamp(20, 800), Ordering::Relaxed);
        self.calc_delay_in_samples()
    }

    /// Sets the level to the new value clamped between [0.0,0.95]
    pub fn set_level(&mut self, level: f32) {
        self.level.store(level.clamp(0.0, 0.95), Ordering::Relaxed);
    }

    /// Sets the sample rate (Hz) to the new value
    ///
    /// Calls `Delay::calc_delay_in_samples` to recalculate the delay in samples and
    /// resizes the delay_buffer to accommodate for the new sample rate
    pub fn set_sample_rate(&mut self, sample_rate: u32) {
        self.sample_rate = sample_rate;
        let max_samples = (MAX_DELAY_TIME_FLOAT * sample_rate as f32 / 1000.0) as usize;
        self.delay_buffer.resize(max_samples + 1, 0.0);
        self.calc_delay_in_samples();
    }
}

impl AudioProcessor for Delay {
    /// Processes a single audio sample through a feedback delay line with linear interpolation.
    ///
    /// The signal flow follows these steps:
    /// 1.  **Interpolated Read**: Calculates a fractional read position to prevent "stepping"
    ///     artifacts when delay time changes.
    /// 2.  **Damping**: Applies a simple One-Pole Low-Pass Filter to the delayed signal.
    /// 3.  **Feedback**: Mixes the filtered delayed signal back into the write head.
    /// 4.  **Output Mix**: Combines the original (dry) signal with the delayed (wet) signal.
    fn process(&mut self, sample: f32) -> f32 {
        if self.delay_buffer.is_empty() {
            return sample;
        }

        let delay_ms = self.delay_time.load(Ordering::Relaxed) as f32;
        let feedback_amount = self.level.load(Ordering::Relaxed);

        let target_delay_samples = (delay_ms * self.sample_rate as f32 / 1000.0);
        let buf_len = self.delay_buffer.len() as f32;
        let read_pos = (self.write_pos as f32 - target_delay_samples + buf_len) % buf_len;

        let i_part = read_pos.floor() as usize;
        let f_part = read_pos - i_part as f32;
        let next_i = (i_part + 1) % self.delay_buffer.len();

        let delayed_sample = self.delay_buffer[i_part] * (1.0 - f_part) + self.delay_buffer[next_i] * f_part;


        let filtered_feedback = (delayed_sample * 0.5) + (self.last_feedback_output * 0.5);
        self.last_feedback_output = filtered_feedback;

        self.delay_buffer[self.write_pos] = sample + (filtered_feedback * feedback_amount);

        self.write_pos = (self.write_pos + 1) % self.delay_buffer.len();


        sample + (delayed_sample * feedback_amount)
    }
}

impl Effect for Delay {
    fn id(&self) -> u32 {
        self.id
    }

    fn name(&self) -> &str {
        self.name.as_str()
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
    /// [`EffectDto::Delay`] with all current parameters
    fn to_dto(&self) -> EffectDto {
        EffectDto::Delay(DelayDto {
            id: self.id(),
            name: self.name.clone(),
            is_active: self.is_active(),
            color: self.color.clone(),
            delay_time: self.delay_time.load(Ordering::Relaxed),
            level: self.level.load(Ordering::Relaxed),
        })
    }

    fn active_flag(&self) -> Arc<AtomicBool> {
        self.is_active.clone()
    }

    fn f32_params(&self) -> HashMap<&'static str, Arc<AtomicF32>> {
        let mut map = HashMap::new();
        map.insert("level", Arc::clone(&self.level));
        map
    }

    fn u32_params(&self) -> HashMap<&'static str, Arc<AtomicU32>> {
        let mut map = HashMap::new();
        map.insert("delay_time", Arc::clone(&self.delay_time));
        map
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod success_path {
        use super::*;

        #[test]
        fn test_initialization_and_buffer_size() {
            let sample_rate = 44100;
            let delay_time_ms = 100;
            let delay = Delay::new(1, "Test".to_string(), true, "blue".to_string(), sample_rate, delay_time_ms, 0.5);

            // Max samples for 800ms (as defined in new()) @ 44.1khz is 35280
            assert!(delay.delay_buffer().len() >= 35280);
            assert_eq!(delay.sample_rate(), sample_rate);
        }

        #[test]
        fn test_signal_passthrough_on_first_sample() {
            let mut delay = Delay::new(1, "Test".to_string(), true, "blue".to_string(), 44100, 100, 0.5);
            let input = 0.8;
            let output = delay.process(input);

            // On the very first sample, the buffer is empty (0.0).
            // Output = Dry (0.8) + (Wet (0.0) * 0.5) = 0.8
            assert_eq!(output, input);
        }

        #[test]
        fn test_delay_echo_occurs() {
            let sample_rate = 1000; // Low SR for easier math
            let delay_time_ms = 100; // 100ms = 100 samples at 1000Hz
            let mut delay = Delay::new(1, "Test".to_string(), true, "blue".to_string(), sample_rate, delay_time_ms, 0.5);

            // Input an impulse
            delay.process(1.0);

            // Process silence for 99 samples
            for _ in 0..99 {
                delay.process(0.0);
            }

            // The 101st sample should contain the delayed signal
            let echo = delay.process(0.0);
            assert!(echo > 0.0, "Echo should be audible after delay period");
        }
    }

    mod failure_path {
        use super::*;

        #[test]
        fn test_parameter_clamping() {
            // Test level clamping (max 0.95)
            let mut delay = Delay::new(1, "Test".to_string(), true, "blue".to_string(), 44100, 100, 2.0);
            assert!(delay.level().load(Ordering::Relaxed) <= 0.95);

            // Test delay time clamping via setter (20ms - 300ms)
            delay.set_delay_time(1000);
            assert_eq!(delay.delay_time().load(Ordering::Relaxed), 800);

            delay.set_delay_time(5);
            assert_eq!(delay.delay_time().load(Ordering::Relaxed), 20);
        }

        #[test]
        fn test_empty_buffer_safety() {
            let mut delay = Delay::new(1, "Test".to_string(), true, "blue".to_string(), 44100, 100, 0.5);
            // Manually force an empty buffer (edge case)
            delay.set_sample_rate(0);
            // Should not crash and should return dry signal
            let output = delay.process(0.5);
            assert_eq!(output, 0.5);
        }
    }
}
