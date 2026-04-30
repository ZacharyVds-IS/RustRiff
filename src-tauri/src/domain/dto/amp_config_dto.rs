use serde::{Serialize, Deserialize};
use std::sync::atomic::Ordering;
use crate::domain::tone_stack_dto::ToneStackDto;
use crate::services::audio_service::AudioService;
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
    pub current_channel: u32,
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
        let channel = service.channels().iter().find(|c| c.id() == *service.current_channel_id()).unwrap();

        Self {
            master_volume: service.master_volume().load(Ordering::Relaxed),
            is_active: *service.is_active(),
            channels: service.channels().iter().map(|c| ChannelDto::from(c)).collect(),
            current_channel: channel.id(),
        }
    }
}