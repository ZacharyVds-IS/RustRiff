pub struct ParsedMidiCc {
    pub channel: u8,
    pub control_number: u8,
    pub value: u8,
}

impl ParsedMidiCc {
    /// Safely parses raw MIDI bytes specifically targeting Control Change (CC) events
    pub fn from_bytes(bytes: &[u8]) -> Option<Self> {
        if bytes.len() < 3 {
            return None;
        }

        let status_byte = bytes[0];
        let message_type = status_byte & 0xF0; // Extract high nibble
        let channel = status_byte & 0x0F; // Extract low nibble (0-15)

        // 0xB0 is the status indicator for a Control Change event
        if message_type == 0xB0 {
            Some(Self {
                channel,
                control_number: bytes[1],
                value: bytes[2],
            })
        } else {
            None // Ignore note-on, note-off, pitch bends for now
        }
    }
}
