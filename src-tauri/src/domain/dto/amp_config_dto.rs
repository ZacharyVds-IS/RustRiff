use crate::domain::dto::channel_dto::ChannelDto;
use crate::services::audio_service::AudioService;
use serde::{Deserialize, Serialize};
use std::sync::atomic::Ordering;

/// Represents the complete amplifier configuration state.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AmpConfigDto {
    pub master_volume: f32,
    pub is_active: bool,
    pub channels: Vec<ChannelDto>,
    pub current_channel: String,
}

impl AmpConfigDto {
    /// Constructs an `AmpConfigDto` from the current state of an [`AudioService`].
    pub fn from_service(service: &AudioService) -> Self {
        let cm = service
            .channel_manager()
            .lock()
            .expect("channel_manager lock");

        Self {
            master_volume: service.master_volume().load(Ordering::Relaxed),
            is_active: *service.is_active(),
            channels: cm.to_channel_dtos(),
            current_channel: cm.current_channel_id().to_string(),
        }
    }
}
