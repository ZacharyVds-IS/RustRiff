use midir::MidiInputConnection;
use serde::Serialize;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tauri::Emitter;
use tracing::info;
use uuid::Uuid;

use crate::domain::channel_manager::ChannelManager;
use crate::domain::dto::midi_mapping_dto::MidiMappingDto;
use crate::domain::dto::MidiDeviceDto::MidiDeviceDto;
use crate::domain::midi_target_parameter::MidiTargetParameter;
use crate::infrastructure::midi_handler::MidiHandler;
use crate::infrastructure::midi_parser::ParsedMidiCc;

#[derive(Clone, Serialize)]
pub struct MidiValueChangedPayload {
    pub effect_id: String,
    pub parameter: String,
    pub value: f64,
    pub cc_number: u8,
}

pub struct MidiService {
    active_connection: Mutex<Option<MidiInputConnection<()>>>,
    bindings: Arc<Mutex<HashMap<(u8, u8), (Uuid, MidiTargetParameter)>>>,
    channel_manager: Arc<Mutex<ChannelManager>>,
    app_handle: Mutex<Option<tauri::AppHandle>>,
}

impl MidiService {
    pub fn new(channel_manager: Arc<Mutex<ChannelManager>>) -> Self {
        Self {
            active_connection: Mutex::new(None),
            bindings: Arc::new(Mutex::new(HashMap::new())),
            channel_manager,
            app_handle: Mutex::new(None),
        }
    }

    pub fn set_app_handle(&self, handle: tauri::AppHandle) {
        *self.app_handle.lock().unwrap() = Some(handle);
    }

    pub fn get_input_devices(&self) -> Vec<MidiDeviceDto> {
        MidiHandler::list_devices()
    }

    /// Overwrites the internal mappings completely using a collection of DTOs.
    pub fn set_bindings(&self, midi_bindings: Vec<MidiMappingDto>) {
        if let Ok(mut bindings) = self.bindings.lock() {
            bindings.clear();
            for mapping in midi_bindings {
                let wire_channel = mapping.channel.saturating_sub(1);
                if let Ok(id) = Uuid::parse_str(&mapping.effect_id) {
                    bindings.insert((wire_channel, mapping.cc_number), (id, mapping.parameter));
                }
            }
            info!("Restored {} MIDI binding(s) into memory", bindings.len());
        }
    }

    pub fn add_mapping(&self, config: MidiMappingDto, parsed_id: Uuid) -> Vec<MidiMappingDto> {
        if let Ok(mut bindings) = self.bindings.lock() {
            let wire_channel = config.channel.saturating_sub(1);

            bindings.insert(
                (wire_channel, config.cc_number),
                (parsed_id, config.parameter.clone()),
            );

            info!(
                "Mapped CC {} on Raw Channel {} (UI Channel {}) to Effect {}",
                config.cc_number, wire_channel, config.channel, parsed_id
            );

            Self::bindings_to_dtos(&bindings)
        } else {
            Vec::new()
        }
    }

    pub fn remove_mapping(&self, channel: u8, cc_number: u8) -> Vec<MidiMappingDto> {
        if let Ok(mut bindings) = self.bindings.lock() {
            let wire_channel = channel.saturating_sub(1);
            if bindings.remove(&(wire_channel, cc_number)).is_some() {
                info!(
                    "Removed MIDI mapping on Wire Channel {}, CC {}",
                    wire_channel, cc_number
                );
            }
            Self::bindings_to_dtos(&bindings)
        } else {
            Vec::new()
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
        let cm_clone = Arc::clone(&self.channel_manager);
        let handle_clone = self.app_handle.lock().unwrap().clone();

        let conn = midi_in
            .connect(
                port,
                "tauri-midi-read-loop",
                move |_timestamp, message, _| {
                    MidiService::process_incoming_message(
                        message,
                        &bindings_clone,
                        &cm_clone,
                        &handle_clone,
                    );
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

    pub fn get_active_mappings(&self) -> Vec<MidiMappingDto> {
        if let Ok(bindings) = self.bindings.lock() {
            Self::bindings_to_dtos(&bindings)
        } else {
            Vec::new()
        }
    }

    // ── Private helpers ───────────────────────────────────────────────────────

    /// Converts the in-memory bindings map to a `Vec` of DTOs suitable for
    /// persistence or command responses.
    fn bindings_to_dtos(
        bindings: &HashMap<(u8, u8), (Uuid, MidiTargetParameter)>,
    ) -> Vec<MidiMappingDto> {
        bindings
            .iter()
            .map(
                |((wire_channel, cc_number), (effect_id, parameter))| MidiMappingDto {
                    channel: wire_channel.saturating_add(1),
                    cc_number: *cc_number,
                    effect_id: effect_id.to_string(),
                    parameter: parameter.clone(),
                },
            )
            .collect()
    }

    fn process_incoming_message(
        bytes: &[u8],
        bindings_ref: &Arc<Mutex<HashMap<(u8, u8), (Uuid, MidiTargetParameter)>>>,
        channel_manager_ref: &Arc<Mutex<ChannelManager>>,
        app_handle: &Option<tauri::AppHandle>,
    ) {
        let raw_hex: String = bytes
            .iter()
            .map(|b| format!("{:02X}", b))
            .collect::<Vec<_>>()
            .join(" ");
        info!("RAW MIDI: [{}] {} bytes", raw_hex, bytes.len());

        if bytes.is_empty() {
            return;
        }

        let status = bytes[0];
        let msg_type = status & 0xF0;
        let channel = status & 0x0F;

        let type_name = match msg_type {
            0x80 => "NoteOff",
            0x90 => "NoteOn",
            0xA0 => "PolyKeyPressure",
            0xB0 => "ControlChange",
            0xC0 => "ProgramChange",
            0xD0 => "ChannelPressure",
            0xE0 => "PitchBend",
            0xF0 => "System",
            _ => "Unknown",
        };

        if msg_type == 0xB0 {
            if let Some(cc) = ParsedMidiCc::from_bytes(bytes) {
                info!(
                    "MIDI CC Struct: ch={} cc_number={} value={}",
                    cc.channel, cc.control_number, cc.value
                );

                if let Some(handle) = app_handle {
                    let ui_channel = cc.channel + 1;
                    let _ = handle.emit("midi-raw-sniff", (ui_channel, cc.control_number));
                }

                if let Ok(bindings) = bindings_ref.lock() {
                    if let Some((effect_id, param)) = bindings.get(&(cc.channel, cc.control_number))
                    {
                        let normalized = cc.value as f32 / 127.0;
                        if let Ok(cm) = channel_manager_ref.lock() {
                            let (value, param_name): (f64, &str) = if param.is_toggle() {
                                match cm.toggle_effect_active(*effect_id) {
                                    Ok(new_active) => {
                                        info!(
                                            "MIDI -> Effect {} ToggleBypass: active={}",
                                            effect_id, new_active
                                        );
                                        (if new_active { 1.0 } else { 0.0 }, "active")
                                    }
                                    Err(e) => {
                                        info!("MIDI toggle failed for Effect {}: {}", effect_id, e);
                                        (0.0, "active")
                                    }
                                }
                            } else {
                                match param {
                                    MidiTargetParameter::WahPedalPosition => {
                                        let _ = cm.set_effect_parameter(
                                            *effect_id,
                                            "pedal_position",
                                            normalized,
                                        );
                                        info!(
                                            "MIDI -> Effect {} WahPedalPosition: {:.2}",
                                            effect_id, normalized
                                        );
                                        (normalized as f64, "pedal_position")
                                    }
                                    MidiTargetParameter::DelayTime => {
                                        let delay_ms = (normalized * 2000.0) as u32;
                                        let _ = cm.set_effect_parameter(
                                            *effect_id,
                                            "delay_time",
                                            delay_ms,
                                        );
                                        info!(
                                            "MIDI -> Effect {} DelayTime: {} ms",
                                            effect_id, delay_ms
                                        );
                                        (delay_ms as f64, "delay_time")
                                    }
                                    MidiTargetParameter::DelayLevel => {
                                        let _ = cm
                                            .set_effect_parameter(*effect_id, "level", normalized);
                                        info!(
                                            "MIDI -> Effect {} DelayLevel: {:.2}",
                                            effect_id, normalized
                                        );
                                        (normalized as f64, "level")
                                    }
                                    MidiTargetParameter::DistortionLevel => {
                                        let gain = 1.0 + normalized;
                                        let _ = cm.set_effect_parameter(*effect_id, "level", gain);
                                        info!(
                                            "MIDI -> Effect {} DistortionLevel: {:.2}",
                                            effect_id, normalized
                                        );
                                        (gain as f64, "level")
                                    }
                                    MidiTargetParameter::DistortionThreshold => {
                                        let safe = normalized.max(0.001);
                                        let _ =
                                            cm.set_effect_parameter(*effect_id, "threshold", safe);
                                        info!(
                                            "MIDI -> Effect {} DistortionThreshold: {:.3}",
                                            effect_id, safe
                                        );
                                        (safe as f64, "threshold")
                                    }
                                    _ => unreachable!(),
                                }
                            };

                            if let Some(handle) = app_handle {
                                let payload = MidiValueChangedPayload {
                                    effect_id: effect_id.to_string(),
                                    parameter: param_name.to_string(),
                                    value,
                                    cc_number: cc.control_number,
                                };
                                let _ = handle.emit("onvaluechange", payload);
                            }
                        }
                    } else {
                        info!(
                            "MIDI CC (unmapped): ch={} cc={} value={}",
                            cc.channel, cc.control_number, cc.value
                        );
                    }
                }
            }
        } else if bytes.len() >= 2 {
            let data: String = bytes[1..]
                .iter()
                .map(|b| format!("{}", b))
                .collect::<Vec<_>>()
                .join(" ");
            info!("MIDI {}: ch={} data=[{}]", type_name, channel, data);
        }
    }
}
