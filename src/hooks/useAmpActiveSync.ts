import {listen, type UnlistenFn} from "@tauri-apps/api/event";
import {useEffect} from "react";
import {AMP_ACTIVE_CHANGED_EVENT, useAmpStore} from "../state/AmpConfigStore.tsx";

export function useAmpActiveSync() {
    useEffect(() => {
        let unlisten: UnlistenFn | null = null;
        let disposed = false;

        const sync = async () => {
            try {
                await useAmpStore.getState().init();
                if (disposed) {
                    return;
                }

                unlisten = await listen<boolean>(AMP_ACTIVE_CHANGED_EVENT, (event) => {
                    if (disposed) {
                        return;
                    }
                    useAmpStore.setState({is_active: event.payload});
                });
            } catch (error) {
                console.error("[useAmpActiveSync] Failed to initialize amp state sync:", error);
            }
        };

        void sync();

        return () => {
            disposed = true;
            if (unlisten) {
                unlisten();
            }
        };
    }, []);
}

