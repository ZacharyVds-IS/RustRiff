//! Tauri commands that expose latency measurement to the frontend.
//!
//! Each command in this module is a thin adapter over
//! [`AudioLatencyMeasurementService`].  They are responsible for:
//!
//! - Locking (or releasing) the [`AudioService`] mutex at the right time.
//! - Logging results via `tracing` so they appear in the backend console.
//! - Returning serialisable DTOs across the IPC boundary.
//!
//! # Command overview
//!
//! | Command | Tauri invoke name | What it measures |
//! |---|---|---|
//! | [`test_gain_latency`] | `test_gain_latency` | Gain processor CPU cost (logs only, no return value) |
//! | [`measure_all_dsp_cpu_timings`] | `measure_all_dsp_cpu_timings` | CPU cost of all DSP processors |
//! | [`measure_all_dsp_algorithmic_latency`] | `measure_all_dsp_algorithmic_latency` | Algorithmic (design) delay per processor |
//! | [`measure_buffer_latency`] | `measure_buffer_latency` | I/O buffer delay from stream config |
//! | [`measure_round_trip_latency`] | `measure_round_trip_latency` | True end-to-end hardware round-trip |
//!
//! [`AudioLatencyMeasurementService`]: crate::services::audio_latency_measurement_service::AudioLatencyMeasurementService
//! [`AudioService`]: crate::services::audio_service::AudioService

use crate::domain::dto::algorithmic_latency_dto::AlgorithmicLatencyDto;
use crate::domain::dto::buffer_latency_dto::BufferLatencyDto;
use crate::domain::dto::execution_timing_dto::ExecutionTimingDto;
use crate::domain::dto::round_trip_latency_dto::RoundTripLatencyDto;
use crate::infrastructure::audio_handler::AudioHandlerTrait;
use crate::services::audio_latency_measurement_service::AudioLatencyMeasurementService;
use crate::services::audio_service::AudioService;
use crate::services::device_service::DeviceService;
use std::sync::{Arc, Mutex};
use tracing::info;

/// Measures the CPU execution impact of the [`GainProcessor`] and logs the result.
///
/// This command is intended for quick developer diagnostics — it prints the measurement
/// to the backend log but does **not** return a value to the frontend.  Use
/// [`measure_all_dsp_cpu_timings`] when you need a structured result.
///
/// The measurement uses a block size of 2 048 samples, which is large enough to
/// suppress timer-call overhead while completing in well under a second.
///
/// # Errors
///
/// Returns `Err` if the [`AudioService`] mutex cannot be locked.
///
/// [`GainProcessor`]: crate::services::processors::gain::gain_processor::GainProcessor
/// [`AudioService`]: crate::services::audio_service::AudioService
#[tauri::command]
pub fn test_gain_latency(
    audio_service: tauri::State<'_, Mutex<AudioService>>,
) -> Result<(), String> {
    let audio_service = audio_service
        .lock()
        .map_err(|_| "Failed to lock audio service".to_string())?;

    let added_us_per_sample =
        AudioLatencyMeasurementService::measure_gain_latency(&audio_service, 2048);

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

/// Measures the CPU execution cost of every processor in the active DSP chain.
///
/// Benchmarks the Gain, Tone Stack, and Master Volume processors in signal-chain order
/// using a block size of 2 048 samples.  Each result is the *net* added cost relative
/// to a zero-work passthrough, clamped to ≥ 0 µs/sample.
///
/// Results are both logged at `info` level and returned to the frontend as a
/// `Vec<ExecutionTimingDto>`.
///
/// # Returns
///
/// `Ok(timings)` — a vector of exactly three [`ExecutionTimingDto`] entries in
/// signal-chain order: Gain → Tone Stack → Master Volume.
///
/// # Errors
///
/// Returns `Err` if the [`AudioService`] mutex cannot be locked.
///
/// [`ExecutionTimingDto`]: crate::domain::dto::execution_timing_dto::ExecutionTimingDto
/// [`AudioService`]: crate::services::audio_service::AudioService
#[tauri::command]
pub fn measure_all_dsp_cpu_timings(
    audio_service: tauri::State<'_, Mutex<AudioService>>,
) -> Result<Vec<ExecutionTimingDto>, String> {
    let audio_service = audio_service
        .lock()
        .map_err(|_| "Failed to lock audio service".to_string())?;

    let timings = AudioLatencyMeasurementService::measure_all_dsp_timings(&audio_service, 2048);

    for timing in &timings {
        info!(
            processor = timing.processor_name,
            execution_us_per_sample = timing.execution_us_per_sample,
            "DSP chain processor timing"
        );
    }

    Ok(timings)
}

/// Returns the algorithmic (design-inherent) delay for every processor in the DSP chain.
///
/// Algorithmic latency is the sample delay an effect introduces by design — e.g. a
/// look-ahead limiter adds a fixed number of samples regardless of CPU speed.
///
/// For the current chain (Gain → Tone Stack → Master Volume) all values are **zero**
/// because no processor uses a delay line or look-ahead buffer.  This command still
/// exists to provide the correct data shape for the developer UI and to remain correct
/// when future processors with non-zero delay are added.
///
/// Results are both logged at `info` level and returned to the frontend.
///
/// # Returns
///
/// `Ok(latency)` — a vector of exactly three [`AlgorithmicLatencyDto`] entries:
/// Gain → Tone Stack → Master Volume, each with `latency_samples = 0`.
///
/// # Errors
///
/// Returns `Err` if the [`AudioService`] mutex cannot be locked.
///
/// [`AlgorithmicLatencyDto`]: crate::domain::dto::algorithmic_latency_dto::AlgorithmicLatencyDto
/// [`AudioService`]: crate::services::audio_service::AudioService
#[tauri::command]
pub fn measure_all_dsp_algorithmic_latency(
    audio_service: tauri::State<'_, Mutex<AudioService>>,
) -> Result<Vec<AlgorithmicLatencyDto>, String> {
    let audio_service = audio_service
        .lock()
        .map_err(|_| "Failed to lock audio service".to_string())?;

    let latency =
        AudioLatencyMeasurementService::measure_all_dsp_algorithmic_latency(&audio_service);

    for item in &latency {
        info!(
            processor = item.processor_name,
            latency_samples = item.latency_samples,
            latency_ms = item.latency_ms,
            "DSP chain processor algorithmic latency"
        );
    }

    Ok(latency)
}

/// Estimates the I/O buffer latency from the current CPAL stream configuration.
///
/// Reads the configured frame count for both the input and output streams and converts
/// them to milliseconds using `(frames / sample_rate) × 1000`.  If either stream uses
/// [`BufferSize::Default`], a fallback of 256 frames is used.
///
/// The result is logged at `info` level and returned to the frontend.
///
/// # Returns
///
/// `Ok(latency)` — a [`BufferLatencyDto`] with `input_buffer_latency_ms`,
/// `output_buffer_latency_ms`, and `total_buffer_latency_ms`.
///
/// # Errors
///
/// Returns `Err` if the [`AudioService`] mutex cannot be locked.
///
/// [`BufferSize::Default`]: cpal::BufferSize::Default
/// [`BufferLatencyDto`]: crate::domain::dto::buffer_latency_dto::BufferLatencyDto
/// [`AudioService`]: crate::services::audio_service::AudioService
#[tauri::command]
pub fn measure_buffer_latency(
    audio_service: tauri::State<'_, Mutex<AudioService>>,
) -> Result<BufferLatencyDto, String> {
    let audio_service = audio_service
        .lock()
        .map_err(|_| "Failed to lock audio service".to_string())?;

    let latency = AudioLatencyMeasurementService::measure_buffer_latency(&audio_service);

    info!(
        input_buffer_latency_ms = latency.input_buffer_latency_ms,
        output_buffer_latency_ms = latency.output_buffer_latency_ms,
        total_buffer_latency_ms = latency.total_buffer_latency_ms,
        "I/O buffer latency"
    );

    Ok(latency)
}

/// Measures true end-to-end round-trip latency using dedicated CPAL streams.
///
/// This is the only latency command that performs a **real hardware measurement** rather
/// than an analytical estimate.  The procedure is:
///
/// 1. The [`AudioService`] mutex is locked just long enough to clone the handler arc,
///    then **released** so the main loopback and UI remain unblocked.
/// 2. A dedicated OS thread is spawned that calls
///    [`AudioLatencyMeasurementService::measure_round_trip_latency`], which in turn
///    opens its own CPAL input/output streams, runs calibration and impulse detection,
///    and returns the averaged result.
/// 3. The command blocks until the thread finishes (typically 3–15 s depending on
///    warmup and timeout settings), then logs and returns the result.
///
/// # Physical requirement
///
/// The audio interface output must be physically (or virtually) connected back to its
/// input.  Without this loopback the impulse can never be detected and the measurement
/// will time out.
///
/// # Returns
///
/// `Ok(result)` — a [`RoundTripLatencyDto`] with:
/// - `is_valid = true` and `latency_ms` set on success.
/// - `is_valid = false` and a human-readable `error` on failure (e.g. timeout or no echo).
///
/// # Errors
///
/// Returns `Err` only if the [`AudioService`] mutex cannot be locked or the
/// measurement thread panics unexpectedly.
///
/// [`AudioService`]: crate::services::audio_service::AudioService
/// [`AudioLatencyMeasurementService::measure_round_trip_latency`]: crate::services::audio_latency_measurement_service::AudioLatencyMeasurementService::measure_round_trip_latency
/// [`RoundTripLatencyDto`]: crate::domain::dto::round_trip_latency_dto::RoundTripLatencyDto
#[tauri::command]
pub fn measure_round_trip_latency(
    device_service: tauri::State<DeviceService>,
    audio_service: tauri::State<'_, Mutex<AudioService>>,
) -> Result<RoundTripLatencyDto, String> {
    // ASIO is an exclusive driver — only one host can own the device at a time.
    // If the main loopback is active we must stop it before the measurement opens its own
    // streams, then restart it afterwards. On non-ASIO hosts both streams can co-exist so
    // we follow the original lighter path (clone handler, release lock, measure).
    let is_asio = device_service.is_asio_selected();

    let (handler, was_active) = {
        let mut audio_service = audio_service
            .lock()
            .map_err(|_| "Failed to lock audio service".to_string())?;

        let was_active = *audio_service.is_active();

        if is_asio && was_active {
            info!("ASIO is exclusive: stopping loopback before round-trip measurement");
            audio_service.stop_loopback();
        }

        let handler: Arc<dyn AudioHandlerTrait> = audio_service.audio_handler().clone();
        (handler, was_active)
    };

    if is_asio && was_active {
        std::thread::sleep(std::time::Duration::from_millis(300));
    }

    let latency = std::thread::spawn(move || {
        AudioLatencyMeasurementService::measure_round_trip_latency(handler.as_ref())
    })
    .join()
    .map_err(|_| "Round-trip measurement thread panicked".to_string())?;

    // Restore the loopback for ASIO after measurement streams have been dropped.
    if is_asio && was_active {
        if let Ok(mut audio_service) = audio_service.lock() {
            info!("Restarting loopback after ASIO round-trip measurement");
            std::thread::sleep(std::time::Duration::from_millis(150));
            audio_service.start_loopback();
        }
    }

    if latency.is_valid {
        info!(
            round_trip_latency_ms = latency.latency_ms,
            "Round-trip latency measurement"
        );
    } else {
        info!(
            error = latency.error.clone(),
            "Round-trip latency measurement failed"
        );
    }

    Ok(latency)
}
