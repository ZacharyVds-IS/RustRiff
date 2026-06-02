use crate::domain::dto::midi_device_dto::MidiDeviceDto;
use midir::MidiInput;

pub struct MidiHandler;

impl MidiHandler {
    pub fn create_input() -> Result<MidiInput, String> {
        MidiInput::new("Tauri-DSP-MIDI-Client")
            .map_err(|e| format!("Failed to create MIDI input client: {}", e))
    }

    pub fn list_devices() -> Vec<MidiDeviceDto> {
        let midi_in = match Self::create_input() {
            Ok(client) => client,
            Err(_) => return vec![],
        };

        midi_in
            .ports()
            .iter()
            .enumerate()
            .map(|(index, port)| {
                let name = midi_in
                    .port_name(port)
                    .unwrap_or_else(|_| format!("MIDI Port {}", index));
                MidiDeviceDto {
                    id: index.to_string(),
                    name,
                }
            })
            .collect()
    }
}
