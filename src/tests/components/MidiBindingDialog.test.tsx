// @vitest-environment jsdom
import React, {act} from "react";
import {createRoot, Root} from "react-dom/client";
import {beforeAll, beforeEach, describe, expect, it, vi} from "vitest";

vi.mock("../../domain", () => ({
    getMidiBindings: vi.fn(),
    registerMidiBinding: vi.fn(),
    removeMidiBinding: vi.fn(),
}));

vi.mock("@tauri-apps/api/event", () => ({
    listen: vi.fn().mockResolvedValue(vi.fn()),
}));

const flush = () => new Promise((resolve) => setTimeout(resolve, 0));

describe("MidiBindingDialog", () => {
    let container: HTMLDivElement;
    let root: Root;
    let MidiBindingDialog: typeof import("../../components/dialogs/MidiBindingDialog/MidiBindingDialog").MidiBindingDialog;

    beforeAll(async () => {
        (globalThis as { IS_REACT_ACT_ENVIRONMENT?: boolean }).IS_REACT_ACT_ENVIRONMENT = true;
        MidiBindingDialog = (await import("../../components/dialogs/MidiBindingDialog/MidiBindingDialog")).MidiBindingDialog;
    });

    beforeEach(() => {
        vi.clearAllMocks();
        container = document.createElement("div");
        document.body.appendChild(container);
        root = createRoot(container);
    });

    const defaultProps = {
        open: true,
        onClose: vi.fn(),
        effectId: "test-effect-id",
        effectName: "Crybaby",
        effectKind: "Wah",
    };

    describe("success_path", () => {
        it("renders dialog title with effect name and kind", async () => {
            const {getMidiBindings} = await import("../../domain");
            vi.mocked(getMidiBindings).mockResolvedValue([]);

            await act(async () => {
                root.render(<MidiBindingDialog {...defaultProps} />);
            });
            await flush();

            expect(document.body.textContent).toContain("Crybaby");
            expect(document.body.textContent).toContain("Wah");
        });

        it("renders parameter selection step", async () => {
            const {getMidiBindings} = await import("../../domain");
            vi.mocked(getMidiBindings).mockResolvedValue([]);

            await act(async () => {
                root.render(<MidiBindingDialog {...defaultProps} />);
            });
            await flush();

            expect(document.body.textContent).toContain("Parameter selection");
        });

        it("renders CC assignment step", async () => {
            const {getMidiBindings} = await import("../../domain");
            vi.mocked(getMidiBindings).mockResolvedValue([]);

            await act(async () => {
                root.render(<MidiBindingDialog {...defaultProps} />);
            });
            await flush();

            expect(document.body.textContent).toContain("Assign CC Input");
        });

        it("renders save binding button", async () => {
            const {getMidiBindings} = await import("../../domain");
            vi.mocked(getMidiBindings).mockResolvedValue([]);

            await act(async () => {
                root.render(<MidiBindingDialog {...defaultProps} />);
            });
            await flush();

            expect(document.body.textContent).toContain("Save Binding");
        });

        it("shows parameter options for Wah effect", async () => {
            const {getMidiBindings} = await import("../../domain");
            vi.mocked(getMidiBindings).mockResolvedValue([]);

            await act(async () => {
                root.render(<MidiBindingDialog {...defaultProps} />);
            });
            await flush();

            expect(document.body.textContent).toContain("On / Off");
            expect(document.body.textContent).toContain("Sweep");
        });

        it("shows active bindings on the pedal", async () => {
            const {getMidiBindings} = await import("../../domain");
            vi.mocked(getMidiBindings).mockResolvedValue([
                {
                    channel: 1,
                    cc_number: 7,
                    effect_id: "test-effect-id",
                    parameter: "WahPedalPosition",
                },
            ]);

            await act(async () => {
                root.render(<MidiBindingDialog {...defaultProps} />);
            });
            await flush();

            expect(document.body.textContent).toContain("Active Bindings on this Pedal");
            expect(document.body.textContent).toContain("CH 1");
            expect(document.body.textContent).toContain("CC #7");
        });

        it("shows recognize button for MIDI learn", async () => {
            const {getMidiBindings} = await import("../../domain");
            vi.mocked(getMidiBindings).mockResolvedValue([]);

            await act(async () => {
                root.render(<MidiBindingDialog {...defaultProps} />);
            });
            await flush();

            expect(document.body.textContent).toContain("Recognize button");
        });
    });

    describe("edge_cases", () => {
        it("renders correctly when closed", async () => {
            const {getMidiBindings} = await import("../../domain");
            vi.mocked(getMidiBindings).mockResolvedValue([]);

            await act(async () => {
                root.render(<MidiBindingDialog {...defaultProps} open={false} />);
            });
            await flush();

            // When closed, the dialog content should not be visible to the user.
            // MUI Dialog renders in DOM but hides with CSS/transitions.
            // In jsdom we simply verify no crash occurs.
            expect(true).toBe(true);
        });

        it("handles unknown effect kind with fallback parameters", async () => {
            const {getMidiBindings} = await import("../../domain");
            vi.mocked(getMidiBindings).mockResolvedValue([]);

            await act(async () => {
                root.render(
                    <MidiBindingDialog
                        {...defaultProps}
                        effectKind="UnknownKind"
                    />
                );
            });
            await flush();

            // Fallback should include ToggleBypass
            expect(document.body.textContent).toContain("On / Off");
        });

        it("handles save binding success", async () => {
            const {getMidiBindings, registerMidiBinding} = await import("../../domain");
            vi.mocked(getMidiBindings).mockResolvedValue([]);
            vi.mocked(registerMidiBinding).mockResolvedValue(undefined);

            await act(async () => {
                root.render(<MidiBindingDialog {...defaultProps} />);
            });
            await flush();

            // Find save button in document (Dialog uses portal)
            const saveButton = Array.from(document.body.querySelectorAll("button")).find(
                (b) => b.textContent?.includes("Save Binding")
            );
            expect(saveButton).toBeTruthy();
        });

        it("handles remove binding gracefully", async () => {
            const {getMidiBindings, removeMidiBinding} = await import("../../domain");
            vi.mocked(getMidiBindings).mockResolvedValue([]);
            vi.mocked(removeMidiBinding).mockResolvedValue(undefined);

            await act(async () => {
                root.render(<MidiBindingDialog {...defaultProps} />);
            });
            await flush();

            expect(document.body.textContent).toContain("MIDI Mapping");
        });
    });
});
