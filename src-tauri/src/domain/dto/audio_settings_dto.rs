use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AudioSettingsDto {
    pub input_device_name: String,
    pub output_device_name: String,
    pub input_sample_rate: u32,
    pub output_sample_rate: u32,
    pub input_channels: u16,
    pub output_channels: u16,
    pub audio_drivers: String,
}