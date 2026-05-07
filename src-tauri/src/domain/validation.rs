/// Sanitizes a user/config-supplied IR filename before it is used in a filesystem path.
///
/// Accepted filenames must:
/// - be non-empty after trimming whitespace,
/// - contain no path separators (`/`, `\\`) or parent-directory segments (`..`),
/// - end with `.wav` (case-insensitive).
///
/// Returns the trimmed filename on success, or a descriptive `Err` string.
pub fn sanitize_wav_file_name(file_name: &str) -> Result<String, String> {
    let trimmed = file_name.trim();

    if trimmed.is_empty() {
        return Err("IR file name cannot be empty".to_string());
    }

    if trimmed.contains('\\') || trimmed.contains('/') || trimmed.contains("..") {
        return Err("Invalid IR file name".to_string());
    }

    if !trimmed.to_ascii_lowercase().ends_with(".wav") {
        return Err("Only .wav IR files are supported".to_string());
    }

    Ok(trimmed.to_string())
}

