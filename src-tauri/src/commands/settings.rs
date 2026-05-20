use crate::domain::dto::audio_device_dto::AudioDeviceDto;
use crate::services::audio_service::AudioService;
use crate::services::device_service::DeviceService;
use cpal::traits::DeviceTrait;
use cpal::{
    Device, SampleFormat, SampleRate, StreamConfig, SupportedStreamConfig,
    SupportedStreamConfigRange,
};
use std::collections::BTreeSet;
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

fn choose_input_supported_config(
    device: &Device,
    device_name: &str,
) -> Result<SupportedStreamConfig, String> {
    let default_supported = device.default_input_config().map_err(|e| {
        format!(
            "Failed to get default input config for '{}': {}",
            device_name, e
        )
    })?;

    if default_supported.sample_format() == SampleFormat::F32 {
        return Ok(default_supported);
    }

    let wanted_rate = default_supported.sample_rate();
    if let Ok(config_ranges) = device.supported_input_configs() {
        for range in config_ranges {
            if range.sample_format() != SampleFormat::F32 {
                continue;
            }

            let min_rate = range.min_sample_rate();
            let max_rate = range.max_sample_rate();
            let selected_rate = if wanted_rate < min_rate {
                min_rate
            } else if wanted_rate > max_rate {
                max_rate
            } else {
                wanted_rate
            };

            info!(
                "Input device '{}' default format is {:?}; using F32 @ {} Hz for stream compatibility",
                device_name,
                default_supported.sample_format(),
                selected_rate
            );
            return Ok(range.with_sample_rate(selected_rate));
        }
    }

    info!(
        "Input device '{}' has no F32-supported config; keeping default format {:?}",
        device_name,
        default_supported.sample_format()
    );
    Ok(default_supported)
}

fn choose_output_supported_config(
    device: &Device,
    device_name: &str,
) -> Result<SupportedStreamConfig, String> {
    let default_supported = device.default_output_config().map_err(|e| {
        format!(
            "Failed to get default output config for '{}': {}",
            device_name, e
        )
    })?;

    if default_supported.sample_format() == SampleFormat::F32 {
        return Ok(default_supported);
    }

    let wanted_rate = default_supported.sample_rate();
    if let Ok(config_ranges) = device.supported_output_configs() {
        for range in config_ranges {
            if range.sample_format() != SampleFormat::F32 {
                continue;
            }

            let min_rate = range.min_sample_rate();
            let max_rate = range.max_sample_rate();
            let selected_rate = if wanted_rate < min_rate {
                min_rate
            } else if wanted_rate > max_rate {
                max_rate
            } else {
                wanted_rate
            };

            info!(
                "Output device '{}' default format is {:?}; using F32 @ {} Hz for stream compatibility",
                device_name,
                default_supported.sample_format(),
                selected_rate
            );
            return Ok(range.with_sample_rate(selected_rate));
        }
    }

    info!(
        "Output device '{}' has no F32-supported config; keeping default format {:?}",
        device_name,
        default_supported.sample_format()
    );
    Ok(default_supported)
}

fn rate_is_in_range(rate: SampleRate, range: &SupportedStreamConfigRange) -> bool {
    rate >= range.min_sample_rate() && rate <= range.max_sample_rate()
}

fn pick_asio_shared_rate(
    input_ranges: &[SupportedStreamConfigRange],
    output_ranges: &[SupportedStreamConfigRange],
    preferred_input_rate: SampleRate,
    preferred_output_rate: SampleRate,
) -> Option<SampleRate> {
    for candidate in [preferred_output_rate, preferred_input_rate] {
        let input_ok = input_ranges
            .iter()
            .any(|range| rate_is_in_range(candidate, range));
        let output_ok = output_ranges
            .iter()
            .any(|range| rate_is_in_range(candidate, range));
        if input_ok && output_ok {
            return Some(candidate);
        }
    }

    for input in input_ranges {
        for output in output_ranges {
            let min_rate = input.min_sample_rate().max(output.min_sample_rate());
            let max_rate = input.max_sample_rate().min(output.max_sample_rate());
            if min_rate <= max_rate {
                return Some(min_rate);
            }
        }
    }

    None
}

fn collect_supported_input_channels(device: &Device) -> Result<Vec<u16>, String> {
    let mut channels = BTreeSet::new();
    for range in device
        .supported_input_configs()
        .map_err(|e| format!("Failed to query supported input channels: {}", e))?
    {
        channels.insert(range.channels());
    }
    Ok(channels.into_iter().collect())
}

fn collect_supported_output_channels(device: &Device) -> Result<Vec<u16>, String> {
    let mut channels = BTreeSet::new();
    for range in device
        .supported_output_configs()
        .map_err(|e| format!("Failed to query supported output channels: {}", e))?
    {
        channels.insert(range.channels());
    }
    Ok(channels.into_iter().collect())
}

fn build_asio_io_configs(
    device: &Device,
    desired_input_channels: Option<u16>,
    desired_output_channels: Option<u16>,
) -> Result<(StreamConfig, StreamConfig), String> {
    let device_name = device
        .description()
        .map(|d| d.name().to_string())
        .unwrap_or_else(|_| "Unknown".to_string());

    let preferred_input_rate = device
        .default_input_config()
        .map_err(|e| {
            format!(
                "Failed to get default input config for '{}': {}",
                device_name, e
            )
        })?
        .sample_rate();
    let preferred_output_rate = device
        .default_output_config()
        .map_err(|e| {
            format!(
                "Failed to get default output config for '{}': {}",
                device_name, e
            )
        })?
        .sample_rate();

    let input_ranges: Vec<SupportedStreamConfigRange> = device
        .supported_input_configs()
        .map_err(|e| {
            format!(
                "Failed to query ASIO input configs for '{}': {}",
                device_name, e
            )
        })?
        .filter(|range| {
            desired_input_channels
                .map(|channels| range.channels() == channels)
                .unwrap_or(true)
        })
        .collect();
    let output_ranges: Vec<SupportedStreamConfigRange> = device
        .supported_output_configs()
        .map_err(|e| {
            format!(
                "Failed to query ASIO output configs for '{}': {}",
                device_name, e
            )
        })?
        .filter(|range| {
            desired_output_channels
                .map(|channels| range.channels() == channels)
                .unwrap_or(true)
        })
        .collect();

    if input_ranges.is_empty() || output_ranges.is_empty() {
        return Err(format!(
            "ASIO device '{}' has no full-duplex stream configuration",
            device_name
        ));
    }

    let shared_rate = pick_asio_shared_rate(
        &input_ranges,
        &output_ranges,
        preferred_input_rate,
        preferred_output_rate,
    )
    .ok_or_else(|| {
        format!(
            "ASIO device '{}' has no shared sample rate between input/output configs",
            device_name
        )
    })?;

    let input_config = input_ranges
        .iter()
        .find(|range| rate_is_in_range(shared_rate, range))
        .ok_or_else(|| {
            format!(
                "ASIO device '{}' failed to select input config at {} Hz",
                device_name, shared_rate
            )
        })?
        .with_sample_rate(shared_rate)
        .config();

    let output_config = output_ranges
        .iter()
        .find(|range| rate_is_in_range(shared_rate, range))
        .ok_or_else(|| {
            format!(
                "ASIO device '{}' failed to select output config at {} Hz",
                device_name, shared_rate
            )
        })?
        .with_sample_rate(shared_rate)
        .config();

    Ok((input_config, output_config))
}

fn build_input_config(device: &Device, normalize: bool) -> Result<StreamConfig, String> {
    let device_name = device
        .description()
        .map(|d| d.name().to_string())
        .unwrap_or_else(|_| "Unknown".to_string());

    let supported_config = choose_input_supported_config(device, &device_name)?;

    let mut input_config = supported_config.config();
    let normalized_channels = normalize_channels(input_config.channels);
    if normalize && input_config.channels != normalized_channels {
        info!(
            "Input device '{}' reported {} channels, normalizing to {}",
            device_name, input_config.channels, normalized_channels
        );
        input_config.channels = normalized_channels;
    }

    Ok(input_config)
}

fn build_output_config(device: &Device, normalize: bool) -> Result<StreamConfig, String> {
    let device_name = device
        .description()
        .map(|d| d.name().to_string())
        .unwrap_or_else(|_| "Unknown".to_string());

    let supported_config = choose_output_supported_config(device, &device_name)?;

    let mut output_config = supported_config.config();
    let normalized_channels = normalize_channels(output_config.channels);
    if normalize && output_config.channels != normalized_channels {
        info!(
            "Output device '{}' reported {} channels, normalizing to {}",
            device_name, output_config.channels, normalized_channels
        );
        output_config.channels = normalized_channels;
    }

    Ok(output_config)
}

fn apply_asio_device_route(
    audio_service: tauri::State<'_, Mutex<AudioService>>,
    device: Device,
) -> Result<(), String> {
    // ASIO typically requires one shared device clock/rate for full-duplex streams.
    let (input_config, output_config) = build_asio_io_configs(&device, None, None)?;
    let device_name = device
        .description()
        .map(|d| d.name().to_string())
        .unwrap_or_else(|_| "Unknown".to_string());

    info!(
        "Switching ASIO full-duplex device '{}' - in {} ch @ {} Hz, out {} ch @ {} Hz",
        device_name,
        input_config.channels,
        input_config.sample_rate,
        output_config.channels,
        output_config.sample_rate
    );

    let mut audio = audio_service
        .lock()
        .map_err(|_| "Failed to lock audio service".to_string())?;
    audio.set_io_devices(device.clone(), device, input_config, output_config);
    Ok(())
}

#[tauri::command]
pub fn get_available_audio_drivers(device_service: tauri::State<DeviceService>) -> Vec<String> {
    device_service.available_audio_drivers()
}

#[tauri::command]
pub fn get_selected_audio_driver(device_service: tauri::State<DeviceService>) -> String {
    device_service.selected_audio_driver()
}

#[tauri::command]
pub fn get_input_channel_options(
    device_service: tauri::State<DeviceService>,
    device_id: String,
) -> Result<Vec<u16>, String> {
    let device = device_service
        .find_input_device_by_id(&device_id)
        .ok_or_else(|| "Input device not found".to_string())?;
    collect_supported_input_channels(&device)
}

#[tauri::command]
pub fn get_output_channel_options(
    device_service: tauri::State<DeviceService>,
    device_id: String,
) -> Result<Vec<u16>, String> {
    let device = device_service
        .find_output_device_by_id(&device_id)
        .ok_or_else(|| "Output device not found".to_string())?;
    collect_supported_output_channels(&device)
}

#[tauri::command]
pub fn get_selected_input_channel_count(
    audio_service: tauri::State<'_, Mutex<AudioService>>,
) -> Result<u16, String> {
    let audio = audio_service
        .lock()
        .map_err(|_| "Failed to lock audio service".to_string())?;
    Ok(audio.audio_handler().input_config().channels)
}

#[tauri::command]
pub fn get_selected_output_channel_count(
    audio_service: tauri::State<'_, Mutex<AudioService>>,
) -> Result<u16, String> {
    let audio = audio_service
        .lock()
        .map_err(|_| "Failed to lock audio service".to_string())?;
    Ok(audio.audio_handler().output_config().channels)
}

#[tauri::command]
pub fn set_asio_channel_config(
    device_service: tauri::State<DeviceService>,
    audio_service: tauri::State<'_, Mutex<AudioService>>,
    device_id: String,
    input_channels: u16,
    output_channels: u16,
) -> Result<(), String> {
    if !device_service.is_asio_selected() {
        return Err(
            "ASIO channel selection is only available when ASIO driver is selected".to_string(),
        );
    }

    let device = device_service
        .find_input_device_by_id(&device_id)
        .ok_or_else(|| "ASIO device not found".to_string())?;
    let (input_config, output_config) =
        build_asio_io_configs(&device, Some(input_channels), Some(output_channels))?;

    let mut audio = audio_service
        .lock()
        .map_err(|_| "Failed to lock audio service".to_string())?;
    audio.set_io_devices(device.clone(), device, input_config, output_config);
    Ok(())
}

#[tauri::command]
pub fn set_audio_driver(
    device_service: tauri::State<DeviceService>,
    audio_service: tauri::State<'_, Mutex<AudioService>>,
    driver: String,
) -> Result<(), String> {
    let previous_driver = device_service.selected_audio_driver();
    device_service.set_selected_audio_driver(&driver)?;

    let reconfigure_result: Result<(), String> = (|| {
        let (input_device, output_device) = device_service.default_devices_for_selected_driver()?;
        if device_service.is_asio_selected() {
            let (input_config, output_config) = build_asio_io_configs(&input_device, None, None)?;
            let mut audio = audio_service
                .lock()
                .map_err(|_| "Failed to lock audio service".to_string())?;
            audio.set_io_devices(
                input_device.clone(),
                output_device,
                input_config,
                output_config,
            );
            return Ok(());
        }

        let should_normalize = !device_service.is_asio_selected();
        let input_config = build_input_config(&input_device, should_normalize)?;
        let output_config = build_output_config(&output_device, should_normalize)?;

        let mut audio = audio_service
            .lock()
            .map_err(|_| "Failed to lock audio service".to_string())?;
        audio.set_io_devices(input_device, output_device, input_config, output_config);
        Ok(())
    })();

    if let Err(err) = reconfigure_result {
        let _ = device_service.set_selected_audio_driver(&previous_driver);
        return Err(err);
    }

    Ok(())
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
pub(crate) fn get_input_device_list(
    device_service: tauri::State<DeviceService>,
) -> Vec<AudioDeviceDto> {
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
pub(crate) fn get_output_device_list(
    device_service: tauri::State<DeviceService>,
) -> Vec<AudioDeviceDto> {
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

    if device_service.is_asio_selected() {
        return apply_asio_device_route(audio_service, device);
    }

    let device_name = device
        .description()
        .map(|d| d.name().to_string())
        .unwrap_or_else(|_| "Unknown".to_string());
    let input_config = build_input_config(&device, true)?;

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

    if device_service.is_asio_selected() {
        return apply_asio_device_route(audio_service, device);
    }

    let device_name = device
        .description()
        .map(|d| d.name().to_string())
        .unwrap_or_else(|_| "Unknown".to_string());
    let output_config = build_output_config(&device, true)?;

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
