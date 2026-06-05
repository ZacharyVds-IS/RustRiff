// @vitest-environment jsdom
import React, {act} from "react";
import {createRoot, Root} from "react-dom/client";
import {beforeAll, beforeEach, describe, expect, it, vi} from "vitest";
import {MidiSection} from "../../components/MidiSection";

vi.mock("../../domain", () => ({
    getMidiInputs: vi.fn(),
    connectMidiDevice: vi.fn(),
    disconnectMidiDevice: vi.fn(),
}));

vi.mock("react-router-dom", () => ({
    useNavigate: () => vi.fn(),
}));

const flush = () => new Promise((resolve) => setTimeout(resolve, 0));

describe("MidiSection", () => {
    let container: HTMLDivElement;
    let root: Root;

    beforeAll(() => {
        (globalThis as { IS_REACT_ACT_ENVIRONMENT?: boolean }).IS_REACT_ACT_ENVIRONMENT = true;
    });

    beforeEach(() => {
        vi.clearAllMocks();
        container = document.createElement("div");
        document.body.appendChild(container);
        root = createRoot(container);
    });

    describe("success_path", () => {
        it("renders title and scan button", async () => {
            const {getMidiInputs} = await import("../../domain");
            vi.mocked(getMidiInputs).mockResolvedValue([]);

            await act(async () => {
                root.render(<MidiSection />);
            });
            await flush();

            expect(container.textContent).toContain("MIDI Configuration");
            expect(container.textContent).toContain("Scan");
        });

        it("shows no devices message when list is empty", async () => {
            const {getMidiInputs} = await import("../../domain");
            vi.mocked(getMidiInputs).mockResolvedValue([]);

            await act(async () => {
                root.render(<MidiSection />);
            });
            await flush();

            expect(container.textContent).toContain("No MIDI devices detected");
        });

        it("renders device list when devices are available", async () => {
            const {getMidiInputs} = await import("../../domain");
            vi.mocked(getMidiInputs).mockResolvedValue([
                {id: "0", name: "USB MIDI Interface"},
                {id: "1", name: "Expression Pedal"},
            ]);

            await act(async () => {
                root.render(<MidiSection />);
            });
            await flush();

            expect(container.textContent).toContain("USB MIDI Interface");
            expect(container.textContent).toContain("Expression Pedal");
        });

        it("auto-connects to discovered devices", async () => {
            const {getMidiInputs, connectMidiDevice} = await import("../../domain");
            vi.mocked(getMidiInputs).mockResolvedValue([
                {id: "0", name: "MIDI Device"},
            ]);

            await act(async () => {
                root.render(<MidiSection />);
            });
            await flush();

            expect(connectMidiDevice).toHaveBeenCalledWith({id: "0"});
            expect(container.textContent).toContain("Connected");
        });

        it("shows configure advanced mappings button", async () => {
            const {getMidiInputs} = await import("../../domain");
            vi.mocked(getMidiInputs).mockResolvedValue([]);

            await act(async () => {
                root.render(<MidiSection />);
            });
            await flush();

            expect(container.textContent).toContain("Configure Advanced Mappings");
        });
    });

    describe("failure_path", () => {
        it("handles getMidiInputs error gracefully", async () => {
            const {getMidiInputs} = await import("../../domain");
            vi.mocked(getMidiInputs).mockRejectedValue(new Error("MIDI bus error"));

            await act(async () => {
                root.render(<MidiSection />);
            });
            await flush();

            // Should still render the component shell
            expect(container.textContent).toContain("MIDI Configuration");
        });
    });
});
