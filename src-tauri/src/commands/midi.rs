use crate::domain::dto::midi_mapping_dto::MidiMappingDto;
use crate::domain::dto::MidiDeviceDto::MidiDeviceDto;
use crate::services::midi_service::MidiService;
// src/commands/midi_commands.rs
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
    mapping: MidiMappingDto,
) -> Result<(), String> {
    let parsed_uuid = uuid::Uuid::parse_str(&mapping.effect_id)
        .map_err(|_| format!("Invalid UUID format provided: {}", mapping.effect_id))?;

    midi_service.add_mapping(mapping, parsed_uuid);
    Ok(())
}
