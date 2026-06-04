use crate::domain::dto::spectrum_contract_dto::SpectrumContractDto;
use crate::domain::dto::spectrum_snapshot_dto::SpectrumSnapshotDto;
use crate::services::analyzers::spectrum_analyzer_service::SpectrumAnalyzerService;
use crate::services::analyzers::spectrum_tap::SpectrumTap;
use crate::services::audio_service::AudioService;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use tauri::Emitter;
use tokio::time::{interval, Duration};

/// Tauri event name emitted by the backend when a new spectrum frame is available.
const LIVE_SPECTRUM_EVENT: &str = "live-spectrum";
/// Target interval for push-streamed spectrum frames (about 60 FPS).
const STREAM_INTERVAL_MS: u64 = 16;

/// Runtime handle for one live-spectrum stream task.
///
/// Each task owns its own shutdown flag to avoid races when a new stream starts
/// before the previous task has observed cancellation.
struct StreamTask {
    handle: tauri::async_runtime::JoinHandle<()>,
    shutdown: Arc<AtomicBool>,
}

/// Shared state for the analyzer stream task.
///
/// The task is started by `start_live_spectrum_stream` and stopped by either
/// `stop_live_spectrum_stream` or when the target window can no longer receive events.
#[derive(Default)]
pub struct SpectrumStreamState {
    task: Mutex<Option<StreamTask>>,
}

/// Lower bound for analyzer frequencies in Hz, shared with frontend chart config.
const MIN_ANALYZER_FREQ_HZ: f32 = 20.0;
/// Upper frequency bound pushed to frontend chart config.
const MAX_ANALYZER_FREQ_HZ: f32 = 20_000.0;
/// Lower clamp for displayed magnitudes (dBFS), shared with frontend chart config.
const MIN_DB: f32 = -90.0;
/// Upper clamp for displayed magnitudes (dBFS), shared with frontend chart config.
const MAX_DB: f32 = 6.0;

/// Returns a single, immediate spectrum snapshot.
///
/// This command is useful for first paint / fallback reads before the push stream
/// starts delivering `live-spectrum` events.
///
/// FFT analysis is offloaded to a blocking task so the async command handler never
/// stalls the Tauri runtime thread.
#[tauri::command]
pub async fn get_live_spectrum(
    audio_service: tauri::State<'_, Mutex<AudioService>>,
) -> Result<SpectrumSnapshotDto, String> {
    let tap = {
        let audio_service = audio_service
            .lock()
            .map_err(|_| "Failed to lock audio service".to_string())?;
        audio_service.analyzer_tap().clone()
    };

    tauri::async_runtime::spawn_blocking(move || SpectrumAnalyzerService::analyze_tap(tap.as_ref()))
        .await
        .map_err(|e| format!("FFT analysis task failed: {e}"))
}

/// Returns static analyzer metadata consumed by frontend chart/state code.
#[tauri::command]
pub fn get_spectrum_contract() -> SpectrumContractDto {
    SpectrumContractDto {
        live_spectrum_event: LIVE_SPECTRUM_EVENT.to_string(),
        min_db: MIN_DB,
        max_db: MAX_DB,
        min_frequency_hz: MIN_ANALYZER_FREQ_HZ,
        max_frequency_hz: MAX_ANALYZER_FREQ_HZ,
    }
}

/// Starts (or restarts) push-based live spectrum streaming for the calling window.
///
/// Behavior:
/// - Captures the current shared `SpectrumTap` from `AudioService`.
/// - Signals any previously running stream task to shut down using that task's own
///   cancellation flag, then replaces it.
/// - Spawns a background loop that analyzes the tap and emits `live-spectrum`
///   events at `STREAM_INTERVAL_MS` cadence.
/// - Automatically exits when event emission fails (for example, when window closes).
#[tauri::command]
pub fn start_live_spectrum_stream(
    window: tauri::Window,
    audio_service: tauri::State<'_, Mutex<AudioService>>,
    stream_state: tauri::State<'_, SpectrumStreamState>,
) -> Result<(), String> {
    let tap: Arc<_> = {
        let audio_service = audio_service
            .lock()
            .map_err(|_| "Failed to lock audio service".to_string())?;
        audio_service.analyzer_tap().clone()
    };

    let shutdown = Arc::new(AtomicBool::new(false));

    {
        let mut guard = stream_state
            .task
            .lock()
            .map_err(|_| "Failed to lock spectrum stream state".to_string())?;

        if let Some(previous) = guard.take() {
            previous.shutdown.store(true, Ordering::Relaxed);
            previous.handle.abort();
        }

        let task_shutdown = Arc::clone(&shutdown);
        let handle = tauri::async_runtime::spawn(async move {
            let mut ticker = interval(Duration::from_millis(STREAM_INTERVAL_MS));
            loop {
                ticker.tick().await;
                if task_shutdown.load(Ordering::Relaxed) {
                    break;
                }

                let tap_ref = Arc::clone(&tap) as Arc<SpectrumTap>;
                let snapshot = tauri::async_runtime::spawn_blocking(move || {
                    SpectrumAnalyzerService::analyze_tap(tap_ref.as_ref())
                })
                .await;

                match snapshot {
                    Ok(data) => {
                        if window.emit(LIVE_SPECTRUM_EVENT, data).is_err() {
                            break;
                        }
                    }
                    Err(_) => break,
                }
            }
        });

        guard.replace(StreamTask { handle, shutdown });
    }

    Ok(())
}

/// Stops the active live spectrum stream task, if one exists.
///
/// This is safe to call repeatedly; when no task is active it becomes a no-op.
/// The active task is signaled to stop using its own cancellation flag.
#[tauri::command]
pub fn stop_live_spectrum_stream(
    stream_state: tauri::State<'_, SpectrumStreamState>,
) -> Result<(), String> {
    if let Some(task) = stream_state
        .task
        .lock()
        .map_err(|_| "Failed to lock spectrum stream state".to_string())?
        .take()
    {
        task.shutdown.store(true, Ordering::Relaxed);
        task.handle.abort();
    }

    Ok(())
}
