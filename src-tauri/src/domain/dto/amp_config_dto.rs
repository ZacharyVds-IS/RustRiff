use crate::domain::dto::audio_settings_dto::AudioSettingsDto;
use crate::domain::dto::channel_dto::ChannelDto;
use crate::domain::dto::midi_mapping_dto::MidiMappingDto;
#[cfg(feature = "audio-backend")]
use crate::services::audio_service::AudioService;
use crate::services::device_service::DeviceService;
use cpal::traits::DeviceTrait;
use serde::{Deserialize, Serialize};
#[cfg(feature = "audio-backend")]
use std::sync::atomic::Ordering;

/// Represents the complete amplifier configuration state.
///
/// This DTO is serialized to JSON and sent to the frontend to display the current
/// settings of the amplifier, including gain, master volume, and active/inactive status.
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
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
    /// MIDI CC → effect bindings. Persisted across restarts; absent from
    /// files written before this field was introduced is tolerated via
    /// `#[serde(default)]` on the persistence-layer struct.
    pub midi_bindings: Vec<MidiMappingDto>,
}

impl AmpConfigDto {
    /// Constructs an `AmpConfigDto` from the current state of an [`AudioService`].
    ///
    /// Reads atomic values from the service's channel and master volume with relaxed memory ordering.
    ///
    /// # Arguments
    ///
    /// * `service` - The [`AudioService`] to snapshot.
    #[cfg(feature = "audio-backend")]
    pub fn from_service(audio_service: &AudioService, device_service: &DeviceService) -> Self {
        Self {
            master_volume: audio_service.master_volume().load(Ordering::Relaxed),
            is_active: *audio_service.is_active(),
            channels: cm.to_channel_dtos(),
            current_channel: cm.current_channel_id().to_string(),
            audio_settings: {
                let input_id = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    audio_service
                        .audio_handler()
                        .input_device()
                        .id()
                        .unwrap()
                        .to_string()
                }))
                .unwrap_or_else(|_| "Unknown Input".to_string());

                let output_id = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    audio_service
                        .audio_handler()
                        .output_device()
                        .id()
                        .unwrap()
                        .to_string()
                }))
                .unwrap_or_else(|_| "Unknown Output".to_string());

                AudioSettingsDto {
                    input_device_name: input_id,
                    output_device_name: output_id,
                    input_sample_rate: audio_service.audio_handler().input_sample_rate(),
                    output_sample_rate: audio_service.audio_handler().output_sample_rate(),
                    input_channels: audio_service.audio_handler().input_config().channels,
                    output_channels: audio_service.audio_handler().output_config().channels,
                    audio_driver: device_service.selected_audio_driver().to_string(),
                }
            },
            midi_bindings: Vec::new(),
        }
    }
}
