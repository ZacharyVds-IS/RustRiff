use crate::commands::helpers::persist_amp_config;
use crate::domain::dto::effect::effect_dto::EffectDto;
use crate::domain::effect::Effect;
use crate::services::amp_config_service::AmpConfigPersistenceService;
use crate::services::audio_service::AudioService;
use crate::services::device_service::DeviceService;
use std::sync::Mutex;
use tracing::info;
use uuid::Uuid;

#[tauri::command]
pub(crate) fn add_effect(
    audio_service: tauri::State<Mutex<AudioService>>,
    device_service: tauri::State<Mutex<DeviceService>>,
    persistence_service: tauri::State<Mutex<AmpConfigPersistenceService>>,
    effect_dto: EffectDto,
) -> Result<(), String> {
    let service = audio_service.inner().lock().unwrap();
    let dsp_sample_rate = service.dsp_chain_sample_rate();

    let effect = effect_dto.add_to_domain(dsp_sample_rate);
    {
        let mut cm = service.channel_manager().lock().unwrap();
        cm.add_effect_to_current(effect);
    }
    let device_service_guard = device_service.inner().lock().unwrap();
    persist_amp_config(&service, &device_service_guard, &persistence_service);
    Ok(())
}

#[tauri::command]
pub(crate) fn remove_effect(
    audio_service: tauri::State<Mutex<AudioService>>,
    device_service: tauri::State<Mutex<DeviceService>>,
    persistence_service: tauri::State<Mutex<AmpConfigPersistenceService>>,
    effect_id: String,
) {
    let service = audio_service.inner().lock().unwrap();
    {
        let mut cm = service.channel_manager().lock().unwrap();
        cm.remove_effect_from_current(Uuid::parse_str(&effect_id).expect("failed to parse id"));
    }
    let device_service_guard = device_service.inner().lock().unwrap();
    persist_amp_config(&service, &device_service_guard, &persistence_service);
}

#[tauri::command]
pub(crate) fn apply_effect_order_change(
    audio_service: tauri::State<Mutex<AudioService>>,
    effects: Vec<EffectDto>,
) {
    let service = audio_service.inner().lock().unwrap();
    let dsp_sample_rate = service.dsp_chain_sample_rate();
    let boxed_effects: Vec<Box<dyn Effect>> = effects
        .into_iter()
        .map(|dto| dto.to_domain(dsp_sample_rate))
        .collect();
    let mut cm = service.channel_manager().lock().unwrap();
    cm.restore_effect_chain_on_current(boxed_effects);
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
    device_service: tauri::State<Mutex<DeviceService>>,
    persistence_service: tauri::State<Mutex<AmpConfigPersistenceService>>,
    effect_id: String,
) -> Result<bool, String> {
    let service = audio_service
        .lock()
        .map_err(|_| "Failed to lock audio service".to_string())?;
    let cm = service.channel_manager().lock().unwrap();
    let new_state =
        cm.toggle_effect_active(Uuid::parse_str(&effect_id).expect("failed to parse id"))?;
    info!(
        channel_id = cm.current_channel_id().to_string(),
        effect_id,
        is_active = new_state,
        "Effect toggled"
    );

    let device_service_guard = device_service
        .lock()
        .map_err(|_| "Failed to lock device service".to_string())?;
    drop(cm);
    persist_amp_config(&service, &device_service_guard, &persistence_service);
    Ok(new_state)
}
