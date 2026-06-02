import {WebviewWindow} from "@tauri-apps/api/webviewWindow";

export {TabWindow} from "./TabWindow.tsx";

export const TAB_WINDOW_LABEL = "tab-view";
const TAB_ROUTE = "/#/tab";
export async function openTabWindow(): Promise<void> {
    const existingWindow = await WebviewWindow.getByLabel(TAB_WINDOW_LABEL);
    if (existingWindow) {
        await existingWindow.setFocus();
        return;
    }

    const TabWindow = new WebviewWindow(TAB_WINDOW_LABEL, {
        title: "RustRiff Tab",
        url: TAB_ROUTE,
        width: 1024,
        height: 700,
        resizable: true,
        minimizable: true,
        maximizable: true,
        center: true,
    });

    await TabWindow.once("tauri://error", (error) => {
        console.error("Failed to create Tab window", error);
    });
}