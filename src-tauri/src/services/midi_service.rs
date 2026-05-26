use midir::MidiInputConnection;
// src/services/midi_service.rs
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tracing::info;
use uuid::Uuid;

use crate::domain::dto::midi_mapping_dto::MidiMappingDto;
use crate::domain::dto::MidiDeviceDto::MidiDeviceDto;
use crate::domain::midi_target_parameter::MidiTargetParameter;
use crate::infrastructure::midi_handler::MidiHandler;
use crate::infrastructure::midi_parser::ParsedMidiCc;

pub struct MidiService {
    active_connection: Mutex<Option<MidiInputConnection<()>>>,
    bindings: Arc<Mutex<HashMap<(u8, u8), (Uuid, MidiTargetParameter)>>>,
}

impl MidiService {
    pub fn new() -> Self {
        Self {
            active_connection: Mutex::new(None),
            bindings: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn get_input_devices(&self) -> Vec<MidiDeviceDto> {
        MidiHandler::list_devices()
    }

    pub fn add_mapping(&self, config: MidiMappingDto, parsed_id: Uuid) {
        if let Ok(mut bindings) = self.bindings.lock() {
            let wire_channel = if config.channel > 0 {
                config.channel - 1
            } else {
                0
            };

            bindings.insert(
                (wire_channel, config.cc_number),
                (parsed_id, config.parameter.clone()),
            );

            info!(
                "Mapped CC {} on Raw Channel {} (UI Channel {}) to Effect {}",
                config.cc_number, wire_channel, config.channel, parsed_id
            );
        }
    }

    pub fn connect_to_device(&self, device_id: &str) -> Result<(), String> {
        self.disconnect();

        let midi_in = MidiHandler::create_input()?;
        let ports = midi_in.ports();
        let port_index: usize = device_id.parse().map_err(|_| "Invalid ID format")?;
        let port = ports
            .get(port_index)
            .ok_or_else(|| "Device unavailable".to_string())?;

        let bindings_clone = Arc::clone(&self.bindings);

        let conn = midi_in
            .connect(
                port,
                "tauri-midi-read-loop",
                move |_timestamp, message, _| {
                    MidiService::process_incoming_message(message, &bindings_clone);
                },
                (),
            )
            .map_err(|e| format!("MIDI link failed: {}", e))?;

        *self.active_connection.lock().unwrap() = Some(conn);
        Ok(())
    }

    pub fn disconnect(&self) {
        if let Ok(mut active) = self.active_connection.lock() {
            *active = None;
        }
    }

    fn process_incoming_message(
        bytes: &[u8],
        bindings_ref: &Arc<Mutex<HashMap<(u8, u8), (Uuid, MidiTargetParameter)>>>,
    ) {
        if let Some(cc) = ParsedMidiCc::from_bytes(bytes) {
            if let Ok(bindings) = bindings_ref.lock() {
                if let Some((effect_id, param)) = bindings.get(&(cc.channel, cc.control_number)) {
                    MidiService::log_simulated_dsp_action(*effect_id, param, cc.value);
                }
            }
        }
    }

    fn log_simulated_dsp_action(effect_id: Uuid, parameter: &MidiTargetParameter, raw_value: u8) {
        let normalized = raw_value as f32 / 127.0;

        match parameter {
            MidiTargetParameter::ToggleBypass => {
                info!(
                    "[SIMULATED DSP] Effect {} -> ToggleBypass: active={}",
                    effect_id,
                    raw_value >= 64
                );
            }
            MidiTargetParameter::WahPedalPosition => {
                info!(
                    "[SIMULATED DSP] Effect {} -> WahPedalPosition: pedal_position={:.2}",
                    effect_id, normalized
                );
            }
            MidiTargetParameter::DelayTime => {
                info!(
                    "[SIMULATED DSP] Effect {} -> DelayTime: delay_time={:.1} ms",
                    effect_id,
                    normalized * 2000.0
                );
            }
            MidiTargetParameter::DelayLevel => {
                info!(
                    "[SIMULATED DSP] Effect {} -> DelayLevel: delay_level={:.2}",
                    effect_id, normalized
                );
            }
            MidiTargetParameter::DistortionLevel => {
                info!(
                    "[SIMULATED DSP] Effect {} -> DistortionLevel: distortion_level={:.2}",
                    effect_id, normalized
                );
            }
            MidiTargetParameter::DistortionThreshold => {
                info!(
                    "[SIMULATED DSP] Effect {} -> DistortionThreshold: threshold={:.2}",
                    effect_id, normalized
                );
            }
        }
    }
}
