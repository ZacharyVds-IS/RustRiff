use crate::domain::channel::Channel;
use crate::domain::tone_stack_dto::ToneStackDto;
use serde::{Deserialize, Serialize};
use std::sync::atomic::Ordering;

/// Data transfer object for a Channel's settings.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ChannelDto {
    /// Unique identifier for the Channel.
    pub id: u32,
    /// Name of the Channel
    pub name: String,
    /// The input gain level of the Channel.
    pub gain: f32,
    /// The tone stack settings, including bass, mid, treble of the Channel.
    pub tone_stack: ToneStackDto,
    /// The volume of the Channel.
    pub volume: f32,
}


impl From<&Channel> for ChannelDto {
    fn from(channel: &Channel) -> Self {
        Self {
            id: channel.id().clone(),
            name: channel.name().clone(),
            gain: channel.gain().load(Ordering::Relaxed),
            tone_stack: ToneStackDto::from(channel.tone_stack().as_ref()),
            volume: channel.volume().load(Ordering::Relaxed),
        }
    }
}

