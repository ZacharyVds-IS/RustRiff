use crate::domain::dto::midi_device_dto::MidiDeviceDto;
use crate::domain::dto::midi_mapping_dto::MidiMappingDto;
use crate::services::amp_config_service::AmpConfigPersistenceService;
use crate::services::midi_service::MidiService;
use std::sync::Mutex;
use tauri::State;

#[tauri::command]
pub async fn get_midi_inputs(
    midi_service: State<'_, MidiService>,
) -> Result<Vec<MidiDeviceDto>, String> {
    Ok(midi_service.get_input_devices())
}

#[tauri::command]
pub async fn connect_midi_device(
    midi_service: State<'_, MidiService>,
    id: String,
) -> Result<(), String> {
    midi_service.connect_to_device(id.as_str())
}

#[tauri::command]
pub async fn disconnect_midi_device(midi_service: State<'_, MidiService>) -> Result<(), String> {
    midi_service.disconnect();
    Ok(())
}

#[tauri::command]
pub async fn register_midi_binding(
    midi_service: State<'_, MidiService>,
    persistence_service: State<'_, Mutex<AmpConfigPersistenceService>>,
    mapping: MidiMappingDto,
) -> Result<(), String> {
    let parsed_uuid = uuid::Uuid::parse_str(&mapping.effect_id)
        .map_err(|_| format!("Invalid UUID format provided: {}", mapping.effect_id))?;

    let updated_dtos = midi_service.add_mapping(mapping, parsed_uuid);

    let persistence = persistence_service.lock().unwrap();
    if let Err(e) = persistence.persist_midi_bindings(updated_dtos) {
        tracing::warn!("Failed to persist MIDI bindings after register_midi_binding: {e}");
    }

    Ok(())
}

#[tauri::command]
pub async fn get_midi_bindings(
    midi_service: State<'_, MidiService>,
) -> Result<Vec<MidiMappingDto>, String> {
    Ok(midi_service.get_active_mappings())
}

#[tauri::command]
pub async fn remove_midi_binding(
    midi_service: State<'_, MidiService>,
    persistence_service: State<'_, Mutex<AmpConfigPersistenceService>>,
    channel: u8,
    cc_number: u8,
) -> Result<(), String> {
    let updated_dtos = midi_service.remove_mapping(channel, cc_number);

    let persistence = persistence_service.lock().unwrap();
    if let Err(e) = persistence.persist_midi_bindings(updated_dtos) {
        tracing::warn!("Failed to persist MIDI bindings after remove_midi_binding: {e}");
    }

    Ok(())
}
