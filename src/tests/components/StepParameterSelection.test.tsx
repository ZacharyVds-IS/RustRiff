// @vitest-environment jsdom
import React, {act} from "react";
import {createRoot, Root} from "react-dom/client";
import {beforeAll, beforeEach, describe, expect, it, vi} from "vitest";

describe("StepParameterSelection", () => {
    let container: HTMLDivElement;
    let root: Root;
    let StepParameterSelection: typeof import("../../components/dialogs/MidiBindingDialog/steps/StepParameterSelection").StepParameterSelection;

    beforeAll(async () => {
        (globalThis as { IS_REACT_ACT_ENVIRONMENT?: boolean }).IS_REACT_ACT_ENVIRONMENT = true;
        StepParameterSelection = (await import("../../components/dialogs/MidiBindingDialog/steps/StepParameterSelection")).StepParameterSelection;
    });

    beforeEach(() => {
        vi.clearAllMocks();
        container = document.createElement("div");
        document.body.appendChild(container);
        root = createRoot(container);
    });

    const paramOptions = [
        { value: "ToggleBypass", label: "On / Off", description: "Toggle the effect", icon: "⏻" },
        { value: "WahPedalPosition", label: "Sweep", description: "Control wah sweep", icon: "〰" },
    ];

    describe("success_path", () => {
        it("renders step title", async () => {
            await act(async () => {
                root.render(
                    <StepParameterSelection
                        paramOptions={paramOptions}
                        selectedParam="ToggleBypass"
                        effectBindings={[]}
                        onSelectParam={vi.fn()}
                    />
                );
            });

            expect(container.textContent).toContain("Step 1");
            expect(container.textContent).toContain("Parameter selection");
        });

        it("renders all parameter options", async () => {
            await act(async () => {
                root.render(
                    <StepParameterSelection
                        paramOptions={paramOptions}
                        selectedParam="ToggleBypass"
                        effectBindings={[]}
                        onSelectParam={vi.fn()}
                    />
                );
            });

            expect(container.textContent).toContain("On / Off");
            expect(container.textContent).toContain("Sweep");
        });

        it("shows mapped badge for bound parameters", async () => {
            const effectBindings = [{ parameter: "WahPedalPosition" }];

            await act(async () => {
                root.render(
                    <StepParameterSelection
                        paramOptions={paramOptions}
                        selectedParam="ToggleBypass"
                        effectBindings={effectBindings}
                        onSelectParam={vi.fn()}
                    />
                );
            });

            // The Sweep option should show "mapped" badge
            expect(container.textContent).toContain("mapped");
        });

        it("renders empty state when no options provided", async () => {
            await act(async () => {
                root.render(
                    <StepParameterSelection
                        paramOptions={[]}
                        selectedParam="ToggleBypass"
                        effectBindings={[]}
                        onSelectParam={vi.fn()}
                    />
                );
            });

            expect(container.textContent).toContain("Parameter selection");
        });
    });
});
