use crate::commands::helpers::persist_amp_config;
use crate::domain::dto::effect::effect_dto::EffectDto;
use crate::domain::effect::Effect;
use crate::services::amp_config_service::AmpConfigPersistenceService;
use crate::services::audio_service::AudioService;
use std::sync::Mutex;
use tracing::info;
use uuid::Uuid;

#[tauri::command]
pub(crate) fn add_effect(
    audio_service: tauri::State<Mutex<AudioService>>,
    persistence_service: tauri::State<Mutex<AmpConfigPersistenceService>>,
    effect_dto: EffectDto,
) -> Result<(), String> {
    let mut service = audio_service.inner().lock().unwrap();
    let target_channel_id = *service.current_channel_id();
    let dsp_sample_rate = service.dsp_chain_sample_rate();

    if let Some(channel) = service
        .channels_mut()
        .iter_mut()
        .find(|c| c.id() == target_channel_id)
    {
        let effect = effect_dto.add_to_domain(dsp_sample_rate);
        channel.add_effect_to_chain(effect);
        persist_amp_config(&service, &persistence_service);
        Ok(())
    } else {
        Err("Channel not found".into())
    }
}

#[tauri::command]
pub(crate) fn remove_effect(
    audio_service: tauri::State<Mutex<AudioService>>,
    persistence_service: tauri::State<Mutex<AmpConfigPersistenceService>>,
    effect_id: String,
) {
    let mut service = audio_service.inner().lock().unwrap();
    let channel_id = *service.current_channel_id();
    let current_channel = service
        .channels_mut()
        .iter_mut()
        .find(|c| c.id() == channel_id)
        .unwrap();
    current_channel
        .remove_effect_from_chain(Uuid::parse_str(&effect_id).expect("failed to parse id"));
    persist_amp_config(&service, &persistence_service);
}

#[tauri::command]
pub(crate) fn apply_effect_order_change(
    audio_service: tauri::State<Mutex<AudioService>>,
    effects: Vec<EffectDto>,
) {
    let mut service = audio_service.inner().lock().unwrap();
    let dsp_sample_rate = service.dsp_chain_sample_rate();
    let channel_id = *service.current_channel_id();
    let current_channel = service
        .channels_mut()
        .iter_mut()
        .find(|c| c.id() == channel_id)
        .unwrap();
    let boxed_effects: Vec<Box<dyn Effect>> = effects
        .into_iter()
        .map(|dto| dto.to_domain(dsp_sample_rate))
        .collect();
    current_channel.restore_effect_chain(boxed_effects);
}

/// Toggles an effect's active state on the current channel.
/// Enables or disables audio processing for a specific effect. The change takes effect
/// on the very next audio sample — no loopback restart needed.
///
/// This is a generic command that works with any effect type.
///
/// # Arguments
/// * `effect_id` — Unique ID of the effect to toggle
///
/// # Returns
/// * `Ok(bool)` — The new active state (`true` = processing, `false` = bypassed)
/// * `Err(String)` — Error message if effect ID is invalid or channel not found
///
/// # Implementation Details
///
/// - Updates the effect's [`Arc<AtomicBool>`] active flag
/// - Changes apply immediately to audio processing thread
#[tauri::command]
pub fn toggle_effect(
    audio_service: tauri::State<Mutex<AudioService>>,
    persistence_service: tauri::State<Mutex<AmpConfigPersistenceService>>,
    effect_id: String,
) -> Result<bool, String> {
    let service = audio_service
        .lock()
        .map_err(|_| "Failed to lock audio service".to_string())?;
    let channel = service
        .channels()
        .iter()
        .find(|c| c.id() == *service.current_channel_id())
        .ok_or("No active channel")?;
    let new_state =
        channel.toggle_effect(Uuid::parse_str(&effect_id).expect("failed to parse id"))?;
    info!(
        channel_id = service.current_channel_id().to_string(),
        effect_id,
        is_active = new_state,
        "Effect toggled"
    );
    persist_amp_config(&service, &persistence_service);
    Ok(new_state)
}
