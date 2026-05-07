use crate::domain::dto::effect::ir_profile_dto::IrProfileDto;
use crate::domain::validation::sanitize_wav_file_name;
use crate::infrastructure::file_loader::FileLoaderTrait;
use std::path::PathBuf;
use tracing::{info, warn};

const DEFAULT_IR_DIRECTORY_NAME: &str = "default_ir";
const RESOURCES_DIRECTORY_NAME: &str = "resources";
/// Minimum absolute value a sample in the first 256 samples of an IR must have
/// to be considered a valid impulse.  Files that fail this check are almost
/// certainly not IRs (e.g., silence, or music recordings).
const IMPULSE_THRESHOLD: f32 = 1e-6;

/// Application service for IR profile discovery, upload, and removal.
///
/// `FileService` is the single authoritative source of truth for which IR
/// profiles the application knows about.  It merges the read-only
/// **default** IR set (shipped inside the Tauri bundle) with the
/// user-managed **custom** IR set stored in a writable directory.
///
/// ## Thread safety
/// `FileService` holds a `Box<dyn FileLoaderTrait>` which is `Send + Sync`,
/// making `FileService` itself safe to place in Tauri's shared-state container.
pub struct FileService {
    file_loader: Box<dyn FileLoaderTrait>,
    /// Root directory from which `resource_root/default_ir` is resolved in
    /// release builds.  In debug builds, `CARGO_MANIFEST_DIR` is used instead
    /// so running `cargo tauri dev` works without installing the app.
    resource_root: PathBuf,
    /// Writable directory where user-uploaded custom IR files are stored.
    custom_ir_directory: PathBuf,
}

impl FileService {
    /// Creates a new `FileService`.
    /// - `file_loader` – production or test-double implementation.
    /// - `resource_root` – used in release builds to locate bundled resources.
    /// - `custom_ir_directory` – path to the user's custom IR storage folder.
    pub fn new(
        file_loader: Box<dyn FileLoaderTrait>,
        resource_root: PathBuf,
        custom_ir_directory: PathBuf,
    ) -> Self {
        Self {
            file_loader,
            resource_root,
            custom_ir_directory,
        }
    }

    /// Returns all available IR profiles, merging default and custom sets.
    ///
    /// For each profile the [`IrProfileDto::is_in_use`] flag is **not** populated
    /// here (it is always `false`).  Callers that need accurate `is_in_use` values
    /// must cross-reference against the running effect chains — this is done by the
    /// [`get_all_ir_profiles`] Tauri command.
    ///
    /// The returned list is sorted alphabetically by [`IrProfileDto::label`].
    /// ## Errors
    /// Propagates errors from [`FileLoaderTrait::list_ir_profile_file_names`] or
    /// [`FileLoaderTrait::ensure_directory`] if the filesystem is inaccessible.
    /// [`get_all_ir_profiles`]: crate::commands::effect_commands::cabinet_ir::get_all_ir_profiles
    pub fn get_all_ir_profiles(&self) -> Result<Vec<IrProfileDto>, String> {
        let default_directory = self.resolve_default_ir_directory()?;
        self.file_loader.ensure_directory(&self.custom_ir_directory)?;

        let mut profiles = self
            .file_loader
            .list_ir_profile_file_names(&default_directory)?
            .into_iter()
            .map(|file_name| IrProfileDto {
                label: to_readable_label(&file_name),
                file_name,
                is_custom: false,
                is_in_use: false,
            })
            .collect::<Vec<_>>();

        let custom_profiles = self
            .file_loader
            .list_ir_profile_file_names(&self.custom_ir_directory)?
            .into_iter()
            .map(|file_name| IrProfileDto {
                label: to_readable_label(&file_name),
                file_name,
                is_custom: true,
                is_in_use: false,
            });

        profiles.extend(custom_profiles);
        profiles.sort_by(|a, b| a.label.cmp(&b.label));
        Ok(profiles)
    }

    /// Validates and persists a user-uploaded custom IR file.
    /// The following checks are applied before writing:
    /// 1. `file_name` is sanitized (no path traversal, `.wav` extension required).
    /// 2. `file_bytes` are validated as a well-formed WAV containing an audible
    ///    impulse via [`FileLoaderTrait::validate_ir_wav_bytes`].
    /// 3. A file with the same name must not already exist in the default IR set —
    ///    uploading a custom profile that would shadow a default one is rejected.
    /// On success, the sanitized file name (which may differ from `file_name` only
    /// in surrounding whitespace) is returned so the caller can display it.
    /// ## Errors
    /// Returns `Err` if any validation step fails or the file cannot be written.
    pub fn save_custom_ir_profile(&self, file_name: &str, file_bytes: &[u8]) -> Result<String, String> {
        let sanitized_file_name = sanitize_wav_file_name(file_name)?;

        self.file_loader.validate_ir_wav_bytes(
            &sanitized_file_name,
            file_bytes,
            IMPULSE_THRESHOLD,
        )?;

        let default_directory = self.resolve_default_ir_directory()?;
        let default_path = default_directory.join(&sanitized_file_name);
        if default_path.exists() {
            return Err(format!(
                "An IR named '{}' already exists in defaults",
                sanitized_file_name
            ));
        }

        self.file_loader.ensure_directory(&self.custom_ir_directory)?;
        let custom_path = self.custom_ir_directory.join(&sanitized_file_name);
        self.file_loader.write_file_bytes(&custom_path, file_bytes)?;

        Ok(sanitized_file_name)
    }

    /// Removes a user-uploaded custom IR file from the custom IR directory.
    ///
    /// `file_name` is sanitized before the path is constructed so that
    /// path-traversal attempts are rejected.
    /// ## Errors
    /// - `file_name` fails sanitization (invalid characters or extension).
    /// - The file does not exist in the custom IR directory.
    /// - The filesystem deletion fails.
    pub fn remove_custom_ir_profile(&self, file_name: &str) -> Result<(), String> {
        let sanitized_file_name = sanitize_wav_file_name(file_name)?;
        let custom_path = self.custom_ir_directory.join(&sanitized_file_name);

        if !custom_path.exists() {
            return Err(format!(
                "Custom IR '{}' does not exist",
                sanitized_file_name
            ));
        }

        self.file_loader.remove_file(&custom_path)
    }

    /// Returns the resolved absolute path of the bundled default IR directory.
    ///
    /// The resolution strategy differs between debug and release builds; see
    /// [`resolve_default_ir_directory`] for details.
    ///
    /// [`resolve_default_ir_directory`]: Self::resolve_default_ir_directory
    pub fn default_ir_directory(&self) -> Result<PathBuf, String> {
        self.resolve_default_ir_directory()
    }

    /// Returns the writable custom IR directory path.
    ///
    /// The directory is not guaranteed to exist yet — call
    /// [`FileLoaderTrait::ensure_directory`] before writing to it.
    pub fn custom_ir_directory(&self) -> PathBuf {
        self.custom_ir_directory.clone()
    }

    /// Resolves the absolute path of the bundled default IR directory.
    /// Resolution strategy:
    /// - **Debug builds** (`cfg(debug_assertions)`): looks inside
    ///   `CARGO_MANIFEST_DIR/resources/default_ir` so that running
    ///   `cargo tauri dev` works without installing the app.
    /// - **Release builds**: checks `resource_root/default_ir` and
    ///   `resource_root/resources/default_ir` as fallback, matching
    ///   different Tauri resource embedding strategies.
    /// The first candidate path that is an existing directory is returned.
    /// All skipped paths are logged at `WARN` level to aid debugging.
    fn resolve_default_ir_directory(&self) -> Result<PathBuf, String> {
        let mut candidates = if cfg!(debug_assertions) {
            vec![
                PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                    .join(RESOURCES_DIRECTORY_NAME)
                    .join(DEFAULT_IR_DIRECTORY_NAME),
            ]
        } else {
            vec![
                self.resource_root.join(DEFAULT_IR_DIRECTORY_NAME),
                self.resource_root
                    .join(RESOURCES_DIRECTORY_NAME)
                    .join(DEFAULT_IR_DIRECTORY_NAME),
            ]
        };

        candidates.dedup();

        for candidate in &candidates {
            if candidate.is_dir() {
                info!("Using default IR directory: {}", candidate.display());
                return Ok(candidate.clone());
            }
            warn!("Skipping missing IR directory candidate: {}", candidate.display());
        }

        let searched = candidates
            .iter()
            .map(|p| p.display().to_string())
            .collect::<Vec<_>>()
            .join(", ");

        Err(format!("Could not locate default IR directory. Searched: {searched}"))
    }
}


/// Converts a `.wav` filename into a human-readable label for display in the UI.
/// Transformations applied:
/// - file extension stripped (using `file_stem()`, handles any capitalization),
/// - hyphens (`-`) and underscores (`_`) replaced with spaces.
/// Example: `"vintage-4x12_cab.wav"` → `"vintage 4x12 cab"`.
fn to_readable_label(file_name: &str) -> String {
    std::path::Path::new(file_name)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or(file_name)
        .replace(['-', '_'], " ")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infrastructure::file_loader::FileLoader;
    use hound::{SampleFormat, WavSpec, WavWriter};
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::time::{SystemTime, UNIX_EPOCH};

    fn unique_test_dir() -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time should be monotonic")
            .as_nanos();
        std::env::temp_dir().join(format!("rustriff-file-service-{nanos}"))
    }

    fn build_service(custom_ir_directory: PathBuf) -> FileService {
        FileService::new(Box::new(FileLoader::new()), unique_test_dir(), custom_ir_directory)
    }

    fn write_float_wav_file(path: &Path, samples: &[f32]) {
        let spec = WavSpec {
            channels: 1,
            sample_rate: 48_000,
            bits_per_sample: 32,
            sample_format: SampleFormat::Float,
        };

        let mut writer = WavWriter::create(path, spec).expect("wav file should be creatable");
        for sample in samples {
            writer.write_sample(*sample).expect("sample should be writable");
        }
        writer.finalize().expect("wav writer should finalize");
    }

    fn float_wav_bytes(samples: &[f32]) -> Vec<u8> {
        let dir = unique_test_dir();
        fs::create_dir_all(&dir).expect("test directory should be creatable");
        let path = dir.join("buffer.wav");
        write_float_wav_file(&path, samples);
        let bytes = fs::read(&path).expect("generated wav should be readable");
        let _ = fs::remove_dir_all(dir);
        bytes
    }

    #[cfg(test)]
    mod success_path {
        use super::*;

        #[test]
        fn sanitize_wav_file_name_trims_whitespace_and_accepts_valid_names() {
            assert_eq!(sanitize_wav_file_name("  my-ir.wav  ").unwrap(), "my-ir.wav");
            assert_eq!(sanitize_wav_file_name("room.WAV").unwrap(), "room.WAV");
        }

        #[test]
        fn to_readable_label_strips_extension_and_replaces_separators() {
            assert_eq!(to_readable_label("vintage-4x12_cab.WAV"), "vintage 4x12 cab");
            assert_eq!(to_readable_label("info-support-hallway.wav"), "info support hallway");
        }

        #[test]
        fn get_all_ir_profiles_merges_defaults_and_customs_and_sorts_by_label() {
            let custom_dir = unique_test_dir();
            fs::create_dir_all(&custom_dir).expect("custom directory should be creatable");
            write_float_wav_file(&custom_dir.join("zzz-room.wav"), &[0.5, 0.0]);
            write_float_wav_file(&custom_dir.join("aaa-bright.wav"), &[0.5, 0.0]);

            let service = build_service(custom_dir.clone());
            let default_names = service
                .default_ir_directory()
                .and_then(|dir| FileLoader::new().list_ir_profile_file_names(&dir))
                .expect("default IRs should be discoverable");

            let profiles = service
                .get_all_ir_profiles()
                .expect("IR profile listing should succeed");

            assert_eq!(profiles.len(), default_names.len() + 2);
            assert!(profiles.windows(2).all(|pair| pair[0].label <= pair[1].label));
            assert!(profiles.iter().all(|profile| !profile.is_in_use));

            let bright = profiles
                .iter()
                .find(|profile| profile.file_name == "aaa-bright.wav")
                .expect("custom bright IR should be present");
            assert_eq!(bright.label, "aaa bright");
            assert!(bright.is_custom);

            let room = profiles
                .iter()
                .find(|profile| profile.file_name == "zzz-room.wav")
                .expect("custom room IR should be present");
            assert_eq!(room.label, "zzz room");
            assert!(room.is_custom);

            let _ = fs::remove_dir_all(custom_dir);
        }

        #[test]
        fn save_custom_ir_profile_writes_sanitized_file_name() {
            let custom_dir = unique_test_dir();
            let service = build_service(custom_dir.clone());
            let bytes = float_wav_bytes(&[0.25, 0.0, 0.0]);

            let saved_name = service
                .save_custom_ir_profile(" custom-room.wav ", &bytes)
                .expect("valid IR should be saved");

            assert_eq!(saved_name, "custom-room.wav");
            assert!(custom_dir.join("custom-room.wav").is_file());

            let _ = fs::remove_dir_all(custom_dir);
        }

        #[test]
        fn remove_custom_ir_profile_deletes_existing_file() {
            let custom_dir = unique_test_dir();
            fs::create_dir_all(&custom_dir).expect("custom directory should be creatable");
            let custom_path = custom_dir.join("to-remove.wav");
            write_float_wav_file(&custom_path, &[0.4, 0.0]);

            let service = build_service(custom_dir.clone());
            service
                .remove_custom_ir_profile("to-remove.wav")
                .expect("existing custom IR should be removed");
            assert!(!custom_path.exists());

            let _ = fs::remove_dir_all(custom_dir);
        }
    }

    #[cfg(test)]
    mod failure_path {
        use super::*;

        #[test]
        fn sanitize_wav_file_name_rejects_path_traversal() {
            assert!(sanitize_wav_file_name("../escape.wav").is_err());
            assert!(sanitize_wav_file_name("sub/dir.wav").is_err());
            assert!(sanitize_wav_file_name("sub\\dir.wav").is_err());
        }

        #[test]
        fn sanitize_wav_file_name_rejects_wrong_extension_and_empty_input() {
            assert!(sanitize_wav_file_name("clip.mp3").is_err());
            assert!(sanitize_wav_file_name("   ").is_err());
            assert!(sanitize_wav_file_name("").is_err());
        }

        #[test]
        fn save_custom_ir_profile_rejects_name_that_shadows_a_default_ir() {
            let custom_dir = unique_test_dir();
            let service = build_service(custom_dir.clone());
            let default_name = service
                .default_ir_directory()
                .and_then(|dir| FileLoader::new().list_ir_profile_file_names(&dir))
                .expect("default IRs should be discoverable")
                .into_iter()
                .next()
                .expect("at least one default IR should exist");

            let err = service
                .save_custom_ir_profile(&default_name, &float_wav_bytes(&[0.5, 0.0]))
                .expect_err("default IR names should be reserved");
            assert!(err.contains("already exists in defaults"));

            let _ = fs::remove_dir_all(custom_dir);
        }

        #[test]
        fn save_custom_ir_profile_rejects_silent_ir() {
            let custom_dir = unique_test_dir();
            let service = build_service(custom_dir.clone());

            let err = service
                .save_custom_ir_profile("silent-ir.wav", &float_wav_bytes(&[0.0; 32]))
                .expect_err("silent IR should be rejected");
            assert!(err.contains("no impulse detected"));

            let _ = fs::remove_dir_all(custom_dir);
        }

        #[test]
        fn remove_custom_ir_profile_rejects_missing_file() {
            let custom_dir = unique_test_dir();
            let service = build_service(custom_dir.clone());

            let err = service
                .remove_custom_ir_profile("not-there.wav")
                .expect_err("missing custom IR should fail removal");
            assert!(err.contains("does not exist"));
        }
    }
}

