pub mod commands;
pub mod domain;
pub mod infrastructure;
pub mod services;

#[cfg(test)]
pub mod tests;

use crate::commands::channels::{add_channel, get_all_channels, get_channel_id, remove_channel, set_channel_id};
use crate::commands::default_controls::{get_amp_config, set_bass, set_gain, set_master_volume, set_middle, set_tone_stack, set_treble, set_volume, toggle_on_off};
use crate::commands::effects::{add_effect, apply_effect_order_change, remove_effect, set_hc_distortion_level, set_hc_distortion_threshold, toggle_effect};
use crate::commands::latency_testing::{measure_all_dsp_algorithmic_latency, measure_all_dsp_cpu_timings, measure_buffer_latency, measure_round_trip_latency, test_gain_latency};
use crate::commands::loopback::start_loopback;
use crate::commands::settings::{get_buffer_size_frames, get_input_device_list, get_output_device_list, set_buffer_size_frames, set_input_device, set_output_device};
use crate::infrastructure::file_loader::FileLoader;
use crate::infrastructure::persistence::json_amp_config_repository::JsonFileAmpConfigRepository;
use crate::services::amp_config_service::AmpConfigPersistenceService;
use crate::services::audio_service::AudioService;
use crate::services::device_service::DeviceService;
use cpal::default_host;
use cpal::traits::{DeviceTrait, HostTrait};
use cpal::{BufferSize, StreamConfig};
use std::sync::Mutex;
use tauri::Manager;
use tracing::{error, info};
use tracing_subscriber::EnvFilter;

const AMP_CONFIG_FILE_NAME: &str = "amp-config.json";

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();

    let host = default_host();
    let input = host.default_input_device().unwrap();
    let output = host.default_output_device().unwrap();

    let input_supported = input.default_input_config().unwrap();
    let output_supported = output.default_output_config().unwrap();

    let input_config = StreamConfig {
        channels: input_supported.channels(),
        sample_rate: input_supported.sample_rate(),
        buffer_size: BufferSize::Default,
    };
    let output_config = StreamConfig {
        channels: output_supported.channels(),
        sample_rate: output_supported.sample_rate(),
        buffer_size: BufferSize::Default,
    };

    let audio_service = AudioService::new(input, output, input_config, output_config);


    tauri::Builder::default()
        .manage(Mutex::new(audio_service))
        .manage(DeviceService::new(host))
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let config_dir = app
                .path()
                .app_config_dir()
                .or_else(|_| app.path().app_data_dir())
                .map_err(|e| format!("Failed to resolve app config/data directory: {e}"))?;

            let config_path = config_dir.join(AMP_CONFIG_FILE_NAME);
            info!("Using persisted amp config path: {}", config_path.display());

            let amp_config_persistence_service = AmpConfigPersistenceService::new(Box::new(
                JsonFileAmpConfigRepository::new(config_path),
            ));

            {
                let audio_service_state = app.state::<Mutex<AudioService>>();
                let mut audio_service = audio_service_state
                    .lock()
                    .map_err(|_| "Failed to lock audio service during startup")?;

                match amp_config_persistence_service.load_amp_config() {
                    Ok(Some(config)) => {
                        info!("Loaded persisted amplifier configuration");
                        audio_service.apply_amp_config(config);
                    }
                    Ok(None) => info!("No persisted amplifier configuration found"),
                    Err(err) => error!("Failed to load persisted amplifier configuration: {err}"),
                }
            }

            app.manage(Mutex::new(amp_config_persistence_service));
            Ok(())
        })
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
            set_tone_stack,
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
            add_effect,
            remove_effect,
            apply_effect_order_change,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
