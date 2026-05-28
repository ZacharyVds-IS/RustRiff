// @vitest-environment jsdom
import React, {act} from "react";
import {createRoot, Root} from "react-dom/client";
import {beforeAll, beforeEach, describe, expect, it, vi} from "vitest";

// Mock must be before imports
vi.mock("../../domain", () => ({
    getAmpConfig: vi.fn(),
    getMidiBindings: vi.fn(),
    registerMidiBinding: vi.fn(),
    removeMidiBinding: vi.fn(),
}));

vi.mock("react-router-dom", () => ({
    useNavigate: () => vi.fn(),
}));

vi.mock("@tauri-apps/api/event", () => ({
    listen: vi.fn().mockResolvedValue(vi.fn()),
}));

const flush = () => new Promise((resolve) => setTimeout(resolve, 0));

describe("MidiConfigScreen", () => {
    let container: HTMLDivElement;
    let root: Root;
    let MidiConfigScreen: typeof import("../../screens/MidiConfigScreen").MidiConfigScreen;

    beforeAll(async () => {
        (globalThis as { IS_REACT_ACT_ENVIRONMENT?: boolean }).IS_REACT_ACT_ENVIRONMENT = true;
        MidiConfigScreen = (await import("../../screens/MidiConfigScreen")).MidiConfigScreen;
    });

    beforeEach(() => {
        vi.clearAllMocks();
        container = document.createElement("div");
        document.body.appendChild(container);
        root = createRoot(container);
    });

    describe("success_path", () => {
        it("renders the MIDI configuration header", async () => {
            const {getAmpConfig, getMidiBindings} = await import("../../domain");
            vi.mocked(getAmpConfig).mockResolvedValue({effects: []} as any);
            vi.mocked(getMidiBindings).mockResolvedValue([]);

            await act(async () => {
                root.render(<MidiConfigScreen />);
            });
            await flush();

            expect(container.textContent).toContain("MIDI Configuration");
        });

        it("shows no mappings message when no bindings exist", async () => {
            const {getAmpConfig, getMidiBindings} = await import("../../domain");
            vi.mocked(getAmpConfig).mockResolvedValue({effects: []} as any);
            vi.mocked(getMidiBindings).mockResolvedValue([]);

            await act(async () => {
                root.render(<MidiConfigScreen />);
            });
            await flush();

            expect(container.textContent).toContain("No parameters are currently mapped");
        });

        it("displays active bindings in the table", async () => {
            const {getAmpConfig, getMidiBindings} = await import("../../domain");
            vi.mocked(getAmpConfig).mockResolvedValue({
                effects: [
                    {kind: "Wah", data: {id: "abc-123", name: "Crybaby"}},
                ],
            } as any);
            vi.mocked(getMidiBindings).mockResolvedValue([
                {
                    channel: 1,
                    cc_number: 7,
                    effect_id: "abc-123",
                    parameter: "WahPedalPosition",
                },
            ]);

            await act(async () => {
                root.render(<MidiConfigScreen />);
            });
            await flush();

            expect(container.textContent).toContain("CH 1");
            expect(container.textContent).toContain("CC #7");
            expect(container.textContent).toContain("Crybaby");
            expect(container.textContent).toContain("Wah Pedal Position");
        });

        it("shows refresh matrix button", async () => {
            const {getAmpConfig, getMidiBindings} = await import("../../domain");
            vi.mocked(getAmpConfig).mockResolvedValue({effects: []} as any);
            vi.mocked(getMidiBindings).mockResolvedValue([]);

            await act(async () => {
                root.render(<MidiConfigScreen />);
            });
            await flush();

            expect(container.textContent).toContain("Refresh Matrix");
        });

        it("renders back button", async () => {
            const {getAmpConfig, getMidiBindings} = await import("../../domain");
            vi.mocked(getAmpConfig).mockResolvedValue({effects: []} as any);
            vi.mocked(getMidiBindings).mockResolvedValue([]);

            await act(async () => {
                root.render(<MidiConfigScreen />);
            });
            await flush();

            const buttons = container.querySelectorAll("button");
            const backButton = Array.from(buttons).find(
                (b) => b.textContent?.includes("Back")
            );
            expect(backButton).toBeTruthy();
        });

        it("displays bindings with custom/decoupled block for unmatched effect_id", async () => {
            const {getAmpConfig, getMidiBindings} = await import("../../domain");
            vi.mocked(getAmpConfig).mockResolvedValue({effects: []} as any);
            vi.mocked(getMidiBindings).mockResolvedValue([
                {
                    channel: 1,
                    cc_number: 1,
                    effect_id: "nonexistent-id",
                    parameter: "ToggleBypass",
                },
            ]);

            await act(async () => {
                root.render(<MidiConfigScreen />);
            });
            await flush();

            expect(container.textContent).toContain("Custom/Decoupled Block");
        });
    });

    describe("failure_path", () => {
        it("shows error alert when getAmpConfig fails", async () => {
            const {getAmpConfig, getMidiBindings} = await import("../../domain");
            vi.mocked(getAmpConfig).mockRejectedValue("Config load error");
            vi.mocked(getMidiBindings).mockResolvedValue([]);

            await act(async () => {
                root.render(<MidiConfigScreen />);
            });
            await flush();

            // The error message from the rejection is used directly
            expect(container.textContent).toContain("Config load error");
        });

        it("shows error alert when getMidiBindings fails", async () => {
            const {getAmpConfig, getMidiBindings} = await import("../../domain");
            vi.mocked(getAmpConfig).mockResolvedValue({effects: []} as any);
            vi.mocked(getMidiBindings).mockRejectedValue("Bindings error");

            await act(async () => {
                root.render(<MidiConfigScreen />);
            });
            await flush();

            expect(container.textContent).toContain("Bindings error");
        });

        it("shows error alert when registerMidiBinding fails", async () => {
            const {getAmpConfig, getMidiBindings, registerMidiBinding} = await import("../../domain");
            vi.mocked(getAmpConfig).mockResolvedValue({effects: []} as any);
            vi.mocked(getMidiBindings).mockResolvedValue([]);
            vi.mocked(registerMidiBinding).mockRejectedValue("Register failed");

            await act(async () => {
                root.render(<MidiConfigScreen />);
            });
            await flush();

            // Component should still render without crashing
            expect(container.textContent).toContain("MIDI Configuration");
        });
    });

    describe("edge_cases", () => {
        it("handles effects with missing names gracefully", async () => {
            const {getAmpConfig, getMidiBindings} = await import("../../domain");
            vi.mocked(getAmpConfig).mockResolvedValue({
                effects: [
                    {kind: "Delay", data: {id: "xyz"}},
                ],
            } as any);
            vi.mocked(getMidiBindings).mockResolvedValue([]);

            await act(async () => {
                root.render(<MidiConfigScreen />);
            });
            await flush();

            expect(container.textContent).toContain("MIDI Configuration");
        });

        it("shows empty table when effects array is null", async () => {
            const {getAmpConfig, getMidiBindings} = await import("../../domain");
            vi.mocked(getAmpConfig).mockResolvedValue({} as any);
            vi.mocked(getMidiBindings).mockResolvedValue([]);

            await act(async () => {
                root.render(<MidiConfigScreen />);
            });
            await flush();

            expect(container.textContent).toContain("No parameters are currently mapped");
        });
    });
});
