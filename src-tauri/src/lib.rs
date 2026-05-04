pub mod commands;
pub mod services;
pub mod domain;
pub mod infrastructure;

#[cfg(test)]
pub mod tests;

use crate::commands::channels::{add_channel, get_all_channels, get_channel_id, remove_channel, set_channel_id};
use crate::commands::default_controls::{get_amp_config, set_bass, set_gain, set_master_volume, set_middle, set_treble, set_volume, toggle_on_off};
use crate::commands::effects::{set_hc_distortion_level, set_hc_distortion_threshold, toggle_effect};
use crate::commands::latency_testing::{measure_all_dsp_algorithmic_latency, measure_all_dsp_cpu_timings, measure_buffer_latency, measure_round_trip_latency, test_gain_latency};
use crate::commands::loopback::start_loopback;
use crate::commands::settings::{get_buffer_size_frames, get_input_device_list, get_output_device_list, set_buffer_size_frames, set_input_device, set_output_device};
use crate::services::audio_service::AudioService;
use crate::services::device_service::DeviceService;
use cpal::default_host;
use cpal::traits::{DeviceTrait, HostTrait};
use cpal::{BufferSize, StreamConfig};
use std::sync::Mutex;
use tracing_subscriber::EnvFilter;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let host = default_host();
    let input = host.default_input_device().unwrap();
    let output = host.default_output_device().unwrap();

    let input_supported = input.default_input_config().unwrap();
    let output_supported = output.default_output_config().unwrap();

    // Use the input sample rate for both streams to avoid conflicts with ASIO
    // devices that require a shared sample rate across input and output.
    // Always use BufferSize::Default so the driver can negotiate a safe buffer size.
    let sample_rate = input_supported.sample_rate();
    let input_config = StreamConfig {
        channels: input_supported.channels(),
        sample_rate,
        buffer_size: BufferSize::Default,
    };
    let output_config = StreamConfig {
        channels: output_supported.channels(),
        sample_rate,
        buffer_size: BufferSize::Default,
    };

    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("info"))
        )
        .init();

    tauri::Builder::default()
        .manage(Mutex::new(AudioService::new(input, output, input_config, output_config)))
        .manage(DeviceService::new(host))
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            start_loopback,
            set_gain,
            get_input_device_list,
            get_output_device_list,
            set_input_device,
            set_output_device,
            set_master_volume,
            toggle_on_off,
            get_amp_config,
            set_bass,
            set_middle,
            set_treble,
            set_volume,
            set_channel_id,
            get_channel_id,
            add_channel,
            get_all_channels,
            remove_channel,
            get_buffer_size_frames,
            set_buffer_size_frames,
            test_gain_latency,
            measure_all_dsp_cpu_timings,
            measure_all_dsp_algorithmic_latency,
            measure_buffer_latency,
            measure_round_trip_latency,
            toggle_effect,
            set_hc_distortion_threshold,
            set_hc_distortion_level,
        ]).run(tauri::generate_context!())
        .expect("error while running tauri application");
}
