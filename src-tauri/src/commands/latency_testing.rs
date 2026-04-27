use crate::domain::execution_timing_dto::ExecutionTimingDto;
use crate::services::audio_service::AudioService;
use std::sync::Mutex;
use tracing::info;

/// Measures gain processor execution impact in microseconds per sample.
#[tauri::command]
pub fn test_gain_latency(audio_service: tauri::State<'_, Mutex<AudioService>>) -> Result<(), String> {
    let service = audio_service
        .lock()
        .map_err(|_| "Failed to lock audio service".to_string())?;

    let added_us_per_sample = service.measure_gain_latency(2048);

    info!(
        "Gain processor execution impact: {:.6} µs/sample",
        added_us_per_sample
    );
    println!(
        "Gain processor execution impact: {:.6} µs/sample",
        added_us_per_sample
    );

    Ok(())
}

/// Measures execution impact of all processors in the DSP chain.
///
/// Returns a vector of timing measurements in chain order:
/// 1. Input Latency (buffer-based I/O latency)
/// 2. Gain
/// 3. Tone Stack
/// 4. Master Volume
/// 5. Output Latency (buffer-based I/O latency)
#[tauri::command]
pub fn measure_all_dsp_timings(
    audio_service: tauri::State<'_, Mutex<AudioService>>,
) -> Result<Vec<ExecutionTimingDto>, String> {
    let service = audio_service
        .lock()
        .map_err(|_| "Failed to lock audio service".to_string())?;

    let timings = service.measure_all_dsp_timings(2048);

    for timing in &timings {
        info!(
            processor = timing.processor_name,
            execution_us_per_sample = timing.execution_us_per_sample,
            "DSP chain processor timing"
        );
    }

    Ok(timings)
}

//TODO:Remove unused commands.
#[tauri::command]
pub fn test_tone_stack_latency(audio_service: tauri::State<'_, Mutex<AudioService>>) -> Result<(), String> {
    let service = audio_service
        .lock()
        .map_err(|_| "Failed to lock audio service".to_string())?;

    let added_us_per_sample = service.measure_tone_stack_latency(2048);

    info!(
        "Tone stack processor execution impact: {:.6} µs/sample",
        added_us_per_sample
    );
    println!(
        "Tone stack processor execution impact: {:.6} µs/sample",
        added_us_per_sample
    );

    Ok(())
}

/// Measures execution impact for a fixed-delay processor.
///
/// This command is a diagnostic sanity check for the time-based analyzer itself.
#[tauri::command]
pub fn test_fixed_delay_latency(audio_service: tauri::State<'_, Mutex<AudioService>>) -> Result<(), String> {
    let service = audio_service
        .lock()
        .map_err(|_| "Failed to lock audio service".to_string())?;

    let configured_delay_samples = 128usize;
    let analysis_block_size = 2048usize;
    let added_us_per_sample = service.measure_fixed_delay_latency(
        configured_delay_samples,
        analysis_block_size,
    );

    info!(
        "Fixed-delay processor execution impact: configured_delay={} samples, {:.6} µs/sample",
        configured_delay_samples,
        added_us_per_sample
    );
    println!(
        "Fixed-delay processor execution impact: configured_delay={} samples, {:.6} µs/sample",
        configured_delay_samples,
        added_us_per_sample
    );

    Ok(())
}

