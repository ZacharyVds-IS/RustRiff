use crate::domain::dto::channel_dto::ChannelDto;
use crate::domain::dto::midi_mapping_dto::MidiMappingDto;
#[cfg(feature = "audio-backend")]
use crate::services::audio_service::AudioService;
use serde::{Deserialize, Serialize};
#[cfg(feature = "audio-backend")]
use std::sync::atomic::Ordering;

/// Represents the complete amplifier configuration state.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AmpConfigDto {
    pub master_volume: f32,
    pub is_active: bool,
    pub channels: Vec<ChannelDto>,
    pub current_channel: String,
    /// MIDI CC → effect bindings. Persisted across restarts; absent from
    /// files written before this field was introduced is tolerated via
    /// `#[serde(default)]` on the persistence-layer struct.
    pub midi_bindings: Vec<MidiMappingDto>,
}

impl Default for AmpConfigDto {
    fn default() -> Self {
        Self {
            master_volume: 1.0,
            is_active: false,
            channels: Vec::new(),
            current_channel: String::new(),
            midi_bindings: Vec::new(),
        }
    }
}

impl AmpConfigDto {
    #[cfg(feature = "audio-backend")]
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
            midi_bindings: Vec::new(),
        }
    }
}
