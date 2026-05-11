use crate::infrastructure::file_loader::file_loader_trait::FileLoaderTrait;
use hound::{SampleFormat, WavReader};
use std::fs;
use std::io::Cursor;
use std::path::Path;
use tracing::{info, warn};

/// Filesystem-backed implementation of [`FileLoaderTrait`].
///
/// Uses [`std::fs`] for directory and file operations, and the [`hound`] crate
/// for WAV decoding and validation.
pub struct FileLoader;

impl FileLoader {
    /// Creates a new `FileLoader`.
    pub fn new() -> Self {
        Self
    }

    /// Downmixes a multi-channel interleaved buffer to mono by averaging channels.
    ///
    /// For mono input (`channels == 1`), returns the buffer unchanged.
    /// For stereo and multi-channel input, groups samples by channel count and
    /// averages them into a single mono stream.
    ///
    /// # Example
    /// ```text
    /// channels = 2, buffer = [L0, R0, L1, R1, L2, R2]
    /// → [(L0 + R0)/2, (L1 + R1)/2, (L2 + R2)/2]
    /// ```
    fn downmix_to_mono(buffer: Vec<f32>, channels: u16) -> Vec<f32> {
        if channels == 1 {
            return buffer;
        }

        let channels = channels as usize;
        let frame_count = buffer.len() / channels;
        let mut mono = Vec::with_capacity(frame_count);

        for frame_index in 0..frame_count {
            let start = frame_index * channels;
            let end = start + channels;
            let frame_sum: f32 = buffer[start..end].iter().sum();
            mono.push(frame_sum / channels as f32);
        }

        mono
    }
}

impl Default for FileLoader {
    fn default() -> Self {
        Self::new()
    }
}

impl FileLoaderTrait for FileLoader {
    fn read_wav_sample_rate(&self, path: &Path) -> Option<u32> {
        WavReader::open(path)
            .ok()
            .map(|reader| reader.spec().sample_rate)
    }

    fn read_wav_to_buffer(&self, path: &Path) -> Vec<f32> {
        match WavReader::open(path) {
            Ok(mut reader) => {
                let spec = reader.spec();
                match spec.sample_format {
                    SampleFormat::Float => {
                        match reader.samples::<f32>().collect::<Result<Vec<_>, _>>() {
                            Ok(buffer) => {
                                let mono = Self::downmix_to_mono(buffer, spec.channels);
                                info!(
                                    "Loaded IR '{}' (channels={}, sample_rate={}, mono_samples={})",
                                    path.display(),
                                    spec.channels,
                                    spec.sample_rate,
                                    mono.len()
                                );
                                mono
                            }
                            Err(e) => {
                                warn!(
                                    "Failed to read float samples from '{}': {e}",
                                    path.display()
                                );
                                Vec::new()
                            }
                        }
                    }
                    SampleFormat::Int => {
                        let max = ((1_i64 << (spec.bits_per_sample.saturating_sub(1))) - 1) as f32;
                        match reader
                            .samples::<i32>()
                            .map(|sample| sample.map(|value| value as f32 / max.max(1.0)))
                            .collect::<Result<Vec<_>, _>>()
                        {
                            Ok(buffer) => {
                                let mono = Self::downmix_to_mono(buffer, spec.channels);
                                info!(
                                    "Loaded IR '{}' (channels={}, sample_rate={}, mono_samples={})",
                                    path.display(),
                                    spec.channels,
                                    spec.sample_rate,
                                    mono.len()
                                );
                                mono
                            }
                            Err(e) => {
                                warn!("Failed to read int samples from '{}': {e}", path.display());
                                Vec::new()
                            }
                        }
                    }
                }
            }
            Err(e) => {
                warn!("Failed to open IR file '{}': {e}", path.display());
                Vec::new()
            }
        }
    }

    fn list_ir_profile_file_names(&self, directory: &Path) -> Result<Vec<String>, String> {
        let entries = fs::read_dir(directory)
            .map_err(|e| format!("Failed to read directory '{}': {e}", directory.display()))?;

        let mut names: Vec<String> = entries
            .filter_map(|entry| entry.ok())
            .filter_map(|entry| {
                let path = entry.path();
                if !path.is_file() {
                    return None;
                }

                if path
                    .extension()
                    .and_then(|ext| ext.to_str())
                    .map(|ext| ext.eq_ignore_ascii_case("wav"))
                    != Some(true)
                {
                    return None;
                }

                path.file_name()
                    .and_then(|name| name.to_str())
                    .map(|name| name.to_string())
            })
            .collect();

        names.sort();
        Ok(names)
    }

    fn ensure_directory(&self, directory: &Path) -> Result<(), String> {
        fs::create_dir_all(directory)
            .map_err(|e| format!("Failed to create directory '{}': {e}", directory.display()))
    }

    fn write_file_bytes(&self, path: &Path, bytes: &[u8]) -> Result<(), String> {
        fs::write(path, bytes)
            .map_err(|e| format!("Failed to write file '{}': {e}", path.display()))
    }

    fn remove_file(&self, path: &Path) -> Result<(), String> {
        fs::remove_file(path)
            .map_err(|e| format!("Failed to remove file '{}': {e}", path.display()))
    }

    fn validate_ir_wav_bytes(
        &self,
        file_name: &str,
        file_bytes: &[u8],
        impulse_threshold: f32,
    ) -> Result<(), String> {
        if !file_name.to_ascii_lowercase().ends_with(".wav") {
            return Err("Only .wav IR files are supported".to_string());
        }

        let mut reader = WavReader::new(Cursor::new(file_bytes)).map_err(|e| {
            let raw = e.to_string();
            if raw.contains("unexpected fmt chunk size") {
                format!(
                    "Unsupported WAV format for '{}': {}. Re-export as PCM 16/24-bit or IEEE float 32-bit WAV.",
                    file_name, raw
                )
            } else {
                format!("Invalid WAV file '{}': {raw}", file_name)
            }
        })?;

        let spec = reader.spec();

        const IMPULSE_SEARCH_WINDOW_SAMPLES: usize = 256;

        let max_abs_in_window = match spec.sample_format {
            SampleFormat::Float => {
                let mut iter = reader.samples::<f32>();
                let first = iter
                    .next()
                    .ok_or_else(|| "IR file is empty".to_string())
                    .and_then(|s| s.map_err(|e| format!("Failed to read first sample: {e}")))?;

                let mut max_abs = first.abs();
                for sample in iter.take(IMPULSE_SEARCH_WINDOW_SAMPLES.saturating_sub(1)) {
                    let value = sample.map_err(|e| format!("Failed to read WAV samples: {e}"))?;
                    max_abs = max_abs.max(value.abs());
                }

                max_abs
            }
            SampleFormat::Int => {
                let max = ((1_i64 << (spec.bits_per_sample.saturating_sub(1))) - 1) as f32;
                let mut iter = reader.samples::<i32>();
                let first = iter
                    .next()
                    .ok_or_else(|| "IR file is empty".to_string())
                    .and_then(|s| s.map_err(|e| format!("Failed to read first sample: {e}")))?;

                let mut max_abs = (first as f32 / max.max(1.0)).abs();
                for sample in iter.take(IMPULSE_SEARCH_WINDOW_SAMPLES.saturating_sub(1)) {
                    let value = sample.map_err(|e| format!("Failed to read WAV samples: {e}"))?;
                    max_abs = max_abs.max((value as f32 / max.max(1.0)).abs());
                }

                max_abs
            }
        };

        if max_abs_in_window <= impulse_threshold {
            return Err(
                "Invalid IR: no impulse detected at file start (first 256 samples are effectively silent)"
                    .to_string(),
            );
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use hound::{SampleFormat, WavSpec, WavWriter};
    use std::path::{Path, PathBuf};
    use std::time::{SystemTime, UNIX_EPOCH};

    fn unique_test_dir() -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time should be monotonic")
            .as_nanos();
        std::env::temp_dir().join(format!("rustriff-file-loader-{nanos}"))
    }

    fn write_float_wav_file(path: &Path, samples: &[f32], sample_rate: u32) {
        let spec = WavSpec {
            channels: 1,
            sample_rate,
            bits_per_sample: 32,
            sample_format: SampleFormat::Float,
        };

        let mut writer = WavWriter::create(path, spec).expect("wav file should be creatable");
        for sample in samples {
            writer
                .write_sample(*sample)
                .expect("sample should be writable");
        }
        writer.finalize().expect("wav writer should finalize");
    }

    fn write_stereo_wav_file(path: &Path, samples: &[f32], sample_rate: u32) {
        let spec = WavSpec {
            channels: 2,
            sample_rate,
            bits_per_sample: 32,
            sample_format: SampleFormat::Float,
        };

        let mut writer =
            WavWriter::create(path, spec).expect("stereo wav file should be creatable");
        for sample in samples {
            writer
                .write_sample(*sample)
                .expect("sample should be writable");
        }
        writer.finalize().expect("wav writer should finalize");
    }

    fn float_wav_bytes(samples: &[f32]) -> Vec<u8> {
        let dir = unique_test_dir();
        fs::create_dir_all(&dir).expect("test directory should be creatable");
        let path = dir.join("buffer.wav");
        write_float_wav_file(&path, samples, 48_000);
        let bytes = fs::read(&path).expect("generated wav should be readable");
        let _ = fs::remove_dir_all(dir);
        bytes
    }

    #[cfg(test)]
    mod success_path {
        use super::*;

        #[test]
        fn list_ir_profile_file_names_returns_sorted_wav_files_only() {
            let loader = FileLoader::new();
            let dir = unique_test_dir();
            fs::create_dir_all(dir.join("nested")).expect("test directory should be creatable");

            write_float_wav_file(&dir.join("z-room.wav"), &[0.5, 0.0], 48_000);
            write_float_wav_file(&dir.join("A-clean.WAV"), &[0.5, 0.0], 48_000);
            // Subdirectory WAVs and non-WAV files should be silently ignored
            write_float_wav_file(&dir.join("nested").join("ignored.wav"), &[0.5, 0.0], 48_000);
            fs::write(dir.join("notes.txt"), b"not a wav").expect("text file should be writable");

            let names = loader
                .list_ir_profile_file_names(&dir)
                .expect("listing IR profiles should succeed");

            assert_eq!(
                names,
                vec!["A-clean.WAV".to_string(), "z-room.wav".to_string()]
            );

            let _ = fs::remove_dir_all(dir);
        }

        #[test]
        fn read_wav_helpers_return_sample_rate_and_buffer_for_valid_ir() {
            let loader = FileLoader::new();
            let dir = unique_test_dir();
            fs::create_dir_all(&dir).expect("test directory should be creatable");
            let path = dir.join("valid-ir.wav");
            let samples = [0.75_f32, -0.25_f32, 0.125_f32];
            write_float_wav_file(&path, &samples, 44_100);

            let sample_rate = loader.read_wav_sample_rate(&path);
            let buffer = loader.read_wav_to_buffer(&path);

            assert_eq!(sample_rate, Some(44_100));
            assert_eq!(buffer.len(), samples.len());
            assert!((buffer[0] - 0.75).abs() < 1e-6);
            assert!((buffer[1] + 0.25).abs() < 1e-6);
            assert!((buffer[2] - 0.125).abs() < 1e-6);

            let _ = fs::remove_dir_all(dir);
        }

        #[test]
        fn validate_ir_wav_bytes_accepts_valid_impulse_wav() {
            let loader = FileLoader::new();
            let bytes = float_wav_bytes(&[0.25, 0.0, 0.0, 0.0]);

            loader
                .validate_ir_wav_bytes("cab.wav", &bytes, 1e-6)
                .expect("impulse IR should validate");
        }

        #[test]
        fn read_wav_to_buffer_downmixes_stereo_to_mono() {
            let loader = FileLoader::new();
            let dir = unique_test_dir();
            fs::create_dir_all(&dir).expect("test directory should be creatable");
            let path = dir.join("stereo-ir.wav");

            // Interleaved stereo: [L0, R0, L1, R1, L2, R2]
            let stereo_samples = [0.8_f32, 0.4_f32, 0.6_f32, 0.2_f32, 1.0_f32, 0.0_f32];
            write_stereo_wav_file(&path, &stereo_samples, 48_000);

            let mono = loader.read_wav_to_buffer(&path);

            // Expected downmix: [(0.8+0.4)/2, (0.6+0.2)/2, (1.0+0.0)/2] = [0.6, 0.4, 0.5]
            assert_eq!(mono.len(), 3);
            assert!((mono[0] - 0.6).abs() < 1e-6);
            assert!((mono[1] - 0.4).abs() < 1e-6);
            assert!((mono[2] - 0.5).abs() < 1e-6);

            let _ = fs::remove_dir_all(dir);
        }
    }

    #[cfg(test)]
    mod failure_path {
        use super::*;

        #[test]
        fn read_wav_sample_rate_returns_none_for_missing_file() {
            let loader = FileLoader::new();
            let missing = std::env::temp_dir().join("does-not-exist-rustriff.wav");
            assert!(loader.read_wav_sample_rate(&missing).is_none());
        }

        #[test]
        fn read_wav_to_buffer_returns_empty_for_missing_file() {
            let loader = FileLoader::new();
            let missing = std::env::temp_dir().join("does-not-exist-rustriff.wav");
            assert!(loader.read_wav_to_buffer(&missing).is_empty());
        }

        #[test]
        fn validate_ir_wav_bytes_rejects_non_wav_extension() {
            let loader = FileLoader::new();
            let bytes = float_wav_bytes(&[0.25, 0.0, 0.0, 0.0]);

            let err = loader
                .validate_ir_wav_bytes("cab.mp3", &bytes, 1e-6)
                .expect_err("non-wav extension should be rejected");
            assert!(err.contains("Only .wav IR files are supported"));
        }

        #[test]
        fn validate_ir_wav_bytes_rejects_silent_file_start() {
            let loader = FileLoader::new();
            let bytes = float_wav_bytes(&[0.0; 32]);

            let err = loader
                .validate_ir_wav_bytes("silent.wav", &bytes, 1e-6)
                .expect_err("silent IR should be rejected");
            assert!(err.contains("no impulse detected"));
        }
    }
}
