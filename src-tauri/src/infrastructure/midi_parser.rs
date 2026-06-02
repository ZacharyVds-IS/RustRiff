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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_bytes_parses_valid_cc_message() {
        let bytes = &[0xB0, 0x15, 0x7F]; // CC on channel 0, controller 21, value 127
        let result = ParsedMidiCc::from_bytes(bytes);
        assert!(result.is_some());
        let cc = result.unwrap();
        assert_eq!(cc.channel, 0);
        assert_eq!(cc.control_number, 0x15);
        assert_eq!(cc.value, 0x7F);
    }

    #[test]
    fn from_bytes_parses_cc_on_channel_15() {
        let bytes = &[0xBF, 0x01, 0x00]; // CC on channel 15, controller 1, value 0
        let result = ParsedMidiCc::from_bytes(bytes);
        assert!(result.is_some());
        let cc = result.unwrap();
        assert_eq!(cc.channel, 15);
        assert_eq!(cc.control_number, 1);
        assert_eq!(cc.value, 0);
    }

    #[test]
    fn from_bytes_returns_none_for_note_on() {
        let bytes = &[0x90, 0x40, 0x7F]; // NoteOn, not CC
        assert!(ParsedMidiCc::from_bytes(bytes).is_none());
    }

    #[test]
    fn from_bytes_returns_none_for_note_off() {
        let bytes = &[0x80, 0x40, 0x00]; // NoteOff
        assert!(ParsedMidiCc::from_bytes(bytes).is_none());
    }

    #[test]
    fn from_bytes_returns_none_for_pitch_bend() {
        let bytes = &[0xE0, 0x00, 0x40]; // PitchBend
        assert!(ParsedMidiCc::from_bytes(bytes).is_none());
    }

    #[test]
    fn from_bytes_returns_none_for_too_short_buffer() {
        assert!(ParsedMidiCc::from_bytes(&[0xB0]).is_none());
        assert!(ParsedMidiCc::from_bytes(&[]).is_none());
        assert!(ParsedMidiCc::from_bytes(&[0xB0, 0x01]).is_none());
    }

    #[test]
    fn from_bytes_returns_none_for_system_message() {
        let bytes = &[0xF0, 0x00, 0x20]; // SysEx start
        assert!(ParsedMidiCc::from_bytes(bytes).is_none());
    }

    #[test]
    fn from_bytes_handles_full_range_cc_values() {
        for value in [0x00, 0x40, 0x7F] {
            let bytes = &[0xB0, 0x01, value];
            let cc = ParsedMidiCc::from_bytes(bytes).unwrap();
            assert_eq!(cc.value, value);
        }
    }

    #[test]
    fn from_bytes_handles_full_range_controller_numbers() {
        for cc_num in [0x00, 0x40, 0x7F] {
            let bytes = &[0xB0, cc_num, 0x40];
            let cc = ParsedMidiCc::from_bytes(bytes).unwrap();
            assert_eq!(cc.control_number, cc_num);
        }
    }
}
