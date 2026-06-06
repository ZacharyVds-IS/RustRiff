import {useCallback, useEffect, useRef, useState} from "react";
import {connectMidiDevice, disconnectMidiDevice, getMidiInputs, MidiDeviceDto} from "../domain";

export function useMidiDevices() {
    const [devices, setDevices] = useState<MidiDeviceDto[]>([]);
    const [connectedDeviceId, setConnectedDeviceId] = useState<string | null>(null);
    const [loading, setLoading] = useState<boolean>(false);
    const lastConnectedId = useRef<string | null>(null);

    const fetchMidiDevices = useCallback(async () => {
        setLoading(true);
        try {
            const inputs = await getMidiInputs();
            setDevices(inputs);

            const hasDevices = inputs.length > 0;
            const connectedStillAvailable = connectedDeviceId && inputs.some(d => d.id === connectedDeviceId);

            if (hasDevices && !connectedStillAvailable) {
                const targetId = inputs.find(d => d.id === lastConnectedId.current)?.id ?? inputs[0].id;
                await connectMidiDevice({id: targetId});
                setConnectedDeviceId(targetId);
                lastConnectedId.current = targetId;
            }
        } catch (err) {
            console.error("Failed to fetch MIDI devices:", err);
        } finally {
            setLoading(false);
        }
    }, [connectedDeviceId]);

    useEffect(() => {
        fetchMidiDevices();
    }, [fetchMidiDevices]);

    const handleConnect = async (id: string) => {
        try {
            await connectMidiDevice({id});
            setConnectedDeviceId(id);
            lastConnectedId.current = id;
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
