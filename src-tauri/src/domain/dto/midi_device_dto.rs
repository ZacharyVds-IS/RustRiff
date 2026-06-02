#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct MidiDeviceDto {
    pub id: String,
    pub name: String,
}
