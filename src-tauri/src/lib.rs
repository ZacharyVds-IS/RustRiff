pub mod commands;
pub mod services;
pub mod domain;
pub mod infrastructure;

#[cfg(test)]
pub mod tests;

use crate::commands::default_controls::{get_amp_config, set_bass, set_gain, set_master_volume, set_middle, set_treble, toggle_on_off};
use crate::commands::latency_testing::{
    measure_all_dsp_timings,
    test_fixed_delay_latency,
    test_gain_latency,
    test_tone_stack_latency,
};
use crate::commands::loopback::start_loopback;
use crate::commands::settings::{get_input_device_list, get_output_device_list, set_input_device, set_output_device};
use crate::services::audio_service::AudioService;
use crate::services::device_service::DeviceService;
use cpal::default_host;
use cpal::traits::{DeviceTrait, HostTrait};
use std::sync::Mutex;
use tracing_subscriber::EnvFilter;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    //TODO remove temporary fields for changing in and output devices.
    let host = default_host();
    let input = host.default_input_device().unwrap();
    let output = host.default_output_device().unwrap();
    let config = input.default_input_config().unwrap().config();

    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("info"))
        )
        .init();

    tauri::Builder::default()
        .manage(Mutex::new(AudioService::new(input,output,config)))
        .manage(DeviceService::new(host))
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![start_loopback, set_gain,get_input_device_list,get_output_device_list,set_input_device,set_output_device, set_master_volume, toggle_on_off, get_amp_config, set_bass, set_middle, set_treble, test_gain_latency, test_tone_stack_latency, test_fixed_delay_latency, measure_all_dsp_timings])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
