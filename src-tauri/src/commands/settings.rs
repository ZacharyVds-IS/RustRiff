use crate::domain::dto::audio_device_dto::AudioDeviceDto;
use crate::services::audio_service::AudioService;
use crate::services::device_service::DeviceService;
use cpal::traits::DeviceTrait;
use std::sync::Mutex;
use tracing::info;

/// Normalizes channel count to a supported value (1 or 2).
///
/// Some devices report unusual channel counts that can cause audio issues.
/// This function ensures we use either mono (1) or stereo (2) channels.
///
/// # Arguments
///
/// * `channels` - The reported channel count from the device
///
/// # Returns
///
/// A normalized channel count (1 or 2)
fn normalize_channels(channels: u16) -> u16 {
    match channels {
        1 => 1,
        _ => 2, // Default to stereo for any other count
    }
}

/// Retrieves a list of all available input devices.
///
/// Queries the [`DeviceService`] for all detected input devices and returns
/// them as [`AudioDeviceDto`] objects suitable for frontend display and selection.
///
/// # Arguments
///
/// * `device_service` - The shared [`DeviceService`] state, accessed via Tauri's state management.
///
/// # Returns
///
/// A [`Vec`] of [`AudioDeviceDto`] representing available input devices.
#[tauri::command]
pub(crate) fn get_input_device_list(device_service: tauri::State<DeviceService>) -> Vec<AudioDeviceDto> {
    device_service.get_input_devices()
}

/// Retrieves a list of all available output devices.
///
/// Queries the [`DeviceService`] for all detected output devices and returns
/// them as [`AudioDeviceDto`] objects suitable for frontend display and selection.
///
/// # Arguments
///
/// * `device_service` - The shared [`DeviceService`] state, accessed via Tauri's state management.
///
/// # Returns
///
/// A [`Vec`] of [`AudioDeviceDto`] representing available output devices.
#[tauri::command]
pub(crate) fn get_output_device_list(device_service: tauri::State<DeviceService>) -> Vec<AudioDeviceDto> {
    device_service.get_output_devices()
}

/// Switches the active input device.
///
/// Looks up the device by ID in the [`DeviceService`], then delegates to
/// [`AudioService::set_input_device`] to perform the hot-swap without interrupting
/// playback longer than necessary.
///
/// # Arguments
///
/// * `device_service` - The shared [`DeviceService`] state for device lookup.
/// * `audio_service` - The shared [`AudioService`] state for performing the switch.
/// * `device_id` - The ID of the input device to activate.
///
/// # Returns
///
/// Returns `Ok(())` on success, or `Err(String)` if the device ID is not found
/// or the service state cannot be locked.
#[tauri::command]
pub fn set_input_device(
    device_service: tauri::State<DeviceService>,
    audio_service: tauri::State<'_, Mutex<AudioService>>,
    device_id: String,
) -> Result<(), String> {
    let device = device_service
        .find_input_device_by_id(&device_id)
        .ok_or("Device not found")?;

    let device_name = device
        .description()
        .map(|d| d.name().to_string())
        .unwrap_or_else(|_| "Unknown".to_string());
    let supported_config = device
        .default_input_config()
        .map_err(|e| format!("Failed to get default input config for '{}': {}", device_name, e))?;

    let mut input_config = supported_config.config();
    let normalized_channels = normalize_channels(input_config.channels);

    if input_config.channels != normalized_channels {
        info!(
            "Input device '{}' reported {} channels, normalizing to {}",
            device_name, input_config.channels, normalized_channels
        );
        input_config.channels = normalized_channels;
    }

    info!(
        "Switching to input device '{}' - {} ch @ {} Hz",
        device_name, input_config.channels, input_config.sample_rate
    );

    let mut audio = audio_service
        .lock()
        .map_err(|_| "Failed to lock audio service".to_string())?;
    audio.set_input_device(device, input_config);

    Ok(())
}

/// Switches the active output device.
///
/// Looks up the device by ID in the [`DeviceService`], then delegates to
/// [`AudioService::set_output_device`] to perform the hot-swap without interrupting
/// playback longer than necessary.
///
/// # Arguments
///
/// * `device_service` - The shared [`DeviceService`] state for device lookup.
/// * `audio_service` - The shared [`AudioService`] state for performing the switch.
/// * `device_id` - The ID of the output device to activate.
///
/// # Returns
///
/// Returns `Ok(())` on success, or `Err(String)` if the device ID is not found
/// or the service state cannot be locked.
#[tauri::command]
pub fn set_output_device(
    device_service: tauri::State<DeviceService>,
    audio_service: tauri::State<'_, Mutex<AudioService>>,
    device_id: String,
) -> Result<(), String> {
    let device = device_service
        .find_output_device_by_id(&device_id)
        .ok_or("Device not found")?;

    let device_name = device
        .description()
        .map(|d| d.name().to_string())
        .unwrap_or_else(|_| "Unknown".to_string());
    let supported_config = device
        .default_output_config()
        .map_err(|e| format!("Failed to get default output config for '{}': {}", device_name, e))?;

    let mut output_config = supported_config.config();
    let normalized_channels = normalize_channels(output_config.channels);

    if output_config.channels != normalized_channels {
        info!(
            "Output device '{}' reported {} channels, normalizing to {}",
            device_name, output_config.channels, normalized_channels
        );
        output_config.channels = normalized_channels;
    }

    info!(
        "Switching to output device '{}' - {} ch @ {} Hz",
        device_name, output_config.channels, output_config.sample_rate
    );

    let mut audio = audio_service
        .lock()
        .map_err(|_| "Failed to lock audio service".to_string())?;
    audio.set_output_device(device, output_config);

    Ok(())
}

#[tauri::command]
pub fn get_buffer_size_frames(
    audio_service: tauri::State<'_, Mutex<AudioService>>,
) -> Result<u32, String> {
    let audio = audio_service
        .lock()
        .map_err(|_| "Failed to lock audio service".to_string())?;
    Ok(audio.buffer_size_frames())
}

#[tauri::command]
pub fn set_buffer_size_frames(
    audio_service: tauri::State<'_, Mutex<AudioService>>,
    frames: u32,
) -> Result<(), String> {
    let mut audio = audio_service
        .lock()
        .map_err(|_| "Failed to lock audio service".to_string())?;
    audio.set_buffer_size_frames(frames)
}
