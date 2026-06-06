import {useEffect, useRef, useState} from "react";
import {useLiveTuner} from "./useLiveTuner.ts";

export function useTunerPitch() {
    const {pitch, loadError} = useLiveTuner();
    const [lastValidPitch, setLastValidPitch] = useState<typeof pitch>(null);
    const [stableNote, setStableNote] = useState("—");
    const noteCounter = useRef({name: "", count: 0});

    const isSignalActive = pitch !== null && pitch.frequency_hz > 20 && pitch.note_name !== "---";

    useEffect(() => {
        if (isSignalActive) {
            setLastValidPitch(pitch);
            if (pitch.note_name === noteCounter.current.name) {
                noteCounter.current.count += 1;
                if (noteCounter.current.count >= 3) {
                    setStableNote(pitch.note_name);
                }
            } else {
                noteCounter.current.name = pitch.note_name;
                noteCounter.current.count = 1;
            }
        }
    }, [pitch, isSignalActive]);

    return {
        pitch,
        loadError,
        lastValidPitch,
        stableNote,
        isSignalActive,
    };
}
