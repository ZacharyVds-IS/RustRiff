pub mod commands;
pub mod config;
pub mod domain;
pub mod infrastructure;
pub mod services;

#[cfg(test)]
pub mod tests;

use crate::commands::analyzer::{
    get_live_spectrum, get_spectrum_contract, start_live_spectrum_stream,
    stop_live_spectrum_stream, SpectrumStreamState,
};
use crate::commands::channels::{
    add_channel, get_all_channels, get_channel_id, remove_channel, set_channel_id,
};
use crate::commands::default_controls::{
    get_amp_config, set_bass, set_gain, set_master_volume, set_middle, set_tone_stack, set_treble,
    set_volume, toggle_on_off,
};
use crate::commands::effect_commands::cabinet_ir::{
    get_all_ir_profiles, remove_ir_profile, upload_ir_profile,
};
use crate::commands::effect_commands::delay::{set_delay_delay_time, set_delay_level};
use crate::commands::effect_commands::hc_distortion::{
    set_hc_distortion_level, set_hc_distortion_threshold,
};
use crate::commands::effect_commands::sc_distortion::{
    set_sc_distortion_level, set_sc_distortion_smoothing, set_sc_distortion_threshold,
};
use crate::commands::latency_testing::{
    measure_all_dsp_algorithmic_latency, measure_all_dsp_cpu_timings, measure_buffer_latency,
    measure_round_trip_latency, test_gain_latency,
};
use crate::commands::loopback::start_loopback;
use crate::commands::settings::{
    get_buffer_size_frames, get_input_device_list, get_output_device_list, set_buffer_size_frames,
    set_input_device, set_output_device,
};
use crate::config::{get_default_ir_file, init_tracing};
use crate::infrastructure::file_loader::FileLoader;
use crate::infrastructure::persistence::json_amp_config_repository::JsonFileAmpConfigRepository;
use crate::services::amp_config_service::AmpConfigPersistenceService;
use crate::services::audio_service::AudioService;
use crate::services::device_service::DeviceService;
use crate::services::file_service::FileService;
use commands::effect_commands::effects::{
    add_effect, apply_effect_order_change, remove_effect, toggle_effect,
};
use cpal::default_host;
use cpal::traits::{DeviceTrait, HostTrait};
use cpal::{BufferSize, StreamConfig};
use std::sync::Mutex;
use tauri::Manager;
use tracing::{error, info};

const AMP_CONFIG_FILE_NAME: &str = "amp-config.json";

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    init_tracing();

    let host = default_host();
    let input = host.default_input_device().unwrap();
    let output = host.default_output_device().unwrap();

    let input_name = input
        .description()
        .map(|d| d.name().to_string())
        .unwrap_or_else(|_| "Unknown".to_string());
    let output_name = output
        .description()
        .map(|d| d.name().to_string())
        .unwrap_or_else(|_| "Unknown".to_string());

    info!("Input device: {}", input_name);
    info!("Output device: {}", output_name);

    let input_supported = input
        .default_input_config()
        .map_err(|e| format!("Failed to get input device config: {}", e))
        .unwrap();
    let output_supported = output
        .default_output_config()
        .map_err(|e| format!("Failed to get output device config: {}", e))
        .unwrap();
    info!(
        "Input config - Channels: {}, Sample Rate: {} Hz",
        input_supported.channels(),
        input_supported.sample_rate()
    );
    info!(
        "Output config - Channels: {}, Sample Rate: {} Hz",
        output_supported.channels(),
        output_supported.sample_rate()
    );

    let normalize_channels = |channels: u16| -> u16 {
        match channels {
            0 => {
                error!("Device reported 0 channels, defaulting to stereo");
                2
            }
            1 => 1,
            _ => {
                if channels > 2 {
                    info!(
                        "Device reported {} channels, normalizing to stereo for stability",
                        channels
                    );
                    2
                } else {
                    channels
                }
            }
        }
    };

    let input_channels = normalize_channels(input_supported.channels());
    let output_channels = normalize_channels(output_supported.channels());

    let input_config = StreamConfig {
        channels: input_channels,
        sample_rate: input_supported.sample_rate(),
        buffer_size: BufferSize::Default,
    };
    let output_config = StreamConfig {
        channels: output_channels,
        sample_rate: output_supported.sample_rate(),
        buffer_size: BufferSize::Default,
    };

    info!(
        "Configured stream - Input: {} ch @ {} Hz, Output: {} ch @ {} Hz",
        input_config.channels,
        input_config.sample_rate,
        output_config.channels,
        output_config.sample_rate
    );

    let audio_service = AudioService::new(input, output, input_config, output_config);

    tauri::Builder::default()
        .manage(Mutex::new(audio_service))
        .manage(SpectrumStreamState::default())
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

            let resource_root = app.path().resource_dir().unwrap_or_else(|_| {
                std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("resources")
            });
            info!("Using resource root: {}", resource_root.display());

            let custom_ir_directory = config_dir.join("default_ir_custom");
            info!(
                "Using custom IR directory: {}",
                custom_ir_directory.display()
            );
            std::env::set_var("RUSTRIFF_CUSTOM_IR_DIR", &custom_ir_directory);

            let file_service = FileService::new(
                Box::new(FileLoader::new()),
                resource_root,
                custom_ir_directory,
            );
            app.manage(file_service);

            app.manage(Mutex::new(amp_config_persistence_service));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_default_ir_file,
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
            get_all_ir_profiles,
            upload_ir_profile,
            remove_ir_profile,
            set_delay_delay_time,
            set_delay_level,
            get_live_spectrum,
            get_spectrum_contract,
            start_live_spectrum_stream,
            stop_live_spectrum_stream,
            set_sc_distortion_threshold,
            set_sc_distortion_level,
            set_sc_distortion_smoothing,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
