import {listen, type UnlistenFn} from "@tauri-apps/api/event";
import {useEffect, useState} from "react";
import {getTunerContract, startLiveTunerStream, stopLiveTunerStream, TunerContractDto,} from "../domain";

export interface PitchSnapshotDto {
    frequency_hz: number;
    note_name: string;
    cents_deviation: number;
    clarity: number;
}

export type LiveTunerState = {
    pitch: PitchSnapshotDto | null;
    contract: TunerContractDto | null;
    loadError: string | null;
};


export function useLiveTuner(): LiveTunerState {
    const [pitch, setPitch] = useState<PitchSnapshotDto | null>(null);
    const [contract, setContract] = useState<TunerContractDto | null>(null);
    const [loadError, setLoadError] = useState<string | null>(null);

    useEffect(() => {
        let disposed = false;
        let unlisten: UnlistenFn | null = null;

        const bind = async () => {
            try {
                // 1. Fetch static streaming configuration from the Rust backend
                const nextContract = await getTunerContract();
                if (disposed) {
                    return;
                }
                setContract(nextContract);

                // 2. Listen to the push-based live tuner event channel.
                // Payload can be PitchSnapshotDto or null (when background filters reject signals)
                unlisten = await listen<PitchSnapshotDto | null>(nextContract.live_tuner_event, (event) => {
                    if (disposed) {
                        return;
                    }
                    setPitch(event.payload);
                    setLoadError(null);
                });

                // 3. Signal the background tokio task loop to initialize pitch tracking
                await startLiveTunerStream();
            } catch (error) {
                if (!disposed) {
                    setLoadError(error instanceof Error ? error.message : "Failed to read tuner stream");
                }
            }
        };

        void bind();

        // Standard cleanup pattern guarantees cancellation vectors fire instantly on unmount
        return () => {
            disposed = true;
            if (unlisten) {
                unlisten();
            }
            void stopLiveTunerStream();
        };
    }, []);

    return { pitch, contract, loadError };
}