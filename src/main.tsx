import React from "react";
import ReactDOM from "react-dom/client";
import App from "./App";
import {useAmpStore} from "./state/AmpConfigStore.tsx";
import {listen} from "@tauri-apps/api/event";
import {ChannelDto} from "./domain";

async function configureListeners() {
    await useAmpStore.getState().init();

    await listen<ChannelDto>("channel-added", (event) => {
        console.log("[event] channel-added", event.payload);
        useAmpStore.getState().addChannelFromBackend(event.payload);
    });
}

configureListeners()

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <App />
  </React.StrictMode>,
);
