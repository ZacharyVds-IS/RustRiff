// @vitest-environment jsdom
import React, {act} from "react";
import {createRoot, Root} from "react-dom/client";
import {beforeAll, beforeEach, describe, expect, it, vi} from "vitest";

describe("StepCcAssignment", () => {
    let container: HTMLDivElement;
    let root: Root;
    let StepCcAssignment: typeof import("../../components/dialogs/MidiBindingDialog/steps/StepCcAssignment").StepCcAssignment;

    beforeAll(async () => {
        (globalThis as { IS_REACT_ACT_ENVIRONMENT?: boolean }).IS_REACT_ACT_ENVIRONMENT = true;
        StepCcAssignment = (await import("../../components/dialogs/MidiBindingDialog/steps/StepCcAssignment")).StepCcAssignment;
    });

    beforeEach(() => {
        vi.clearAllMocks();
        container = document.createElement("div");
        document.body.appendChild(container);
        root = createRoot(container);
    });

    const defaultProps = {
        isLearning: false,
        onToggleLearning: vi.fn(),
        midiChannel: 1,
        onMidiChannelChange: vi.fn(),
        ccNumber: 7,
        onCcNumberChange: vi.fn(),
        successMessage: null,
        onCloseSuccess: vi.fn(),
        selectedParamLabel: "Sweep",
        effectName: "Crybaby",
    };

    describe("success_path", () => {
        it("renders step title and recognize button", async () => {
            await act(async () => {
                root.render(<StepCcAssignment {...defaultProps} />);
            });

            expect(container.textContent).toContain("Step 2");
            expect(container.textContent).toContain("Assign CC Input");
            expect(container.textContent).toContain("Recognize button");
        });

        it("shows listening state when isLearning is true", async () => {
            await act(async () => {
                root.render(<StepCcAssignment {...defaultProps} isLearning={true} />);
            });

            expect(container.textContent).toContain("Listening");
            expect(container.textContent).toContain("press a MIDI controller now");
        });

        it("shows summary with channel, CC, param and effect name", async () => {
            await act(async () => {
                root.render(<StepCcAssignment {...defaultProps} />);
            });

            expect(container.textContent).toContain("CH 1");
            expect(container.textContent).toContain("CC #7");
            expect(container.textContent).toContain("Sweep");
            expect(container.textContent).toContain("Crybaby");
        });

        it("shows success message when provided", async () => {
            await act(async () => {
                root.render(
                    <StepCcAssignment
                        {...defaultProps}
                        successMessage="Detected! Channel 1, CC #7"
                    />
                );
            });

            expect(container.textContent).toContain("Detected! Channel 1, CC #7");
        });

        it("shows advanced manual settings button", async () => {
            await act(async () => {
                root.render(<StepCcAssignment {...defaultProps} />);
            });

            expect(container.textContent).toContain("Advanced Manual Settings");
        });
    });

    describe("failure_path", () => {
        it("does not show success alert when successMessage is null", async () => {
            await act(async () => {
                root.render(<StepCcAssignment {...defaultProps} successMessage={null} />);
            });

            const alerts = container.querySelectorAll('[class*="MuiAlert"]');
            expect(alerts.length).toBe(0);
        });
    });

    describe("edge_cases", () => {
        it("handles undefined selectedParamLabel", async () => {
            await act(async () => {
                root.render(
                    <StepCcAssignment {...defaultProps} selectedParamLabel={undefined} />
                );
            });

            // Should still render without the summary card
            expect(container.textContent).toContain("Step 2");
        });

        it("handles extreme MIDI channel values", async () => {
            await act(async () => {
                root.render(
                    <StepCcAssignment {...defaultProps} midiChannel={16} ccNumber={127} />
                );
            });

            expect(container.textContent).toContain("CH 16");
            expect(container.textContent).toContain("CC #127");
        });
    });
});
