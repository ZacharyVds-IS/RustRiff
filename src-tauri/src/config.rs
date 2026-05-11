/// Application-wide configuration constants.
///
/// This module provides a single source of truth for configuration values
/// that are referenced throughout the application, ensuring consistency
///
/// between backend logic, frontend defaults, and bundled resources.
/// Default cabinet impulse-response WAV file name loaded when no explicit
/// profile is selected by the user.
///
/// **Important**: This filename must match one of the WAV files bundled in
/// `resources/default_ir/` so that newly created Cabinet effects work
/// out-of-the-box without falling back to passthrough mode.
pub const DEFAULT_IR_FILE: &str = "Vox-ac30.wav";
/// Filename used for the persisted amplifier configuration JSON document.
pub const AMP_CONFIG_FILE_NAME: &str = "amp-config.json";
/// Initializes the application's tracing subscriber for structured logging.
///
/// Reads the log filter from the `RUST_LOG` environment variable if present,
/// otherwise defaults to `info` level for all modules.
pub fn init_tracing() {
    use tracing_subscriber::EnvFilter;

    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();
}

/// Returns the default cabinet IR filename.
///
/// This Tauri command exposes the backend's `DEFAULT_IR_FILE` constant to
/// the frontend, ensuring the UI uses the same default as the backend logic.
#[tauri::command]
pub fn get_default_ir_file() -> String {
    DEFAULT_IR_FILE.to_string()
}
