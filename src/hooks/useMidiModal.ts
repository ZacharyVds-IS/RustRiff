import {useState} from "react";

export function useMidiModal() {
    const [midiModalOpen, setMidiModalOpen] = useState(false);

    return {
        midiModalOpen,
        openMidiModal: () => setMidiModalOpen(true),
        closeMidiModal: () => setMidiModalOpen(false),
    };
}
