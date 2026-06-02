import React from "react";
import ReactDOM from "react-dom/client";
import App from "./App";
import {useAmpStore} from "./state/AmpConfigStore.tsx";
import {ChannelDto, getAmpConfig} from "./domain";
import {listen} from "@tauri-apps/api/event";
import {getCurrentWebviewWindow} from "@tauri-apps/api/webviewWindow";
import {ANALYZER_WINDOW_LABEL} from "./windows/AnalyzerWindow";

const isAnalyzerWindow = getCurrentWebviewWindow().label === ANALYZER_WINDOW_LABEL;

interface MidiValueChangedPayload {
    effect_id: string;
    parameter: string;
    value: number;
    cc_number: number;
}

async function configureListeners() {
    await useAmpStore.getState().init();

    await listen<ChannelDto>("channel-added", (event) => {
        console.log("[event] channel-added", event.payload);
        useAmpStore.getState().addChannelFromBackend(event.payload);
    });

    await listen<MidiValueChangedPayload>("onvaluechange", (event) => {
        console.log("[event] onvaluechange", event.payload);
        const {effect_id, parameter, value} = event.payload;
        if (parameter === "active") {
            useAmpStore.getState().updateEffectActiveState(effect_id, value === 1.0);
        } else {
            getAmpConfig().then((config) => useAmpStore.setState(config)).catch(console.error);
        }
    });
}

if (!isAnalyzerWindow) {
    configureListeners().catch((error) => {
        console.error("Failed to configure backend listeners", error);
    });
}

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <App />
  </React.StrictMode>,
);
