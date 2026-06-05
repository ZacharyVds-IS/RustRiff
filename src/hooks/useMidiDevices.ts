import {useEffect, useState} from "react";
import {connectMidiDevice, disconnectMidiDevice, getMidiInputs, MidiDeviceDto} from "../domain";

export function useMidiDevices() {
    const [devices, setDevices] = useState<MidiDeviceDto[]>([]);
    const [connectedDeviceId, setConnectedDeviceId] = useState<string | null>(null);
    const [loading, setLoading] = useState<boolean>(false);

    const fetchMidiDevices = async () => {
        setLoading(true);
        try {
            const inputs = await getMidiInputs();
            setDevices(inputs);
        } catch (err) {
            console.error("Failed to fetch MIDI devices:", err);
        } finally {
            setLoading(false);
        }
    };

    useEffect(() => {
        fetchMidiDevices();
    }, []);

    const handleConnect = async (id: string) => {
        try {
            await connectMidiDevice({id});
            setConnectedDeviceId(id);
        } catch (err) {
            console.error("Failed to connect device:", err);
        }
    };

    const handleDisconnect = async () => {
        try {
            await disconnectMidiDevice();
            setConnectedDeviceId(null);
        } catch (err) {
            console.error("Failed to disconnect device:", err);
        }
    };

    return {
        devices,
        connectedDeviceId,
        loading,
        handleConnect,
        handleDisconnect,
        refresh: fetchMidiDevices,
    };
}
