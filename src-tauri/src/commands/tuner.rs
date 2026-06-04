use crate::config::LIVE_TUNER_EVENT;
use crate::domain::dto::tuner_contract_dto::TunerContractDto;
use crate::services::audio_service::AudioService;
use crate::services::tuner_service::TunerService;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use tauri::Emitter;
use tokio::time::{interval, Duration};

/// Target interval for push-streamed tuner updates (about 60 FPS).
const TUNER_STREAM_INTERVAL_MS: u64 = 16;

struct TunerStreamTask {
    handle: tauri::async_runtime::JoinHandle<()>,
    shutdown: Arc<AtomicBool>,
}

/// Shared state for managing the active live tuner background processing thread.
#[derive(Default)]
pub struct TunerStreamState {
    task: Mutex<Option<TunerStreamTask>>,
}

/// Returns static configuration context for the tuner screen.
#[tauri::command]
pub fn get_tuner_contract() -> TunerContractDto {
    TunerContractDto {
        live_tuner_event: LIVE_TUNER_EVENT.to_string(),
    }
}

/// Starts the live pitch detection processing loop.
///
/// It captures the audio tap from your shared AudioService, aborts any existing
/// tuner thread, and begins pushing updates directly to the open window.
#[tauri::command]
pub fn start_live_tuner_stream(
    window: tauri::Window,
    audio_service: tauri::State<'_, Mutex<AudioService>>,
    stream_state: tauri::State<'_, TunerStreamState>,
) -> Result<(), String> {
    let tap = {
        let mut audio_service = audio_service
            .lock()
            .map_err(|_| "Failed to lock audio service".to_string())?;
        audio_service.set_tuner_active(true);
        audio_service.tuner_tap().clone()
    };

    let shutdown = Arc::new(AtomicBool::new(false));

    {
        let mut guard = stream_state
            .task
            .lock()
            .map_err(|_| "Failed to lock tuner stream state".to_string())?;

        if let Some(previous) = guard.take() {
            previous.shutdown.store(true, Ordering::Relaxed);
            previous.handle.abort();
        }

        let task_shutdown = Arc::clone(&shutdown);
        let handle = tauri::async_runtime::spawn(async move {
            let mut ticker = interval(Duration::from_millis(TUNER_STREAM_INTERVAL_MS));

            loop {
                ticker.tick().await;
                if task_shutdown.load(Ordering::Relaxed) {
                    break;
                }

                let tap_ref = Arc::clone(&tap);

                let snapshot = tauri::async_runtime::spawn_blocking(move || {
                    TunerService::detect_pitch(tap_ref.as_ref())
                })
                .await;

                match snapshot {
                    Ok(data) => {
                        // Dispatch payload to the frontend. Exits loop early if window collapses.
                        if window.emit(LIVE_TUNER_EVENT, data).is_err() {
                            break;
                        }
                    }
                    Err(_) => break,
                }
            }
        });

        guard.replace(TunerStreamTask { handle, shutdown });
    }

    Ok(())
}

/// Discontinues the audio evaluation background stream for the tuner.
#[tauri::command]
pub fn stop_live_tuner_stream(
    audio_service: tauri::State<'_, Mutex<AudioService>>,
    stream_state: tauri::State<'_, TunerStreamState>,
) -> Result<(), String> {
    let mut audio_service = audio_service
        .lock()
        .map_err(|_| "Failed to lock audio service".to_string())?;
    audio_service.set_tuner_active(false);
    if let Some(task) = stream_state
        .task
        .lock()
        .map_err(|_| "Failed to lock tuner stream state".to_string())?
        .take()
    {
        task.shutdown.store(true, Ordering::Relaxed);
        task.handle.abort();
    }

    Ok(())
}
