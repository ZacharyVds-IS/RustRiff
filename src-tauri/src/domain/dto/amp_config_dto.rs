use crate::domain::dto::audio_settings_dto::AudioSettingsDto;
use crate::domain::dto::channel_dto::ChannelDto;
use crate::services::audio_service::AudioService;
use cpal::traits::DeviceTrait;
use serde::{Deserialize, Serialize};
use std::sync::atomic::Ordering;

/// Represents the complete amplifier configuration state.
///
/// This DTO is serialized to JSON and sent to the frontend to display the current
/// settings of the amplifier, including gain, master volume, and active/inactive status.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AmpConfigDto {
    /// The current master volume level.
    pub master_volume: f32,
    /// Whether the audio loopback is currently active.
    pub is_active: bool,
    /// The list of all channels with their respective settings (gain, tone stack, volume).
    pub channels: Vec<ChannelDto>,
    /// The current channel id
    pub current_channel: String,
    /// Hardware Configuration
    pub audio_settings: AudioSettingsDto,
}

impl AmpConfigDto {
    /// Constructs an `AmpConfigDto` from the current state of an [`AudioService`].
    ///
    /// Reads atomic values from the service's channel and master volume with relaxed memory ordering.
    ///
    /// # Arguments
    ///
    /// * `service` - The [`AudioService`] to snapshot.
    pub fn from_service(service: &AudioService) -> Self {
        let channel = service
            .channels()
            .iter()
            .find(|c| c.id() == *service.current_channel_id())
            .unwrap();

        Self {
            master_volume: service.master_volume().load(Ordering::Relaxed),
            is_active: *service.is_active(),
            channels: service.channels().iter().map(ChannelDto::from).collect(),
            current_channel: channel.id().to_string(),
            audio_settings: {
                // Wrap device access in catch_unwind to handle mock panic scenarios in tests
                let input_name = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    service.audio_handler().input_device().name().unwrap_or_else(|_| "Unknown Input".to_string())
                })).unwrap_or_else(|_| "Unknown Input".to_string());

                let output_name = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    service.audio_handler().output_device().name().unwrap_or_else(|_| "Unknown Output".to_string())
                })).unwrap_or_else(|_| "Unknown Output".to_string());

                AudioSettingsDto {
                    input_device_name: input_name,
                    output_device_name: output_name,
                    input_sample_rate: service.audio_handler().input_sample_rate(),
                    output_sample_rate: service.audio_handler().output_sample_rate(),
                    input_channels: service.audio_handler().input_config().channels,
                    output_channels: service.audio_handler().output_config().channels,
                }
            }
        }
    }
}
