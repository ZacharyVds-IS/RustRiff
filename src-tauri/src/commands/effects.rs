use crate::commands::helpers::persist_amp_config;
use crate::domain::dto::effect::effect_dto::EffectDto;
use crate::domain::effect::Effect;
use crate::services::amp_config_service::AmpConfigPersistenceService;
use crate::services::audio_service::AudioService;
use std::sync::Mutex;
use tracing::info;

#[tauri::command]
pub(crate) fn add_effect(
    audio_service: tauri::State<Mutex<AudioService>>,
    effect_dto: EffectDto,
) -> Result<(), String> {
    let mut service = audio_service.inner().lock().unwrap();
    let target_channel_id = *service.current_channel_id();
    let sample_rate = service.dsp_chain_sample_rate();

    if let Some(channel) = service
        .channels_mut()
        .iter_mut()
        .find(|c| c.id() == target_channel_id)
    {
        let effect = effect_dto.add_to_domain(channel.next_effect_id(), sample_rate);
        channel.add_effect_to_chain(effect);
        Ok(())
    } else {
        Err("Channel not found".into())
    }
}

#[tauri::command]
pub(crate) fn remove_effect(audio_service: tauri::State<Mutex<AudioService>>, effect_id: u32) {
    let mut service = audio_service.inner().lock().unwrap();
    let channel_id = *service.current_channel_id();
    let current_channel = service
        .channels_mut()
        .iter_mut()
        .find(|c| c.id() == channel_id)
        .unwrap();
    current_channel.remove_effect_from_chain(effect_id);
}

#[tauri::command]
pub(crate) fn apply_effect_order_change(
    audio_service: tauri::State<Mutex<AudioService>>,
    effects: Vec<EffectDto>,
) {
    let mut service = audio_service.inner().lock().unwrap();
    let channel_id = *service.current_channel_id();
    let sample_rate = service.dsp_chain_sample_rate();

    let current_channel = service
        .channels_mut()
        .iter_mut()
        .find(|c| c.id() == channel_id)
        .unwrap();
    let boxed_effects: Vec<Box<dyn Effect>> = effects
        .into_iter()
        .map(|dto| dto.to_domain(sample_rate))
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
    effect_id: u32,
) -> Result<bool, String> {
    let service = audio_service
        .lock()
        .map_err(|_| "Failed to lock audio service".to_string())?;
    let channel = service
        .channels()
        .iter()
        .find(|c| c.id() == *service.current_channel_id())
        .ok_or("No active channel")?;
    let new_state = channel.toggle_effect(effect_id)?;
    info!(
        channel_id = *service.current_channel_id(),
        effect_id,
        is_active = new_state,
        "Effect toggled"
    );
    persist_amp_config(&service, &persistence_service);
    Ok(new_state)
}

/// # Sets the Clipping Threshold on an HC Distortion Effect
///
/// Adjusts the Drive parameter: lower thresholds produce heavier distortion.
///
/// # Arguments
/// * `effect_id` — ID of the HCDistortion effect to modify
/// * `threshold` — Clipping level in range `(0.0, 1.0]`
///                 * Values < 0.001 are clamped to 0.001
///                 * Values > 1.0 are clamped to 1.0
///
/// # Returns
/// * `Ok(())` — Threshold updated successfully
/// * `Err(String)` — Error if:
///   - Effect not found or parameter update fails
///   - Threshold is NaN or infinite (audio thread safety)
///
/// # Validation
///
/// This command validates the threshold before forwarding to the audio thread:
/// - Rejects NaN and infinite values (would panic in audio processor)
/// - Clamps finite values to safe range `[0.001, 1.0]`
/// - Prevents audio thread crashes from invalid clamp operations

#[tauri::command]
pub fn set_hc_distortion_threshold(
    audio_service: tauri::State<Mutex<AudioService>>,
    persistence_service: tauri::State<Mutex<AmpConfigPersistenceService>>,
    effect_id: u32,
    threshold: f32,
) -> Result<(), String> {
    if !threshold.is_finite() {
        return Err(format!(
            "Invalid threshold: {} (must be finite, not NaN or infinite)",
            threshold
        ));
    }

    let safe_threshold = threshold.clamp(0.001, 1.0);

    let service = audio_service
        .lock()
        .map_err(|_| "Failed to lock audio service".to_string())?;
    let channel = service
        .channels()
        .iter()
        .find(|c| c.id() == *service.current_channel_id())
        .ok_or("No active channel")?;
    channel.set_effect_param(effect_id, "threshold", safe_threshold)?;
    info!(
        channel_id = *service.current_channel_id(),
        effect_id,
        threshold = safe_threshold,
        "HCDistortion threshold updated"
    );
    persist_amp_config(&service, &persistence_service);
    Ok(())
}

/// # Sets the Output Level (Boost) on an HC Distortion Effect
///
/// Adjusts the Level parameter: controls output amplitude after clipping.
///
/// # Arguments
/// * `effect_id` — ID of the HCDistortion effect to modify
/// * `level` — Normalised level in range `[0.0, 1.0]`
///            * `0.0` = unity gain (no boost)
///            * `1.0` = ×2.0 boost
///            * Values outside range are clamped
///
/// # Returns
/// * `Ok(())` — Level updated successfully
/// * `Err(String)` — Error if:
///   - Effect not found or parameter update fails
///   - Level is NaN or infinite (audio thread safety)
///
/// # Validation
///
/// This command validates the level before forwarding to the audio thread:
/// - Rejects NaN and infinite values (would create invalid gain multiplier)
/// - Clamps finite values to range `[0.0, 1.0]`
/// - Maps to internal gain `[1.0, 2.0]` after validation
/// - Prevents audio thread crashes from invalid gain operations

#[tauri::command]
pub fn set_hc_distortion_level(
    audio_service: tauri::State<Mutex<AudioService>>,
    persistence_service: tauri::State<Mutex<AmpConfigPersistenceService>>,
    effect_id: u32,
    level: f32,
) -> Result<(), String> {
    // Validate level before forwarding to audio thread
    if !level.is_finite() {
        return Err(format!(
            "Invalid level: {} (must be finite, not NaN or infinite)",
            level
        ));
    }

    // Clamp to safe range [0.0, 1.0] then map to internal gain [1.0, 2.0]
    let safe_level = level.clamp(0.0, 1.0);
    let gain = 1.0 + safe_level;

    let service = audio_service
        .lock()
        .map_err(|_| "Failed to lock audio service".to_string())?;
    let channel = service
        .channels()
        .iter()
        .find(|c| c.id() == *service.current_channel_id())
        .ok_or("No active channel")?;
    channel.set_effect_param(effect_id, "level", gain)?;
    info!(
        channel_id = *service.current_channel_id(),
        effect_id,
        level = safe_level,
        gain,
        "HCDistortion level updated"
    );
    persist_amp_config(&service, &persistence_service);
    Ok(())
}

#[tauri::command]
pub fn set_delay_level(
    audio_service: tauri::State<Mutex<AudioService>>,
    persistence_service: tauri::State<Mutex<AmpConfigPersistenceService>>,
    effect_id: u32,
    level: f32,
) -> Result<(), String> {
    if !level.is_finite() {
        return Err(format!(
            "Invalid level: {} (must be finite, not NaN or infinite)",
            level
        ));
    }

    let service = audio_service
        .lock()
        .map_err(|_| "Failed to lock audio service".to_string())?;
    let channel = service
        .channels()
        .iter()
        .find(|c| c.id() == *service.current_channel_id())
        .ok_or("No active channel")?;
    channel.set_effect_param(effect_id, "level", level)?;
    info!(
        channel_id = *service.current_channel_id(),
        effect_id, level, "Delay level updated"
    );
    persist_amp_config(&service, &persistence_service);
    Ok(())
}

#[tauri::command]
pub fn set_delay_delay_time(
    audio_service: tauri::State<Mutex<AudioService>>,
    persistence_service: tauri::State<Mutex<AmpConfigPersistenceService>>,
    effect_id: u32,
    delay_time: f32,
) -> Result<(), String> {
    let service = audio_service
        .lock()
        .map_err(|_| "Failed to lock audio service".to_string())?;
    let channel = service
        .channels()
        .iter()
        .find(|c| c.id() == *service.current_channel_id())
        .ok_or("No active channel")?;
    channel.set_effect_param(effect_id, "delay_time", delay_time)?;
    info!(
        channel_id = *service.current_channel_id(),
        effect_id, delay_time, "Delay delay_time updated"
    );
    persist_amp_config(&service, &persistence_service);
    Ok(())
}
