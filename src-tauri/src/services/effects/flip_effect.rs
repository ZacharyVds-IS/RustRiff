use crate::domain::audio_processor::AudioProcessor;
use crate::domain::effect::Effect;
use tracing::info;
//This is a test effect that simply flips the audio signal (multiplies by -1). It is used for UI integration.
pub struct FlipEffect {
    id: u32,
    name: String,
    is_active: bool,
    color: String,
}

impl FlipEffect {
    pub fn new(id: u32, name: String, color: String) -> Self {
        Self{
            id,
            name,
            is_active: false,
            color,
        }
    }
}

impl AudioProcessor for FlipEffect {
    fn process(&mut self, sample: f32) -> f32 {
        sample * -1.0
    }
}

impl Effect for FlipEffect {
    fn id(&self) -> u32 {
        self.id
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn is_active(&self) -> bool {
        self.is_active
    }

    fn set_active(&mut self, active: bool) {
        self.is_active = active;
    }

    fn get_color(&self) -> String {
        self.color.clone()
    }

    fn process_if_active(&mut self, sample: f32) -> f32 {
        if self.is_active() {
            info!("Processing sample through {}: input={}", self.name, sample);
            self.process(sample)
        } else {
            sample
        }
    }
}