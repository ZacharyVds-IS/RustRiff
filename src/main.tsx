import React from "react";
import ReactDOM from "react-dom/client";
import App from "./App";
import {useAmpStore} from "./state/AmpConfigStore.tsx";
import {listen} from "@tauri-apps/api/event";
import {getCurrentWebviewWindow} from "@tauri-apps/api/webviewWindow";
import {ChannelDto} from "./domain";
import {ANALYZER_WINDOW_LABEL} from "./windows/AnalyzerWindow";

const isRunningWithTauri = "__TAURI__" in window;
let isAnalyzerWindow = false;
if (isRunningWithTauri) {
    const win = getCurrentWebviewWindow();
    isAnalyzerWindow = win.label === ANALYZER_WINDOW_LABEL;
}

async function configureListeners() {
    await useAmpStore.getState().init();

    await listen<ChannelDto>("channel-added", (event) => {
        console.log("[event] channel-added", event.payload);
        useAmpStore.getState().addChannelFromBackend(event.payload);
    });
}

if (isRunningWithTauri && !isAnalyzerWindow) {
    configureListeners().catch((error) => {
        console.error("Failed to configure backend listeners", error);
    });
}

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
    <React.StrictMode>
        <App />
    </React.StrictMode>
);
