import {useEffect, useRef, useState} from "react";
import * as commands from "../domain/commands";
import * as types from "../domain/types";

export function useAudioLatency() {
    const BUFFER_SIZE_OPTIONS = [64, 128, 256, 512, 1024, 2048, 4096];
    const isInitialMount = useRef(true);

    const [bufferLatency, setBufferLatency] = useState<types.BufferLatencyDto | null>(null);
    const [bufferSizeFrames, setBufferSizeFrames] = useState<number>(256);
    const [bufferSizeSaving, setBufferSizeSaving] = useState(false);
    const [bufferSizeError, setBufferSizeError] = useState<string | null>(null);

    const [roundTripLatency, setRoundTripLatency] = useState<number | null>(null);
    const [roundTripLoading, setRoundTripLoading] = useState(false);
    const [roundTripError, setRoundTripError] = useState<string | null>(null);

    async function loadBufferLatency() {
        try {
            const latency = await commands.measureBufferLatency();
            setBufferLatency(latency);
        } catch {
            setBufferLatency(null);
        }
    }

    async function loadBufferSizeFrames() {
        try {
            const frames = await commands.getBufferSizeFrames();
            setBufferSizeFrames(frames);
            setBufferSizeError(null);
        } catch (err) {
            setBufferSizeError(err instanceof Error ? err.message : "Failed to load buffer size");
        } finally {
            isInitialMount.current = false;
        }
    }

    async function performRoundTripMeasurement() {
        setRoundTripLoading(true);
        setRoundTripError(null);
        try {
            const result = await commands.measureRoundTripLatency();
            if (result.is_valid) {
                setRoundTripLatency(result.latency_ms);
            } else {
                setRoundTripError(result.error || "Failed to measure round-trip latency");
                setRoundTripLatency(null);
            }
        } catch (err) {
            setRoundTripError(err instanceof Error ? err.message : "Unknown error occurred");
            setRoundTripLatency(null);
        } finally {
            setRoundTripLoading(false);
        }
    }

    useEffect(() => {
        void loadBufferLatency();
        void loadBufferSizeFrames();
    }, []);

    useEffect(() => {
        if (isInitialMount.current) return;

        async function autoApplyBuffer() {
            setBufferSizeSaving(true);
            setBufferSizeError(null);
            try {
                await commands.setBufferSizeFrames({frames: bufferSizeFrames});
                await loadBufferLatency();
            } catch (err) {
                setBufferSizeError(err instanceof Error ? err.message : "Failed to auto-apply buffer size");
            } finally {
                setBufferSizeSaving(false);
            }
        }

        void autoApplyBuffer();
    }, [bufferSizeFrames]);

    function handleBufferSizeChange(value: string | number) {
        setBufferSizeFrames(Number(value));
    }

    const bufferSizeOptions = BUFFER_SIZE_OPTIONS.map((frames) => ({
        label: `${frames} frames`,
        value: frames,
    }));

    return {
        bufferLatency,
        bufferSizeFrames,
        bufferSizeSaving,
        bufferSizeError,
        bufferSizeOptions,
        roundTripLatency,
        roundTripLoading,
        roundTripError,
        handleBufferSizeChange,
        handleMeasureRoundTripLatency: performRoundTripMeasurement,
        loadBufferLatency,
    };
}
