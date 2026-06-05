import {useEffect, useState} from "react";
import {listen} from "@tauri-apps/api/event";

export function useMidiLearning() {
    const [isLearning, setIsLearning] = useState<boolean>(false);
    const [midiChannel, setMidiChannel] = useState<number>(1);
    const [ccNumber, setCcNumber] = useState<number>(11);
    const [selectedEffectId, setSelectedEffectId] = useState<string>("");
    const [learnedMessage, setLearnedMessage] = useState<string | null>(null);

    useEffect(() => {
        const unlistenPromise = listen<[number, number]>("midi-raw-sniff", (event) => {
            if (isLearning) {
                const [payloadChannel, payloadCc] = event.payload;
                setMidiChannel(payloadChannel);
                setCcNumber(payloadCc);
                setIsLearning(false);
                setLearnedMessage(`Recognized Input! Set Port Line to CH ${payloadChannel}, CC Event ID to #${payloadCc}.`);
            }
        });

        return () => {
            unlistenPromise.then((cleanup) => cleanup());
        };
    }, [isLearning]);

    return {
        isLearning,
        setIsLearning,
        midiChannel,
        ccNumber,
        selectedEffectId,
        setSelectedEffectId,
        learnedMessage,
        setLearnedMessage,
    };
}
