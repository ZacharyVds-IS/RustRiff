use crate::domain::channel::{Channel, ParamInput};
use crate::domain::dto::channel_dto::ChannelDto;
use crate::domain::dto::effect::effect_dto::EffectDto;
use crate::domain::effect::Effect;
use atomic_float::AtomicF32;
use std::sync::{Arc, Mutex};
use tracing::error;
use uuid::Uuid;

/// Processor `Arc`s cloned from a channel and passed into the loopback threads.
pub struct ChannelArcs {
    pub gain: Arc<AtomicF32>,
    pub volume: Arc<AtomicF32>,
    pub tone_stack: Arc<crate::domain::tone_stack::ToneStack>,
    pub effect_chain: Arc<Mutex<Vec<Box<dyn Effect>>>>,
}

/// Manages the amplifier's channel state independently of the audio service.
///
/// Holds the full channel list, current channel selection, and provides
/// methods for both channel CRUD and effect parameter control. This struct
/// is shared between `AudioService` (for audio thread setup) and
/// `MidiService` (for real‑time parameter writes) via `Arc<Mutex<ChannelManager>>`.
pub struct ChannelManager {
    channels: Vec<Channel>,
    current_channel_id: Uuid,
}

impl Default for ChannelManager {
    fn default() -> Self {
        Self::new()
    }
}

impl ChannelManager {
    pub fn new() -> Self {
        let default_channel_id = Uuid::new_v4();
        Self {
            channels: vec![Channel::new(
                default_channel_id,
                "Default".to_string(),
                None,
                None,
            )],
            current_channel_id: default_channel_id,
        }
    }

    // ── Accessors ──────────────────────────────────────────────────────────────

    pub fn channels(&self) -> &Vec<Channel> {
        &self.channels
    }

    pub fn channels_mut(&mut self) -> &mut Vec<Channel> {
        &mut self.channels
    }

    pub fn current_channel_id(&self) -> &Uuid {
        &self.current_channel_id
    }

    pub fn current_channel(&self) -> Result<&Channel, String> {
        self.channels
            .iter()
            .find(|c| c.id() == self.current_channel_id)
            .ok_or_else(|| "No active channel".to_string())
    }

    fn current_channel_mut(&mut self) -> Result<&mut Channel, String> {
        self.channels
            .iter_mut()
            .find(|c| c.id() == self.current_channel_id)
            .ok_or_else(|| "No active channel".to_string())
    }

    // ── Channel CRUD ──────────────────────────────────────────────────────────

    pub fn add_channel(&mut self, name: String) -> Uuid {
        if name.len() <= 30 {
            let id = Uuid::new_v4();
            self.channels.push(Channel::new(id, name, None, None));
            self.current_channel_id = id;
            id
        } else {
            error!("Channel name must be 30 characters or less");
            panic!("Channel name must be 30 characters or less");
        }
    }

    pub fn remove_channel(&mut self, channel_id: Uuid) {
        let default_channel_id = self.channels.first().unwrap().id();
        if channel_id != default_channel_id {
            self.channels.retain(|c| c.id() != channel_id);
            self.current_channel_id = default_channel_id;
        } else {
            error!("Cannot remove default channel");
        }
    }

    pub fn set_current_channel_id(&mut self, new_id: Uuid) {
        self.current_channel_id = new_id;
    }

    // ── Audio thread support ──────────────────────────────────────────────────

    /// Clones the processor `Arc`s from the current channel.
    ///
    /// Panics if `current_channel_id` does not match any channel.
    pub fn resolve_channel_arcs(&self) -> ChannelArcs {
        let channel = self
            .channels
            .iter()
            .find(|c| c.id() == self.current_channel_id)
            .expect("current_channel_id must reference an existing channel");

        ChannelArcs {
            gain: channel.gain(),
            volume: channel.volume(),
            tone_stack: channel.tone_stack(),
            effect_chain: channel.effect_chain(),
        }
    }

    // ── Effect control (the "state mappings of channel") ──────────────────────

    pub fn set_effect_parameter(
        &self,
        effect_id: Uuid,
        param: &str,
        value: impl Into<ParamInput>,
    ) -> Result<(), String> {
        let channel = self.current_channel()?;
        channel.set_effect_param(effect_id, param, value)
    }

    pub fn toggle_effect_active(&self, effect_id: Uuid) -> Result<bool, String> {
        let channel = self.current_channel()?;
        channel.toggle_effect(effect_id)
    }

    pub fn set_effect_active(&self, effect_id: Uuid, active: bool) -> Result<(), String> {
        let channel = self.current_channel()?;
        channel.set_effect_active(effect_id, active)
    }

    pub fn add_effect_to_current(&mut self, effect: Box<dyn Effect>) {
        if let Ok(channel) = self.current_channel_mut() {
            channel.add_effect_to_chain(effect);
        }
    }

    pub fn remove_effect_from_current(&mut self, effect_id: Uuid) {
        if let Ok(channel) = self.current_channel_mut() {
            channel.remove_effect_from_chain(effect_id);
        }
    }

    pub fn restore_effect_chain_on_current(&mut self, effects: Vec<Box<dyn Effect>>) {
        if let Ok(channel) = self.current_channel_mut() {
            channel.restore_effect_chain(effects);
        }
    }

    // ── Persistence / snapshot ────────────────────────────────────────────────

    /// Restores channels from a persisted configuration.
    ///
    /// Takes ownership of channel DTOs and the current channel id string
    /// so `EffectDto::to_domain` can consume each entry.
    pub fn restore_from_dtos(
        &mut self,
        channel_dtos: Vec<ChannelDto>,
        current_channel_id_str: &str,
        dsp_sample_rate: u32,
    ) {
        let mut restored_channels = Vec::new();

        let normalize_tone_value = |value: f32| -> f32 {
            if value > 1.0 {
                (value / 100.0).clamp(0.0, 1.0)
            } else {
                value.clamp(0.0, 1.0)
            }
        };

        for channel_dto in channel_dtos {
            let mut channel = Channel::new(
                Uuid::parse_str(channel_dto.id.as_str()).expect("Could not parse UUID"),
                channel_dto.name,
                Some(channel_dto.gain.max(0.0001)),
                Some(channel_dto.volume.max(0.0001)),
            );

            channel.set_bass(normalize_tone_value(channel_dto.tone_stack.bass));
            channel.set_middle(normalize_tone_value(channel_dto.tone_stack.middle));
            channel.set_treble(normalize_tone_value(channel_dto.tone_stack.treble));

            let restored_effects = channel_dto
                .effect_chain
                .into_iter()
                .map(|effect_dto: EffectDto| effect_dto.to_domain(dsp_sample_rate))
                .collect::<Vec<_>>();

            if !restored_effects.is_empty() {
                channel.restore_effect_chain(restored_effects);
            }
            restored_channels.push(channel);
        }

        if restored_channels.is_empty() {
            restored_channels.push(Channel::new(
                Uuid::new_v4(),
                "Default".to_string(),
                None,
                None,
            ));
        }

        let current_channel = if restored_channels
            .iter()
            .any(|c| c.id().to_string() == current_channel_id_str)
        {
            Uuid::parse_str(current_channel_id_str).expect("Could not parse UUID")
        } else {
            restored_channels[0].id()
        };

        self.channels = restored_channels;
        self.current_channel_id = current_channel;
    }

    /// Produces channel DTOs for the frontend.
    pub fn to_channel_dtos(&self) -> Vec<ChannelDto> {
        self.channels.iter().map(ChannelDto::from).collect()
    }
}
