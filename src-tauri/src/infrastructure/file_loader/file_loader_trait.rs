use std::path::Path;

/// Abstraction over filesystem operations required by the IR profile pipeline.
///
/// The trait is `Send + Sync` so it can be injected into services that may be
/// shared across threads (e.g. Tauri's managed state).
///
/// A real implementation based on [`std::fs`] and the [`hound`] WAV crate is
/// provided by [`FileLoader`].
///
/// [`FileLoader`]: super::FileLoader
pub trait FileLoaderTrait: Send + Sync {
    /// Reads the sample rate declared in the WAV header at `path`.
    /// Returns `None` if the file cannot be opened or parsed (missing, corrupt,
    /// unsupported format). The caller should treat `None` as "assume DSP rate
    /// and skip resampling".
    fn read_wav_sample_rate(&self, path: &Path) -> Option<u32>;

    /// Decodes a WAV file into a flat mono `f32` sample buffer.
    ///
    /// **Multi-channel handling**: Stereo and multi-channel WAV files are
    /// automatically downmixed to mono by averaging all channels in each frame.
    /// For example, a stereo file with interleaved `[L0, R0, L1, R1]` samples
    /// produces `[(L0+R0)/2, (L1+R1)/2]`.
    ///
    /// **Normalization**: Integer samples are normalized to `[-1.0, 1.0]` using
    /// the full integer range determined by `bits_per_sample`.
    ///
    /// Returns an empty `Vec` and logs a warning on any I/O or parse error so
    /// that the caller can fall back to passthrough rather than panicking.
    fn read_wav_to_buffer(&self, path: &Path) -> Vec<f32>;

    /// Returns a sorted list of `.wav` filenames (not full paths) found in
    /// `directory`.
    ///
    /// Only regular files with a `.wav` extension (case-insensitive) are
    /// included; subdirectories and other file types are silently skipped.
    ///
    /// Returns `Err` when the directory itself cannot be read.
    fn list_ir_profile_file_names(&self, directory: &Path) -> Result<Vec<String>, String>;

    /// Creates `directory` and all missing parent components if they do not
    /// already exist (`mkdir -p` semantics).
    fn ensure_directory(&self, directory: &Path) -> Result<(), String>;

    /// Writes `bytes` to `path`, truncating any existing file.
    fn write_file_bytes(&self, path: &Path, bytes: &[u8]) -> Result<(), String>;

    /// Removes the file at `path`.
    ///
    /// Returns `Err` when the file does not exist or cannot be removed.
    fn remove_file(&self, path: &Path) -> Result<(), String>;

    /// Validates that raw bytes represent a usable cabinet IR WAV file.
    ///
    /// Checks performed in order:
    ///
    /// 1. **Extension** – `file_name` must end with `.wav` (case-insensitive).
    /// 2. **Parse** – bytes must form a valid WAV container readable by [`hound`].
    ///    Proprietary extensions that produce unexpected fmt-chunk sizes are
    ///    detected and surface a user-friendly re-export suggestion.
    /// 3. **Impulse presence** – at least one of the first `256` samples must
    ///    exceed `impulse_threshold` in absolute value.  A completely silent
    ///    header region usually indicates the wrong file was selected (e.g. a
    ///    music recording rather than an IR capture).
    ///
    /// Returns `Ok(())` when all checks pass, or `Err(description)` on the first
    /// failing check.
    fn validate_ir_wav_bytes(
        &self,
        file_name: &str,
        file_bytes: &[u8],
        impulse_threshold: f32,
    ) -> Result<(), String>;
}
