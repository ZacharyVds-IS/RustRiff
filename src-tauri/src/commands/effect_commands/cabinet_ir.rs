use crate::domain::channel_manager::ChannelManager;
use crate::domain::dto::effect::ir_profile_dto::IrProfileDto;
use crate::services::audio_service::AudioService;
use crate::services::file_service::FileService;
use std::collections::HashSet;
use std::sync::Mutex;
use tracing::{info, warn};

#[tauri::command]
pub fn get_all_ir_profiles(
    file_service: tauri::State<FileService>,
    audio_service: tauri::State<Mutex<AudioService>>,
) -> Result<Vec<IrProfileDto>, String> {
    let used_profiles = used_ir_profiles(&audio_service).map_err(|err| {
        warn!("get_all_ir_profiles failed while reading used profiles: {err}");
        err
    })?;
    let mut profiles = file_service.get_all_ir_profiles().map_err(|err| {
        warn!("get_all_ir_profiles failed while reading profile inventory: {err}");
        err
    })?;

    mark_profiles_in_use(&mut profiles, &used_profiles);

    Ok(profiles)
}

#[tauri::command]
pub fn upload_ir_profile(
    file_service: tauri::State<FileService>,
    file_name: String,
    file_bytes: Vec<u8>,
) -> Result<String, String> {
    info!(
        "Uploading custom IR profile '{}' ({} bytes)",
        file_name,
        file_bytes.len()
    );
    file_service
        .save_custom_ir_profile(&file_name, &file_bytes)
        .map_err(|err| {
            warn!("upload_ir_profile failed for '{}': {err}", file_name);
            err
        })
}

#[tauri::command]
pub fn remove_ir_profile(
    file_service: tauri::State<FileService>,
    audio_service: tauri::State<Mutex<AudioService>>,
    file_name: String,
) -> Result<(), String> {
    let profiles = file_service.get_all_ir_profiles().map_err(|err| {
        warn!("remove_ir_profile failed while reading profile inventory: {err}");
        err
    })?;

    let used_profiles = used_ir_profiles(&audio_service).map_err(|err| {
        warn!("remove_ir_profile failed while checking chain usage: {err}");
        err
    })?;
    ensure_profile_can_be_removed(&profiles, &file_name, &used_profiles)?;

    file_service
        .remove_custom_ir_profile(&file_name)
        .map_err(|err| {
            warn!("remove_ir_profile failed for '{}': {err}", file_name);
            err
        })
}

fn mark_profiles_in_use(profiles: &mut [IrProfileDto], used_profiles: &HashSet<String>) {
    for profile in profiles {
        profile.is_in_use = used_profiles.contains(&profile.file_name);
    }
}

fn ensure_profile_can_be_removed(
    profiles: &[IrProfileDto],
    file_name: &str,
    used_profiles: &HashSet<String>,
) -> Result<(), String> {
    let profile = profiles
        .iter()
        .find(|p| p.file_name == file_name)
        .ok_or_else(|| format!("IR profile '{}' not found", file_name))?;

    if !profile.is_custom {
        return Err("Default IR profiles cannot be removed".to_string());
    }

    if used_profiles.contains(file_name) {
        return Err(format!(
            "IR profile '{}' is currently used by an effect chain",
            file_name
        ));
    }

    Ok(())
}

fn used_ir_profiles(
    audio_service: &tauri::State<Mutex<AudioService>>,
) -> Result<HashSet<String>, String> {
    let audio_service = audio_service
        .lock()
        .map_err(|_| "Failed to lock audio service".to_string())?;
    let cm = audio_service
        .channel_manager()
        .lock()
        .map_err(|_| "Failed to lock channel manager".to_string())?;
    collect_used_ir_profiles(&cm)
}

fn collect_used_ir_profiles(channel_manager: &ChannelManager) -> Result<HashSet<String>, String> {
    let mut used = HashSet::new();
    for channel in channel_manager.channels().iter() {
        used.extend(channel.used_cabinet_ir_profiles());
    }
    Ok(used)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::channel_manager::ChannelManager;
    use crate::services::audio_service::AudioService;
    use crate::services::effects::cabinet::cabinet::Cabinet;
    use crate::tests::mock::make_mock_handler;
    use std::sync::{Arc, Mutex};

    fn make_service() -> AudioService {
        AudioService::new_with_handler(
            Arc::new(make_mock_handler()),
            Arc::new(Mutex::new(ChannelManager::new())),
        )
    }

    fn profile(file_name: &str, is_custom: bool) -> IrProfileDto {
        IrProfileDto {
            file_name: file_name.to_string(),
            label: file_name.to_string(),
            is_custom,
            is_in_use: false,
        }
    }

    #[cfg(test)]
    mod success_path {
        use super::*;
        use uuid::Uuid;

        #[test]
        fn collect_used_ir_profiles_deduplicates_cabinet_profiles_across_channels() {
            let service = make_service();

            {
                let mut cm = service.channel_manager().lock().unwrap();
                cm.channels_mut()[0].add_effect_to_chain(Box::new(Cabinet::new(
                    Uuid::new_v4(),
                    "Cab A".to_string(),
                    true,
                    "#111111".to_string(),
                    "Reverb-oxford-lean.wav".to_string(),
                    48_000,
                )));

                let second_channel_id = cm.add_channel("Lead".to_string());
                let second_channel = cm
                    .channels_mut()
                    .iter_mut()
                    .find(|channel| channel.id() == second_channel_id)
                    .expect("second channel should exist");
                second_channel.add_effect_to_chain(Box::new(Cabinet::new(
                    Uuid::new_v4(),
                    "Cab B".to_string(),
                    true,
                    "#222222".to_string(),
                    "Reverb-oxford-lean.wav".to_string(),
                    48_000,
                )));
                second_channel.add_effect_to_chain(Box::new(Cabinet::new(
                    Uuid::new_v4(),
                    "Cab C".to_string(),
                    true,
                    "#333333".to_string(),
                    "Vox-ac30.wav".to_string(),
                    48_000,
                )));
            }

            let cm = service.channel_manager().lock().unwrap();
            let used =
                collect_used_ir_profiles(&cm).expect("used IR profile discovery should succeed");

            assert_eq!(used.len(), 2);
            assert!(used.contains("Reverb-oxford-lean.wav"));
            assert!(used.contains("Vox-ac30.wav"));
        }

        #[test]
        fn mark_profiles_in_use_sets_flags_from_used_set() {
            let mut profiles = vec![
                profile("Vox-ac30.wav", false),
                profile("custom-room.wav", true),
            ];
            let used_profiles = HashSet::from(["custom-room.wav".to_string()]);

            mark_profiles_in_use(&mut profiles, &used_profiles);

            assert!(!profiles[0].is_in_use);
            assert!(profiles[1].is_in_use);
        }

        #[test]
        fn ensure_profile_can_be_removed_accepts_unused_custom_profile() {
            let profiles = vec![profile("custom-room.wav", true)];

            ensure_profile_can_be_removed(&profiles, "custom-room.wav", &HashSet::new())
                .expect("unused custom profile should be removable");
        }

        #[test]
        fn collect_used_ir_profiles_reflects_restored_chain_metadata() {
            let service = make_service();

            {
                let mut cm = service.channel_manager().lock().unwrap();
                cm.channels_mut()[0].add_effect_to_chain(Box::new(Cabinet::new(
                    Uuid::new_v4(),
                    "Cab A".to_string(),
                    true,
                    "#111111".to_string(),
                    "Vox-ac30.wav".to_string(),
                    48_000,
                )));

                cm.channels_mut()[0].restore_effect_chain(vec![Box::new(Cabinet::new(
                    Uuid::new_v4(),
                    "Cab B".to_string(),
                    true,
                    "#222222".to_string(),
                    "Reverb-oxford-lean.wav".to_string(),
                    48_000,
                ))]);
            }

            let cm = service.channel_manager().lock().unwrap();
            let used = collect_used_ir_profiles(&cm)
                .expect("restored chain usage discovery should succeed");

            assert_eq!(used.len(), 1);
            assert!(used.contains("Reverb-oxford-lean.wav"));
            assert!(!used.contains("Vox-ac30.wav"));
        }
    }

    #[cfg(test)]
    mod failure_path {
        use super::*;

        #[test]
        fn ensure_profile_can_be_removed_rejects_missing_profile() {
            let profiles = vec![profile("Vox-ac30.wav", false)];

            let err = ensure_profile_can_be_removed(&profiles, "missing.wav", &HashSet::new())
                .expect_err("missing profile should fail");
            assert!(err.contains("not found"));
        }

        #[test]
        fn ensure_profile_can_be_removed_rejects_default_profile() {
            let profiles = vec![profile("Vox-ac30.wav", false)];

            assert_eq!(
                ensure_profile_can_be_removed(&profiles, "Vox-ac30.wav", &HashSet::new())
                    .expect_err("default profile should be protected"),
                "Default IR profiles cannot be removed"
            );
        }

        #[test]
        fn ensure_profile_can_be_removed_rejects_in_use_custom_profile() {
            let profiles = vec![profile("custom-room.wav", true)];
            let used_profiles = HashSet::from(["custom-room.wav".to_string()]);

            let err = ensure_profile_can_be_removed(&profiles, "custom-room.wav", &used_profiles)
                .expect_err("in-use profile should be protected");
            assert!(err.contains("currently used by an effect chain"));
        }
    }
}
