import {useCallback, useEffect, useState} from "react";
import {getAmpConfig, getMidiBindings, MidiTargetParameter, removeMidiBinding} from "../domain";

export interface ActiveBinding {
    channel: number;
    cc_number: number;
    effect_id: string;
    parameter: MidiTargetParameter;
}

export function useMidiBindings() {
    const [bindings, setBindings] = useState<ActiveBinding[]>([]);
    const [activeEffects, setActiveEffects] = useState<{ id: string; name: string; kind: string }[]>([]);
    const [loading, setLoading] = useState<boolean>(false);
    const [error, setError] = useState<string | null>(null);
    const [successMessage, setSuccessMessage] = useState<string | null>(null);

    const fetchMidiMatrixData = useCallback(async () => {
        setLoading(true);
        setError(null);
        try {
            const ampConfig = await getAmpConfig();
            const incomingEffects = (ampConfig as { effects?: { kind: string; data?: { id?: string; name?: string } }[] })?.effects || [];

            const parsedEffects = incomingEffects.map((eff) => ({
                id: eff.data?.id || "",
                name: eff.data?.name || `${eff.kind || 'DSP'} Module`,
                kind: eff.kind || "Unknown"
            }));
            setActiveEffects(parsedEffects);

            const activeBindings = await getMidiBindings();
            setBindings(activeBindings);
        } catch (err) {
            console.error("Failed to sync system matrices:", err);
            setError(typeof err === "string" ? err : "Failed to sync system operational state.");
        } finally {
            setLoading(false);
        }
    }, []);

    useEffect(() => {
        fetchMidiMatrixData();
    }, [fetchMidiMatrixData]);

    const handleRemoveBinding = async (channel: number, ccNumber: number) => {
        setError(null);
        setSuccessMessage(null);
        try {
            await removeMidiBinding({channel, ccNumber});
            setSuccessMessage(`Mapping for Channel ${channel}, CC #${ccNumber} removed.`);
            await fetchMidiMatrixData();
        } catch (err) {
            console.error("Failed to delete map entry:", err);
            setError(typeof err === "string" ? err : "Failed to remove mapping.");
        }
    };

    return {
        bindings,
        activeEffects,
        loading,
        error,
        successMessage,
        setError,
        setSuccessMessage,
        handleRemoveBinding,
        refresh: fetchMidiMatrixData,
    };
}

export function formatParameterName(param: string) {
    return param.replace(/([A-Z])/g, ' $1').trim();
}
