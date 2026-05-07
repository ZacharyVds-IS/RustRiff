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
    delay_time: Arc<AtomicU32>,       //20ms - 300ms
    level: Arc<AtomicF32>, //0.0-0.95
    delay_buffer: Vec<f32>,
    write_pos: usize,
    sample_rate: u32,
    delay_in_samples: usize,
}

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
        let delay_time_arc = Arc::new(AtomicU32::new(delay_time.clamp(20, 300)));

        Self {
            id,
            name,
            is_active: Arc::new(AtomicBool::new(is_active)),
            color,
            delay_time: delay_time_arc,
            level: level_arc,
            delay_buffer: Vec::new(),
            write_pos: 0,
            sample_rate,
            delay_in_samples: 0,
        }
    }

    fn calc_delay_in_samples(&mut self) {
        self.delay_in_samples =
            (self.delay_time.load(Ordering::Relaxed) as f32 * self.sample_rate as f32 / 1000.0) as usize;
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
    pub fn set_delay_time(&mut self, delay_time: u32) {
        self.delay_time.store(delay_time.clamp(20,300), Ordering::Relaxed);
        self.calc_delay_in_samples()
    }

    pub fn set_level(&mut self, level: f32) {
        self.level.store(level.clamp(0.0, 0.95), Ordering::Relaxed);
    }

    pub fn set_sample_rate(&mut self, sample_rate: u32) {
        self.sample_rate = sample_rate;
        self.calc_delay_in_samples()
    }
}

impl AudioProcessor for Delay {
    fn process(&mut self, sample: f32) -> f32 {
        let read_pos = (self.write_pos + self.delay_buffer.len() - self.delay_in_samples)
            % self.delay_buffer.len();

        let delayed_sample = self.delay_buffer[read_pos];

        let feedback = self.level.load(Ordering::Relaxed);
        self.delay_buffer[self.write_pos] = sample + (delayed_sample * feedback);

        self.write_pos = (self.write_pos + 1) % self.delay_buffer.len();

        delayed_sample
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
            id:self.id(),
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
