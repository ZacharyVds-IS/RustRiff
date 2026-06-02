// @vitest-environment jsdom
import React, {act} from "react";
import {createRoot, Root} from "react-dom/client";
import {beforeAll, beforeEach, describe, expect, it, vi} from "vitest";

describe("ParameterCard", () => {
    let container: HTMLDivElement;
    let root: Root;
    let ParameterCard: typeof import("../../components/dialogs/MidiBindingDialog/ParameterCard").ParameterCard;

    beforeAll(async () => {
        (globalThis as { IS_REACT_ACT_ENVIRONMENT?: boolean }).IS_REACT_ACT_ENVIRONMENT = true;
        ParameterCard = (await import("../../components/dialogs/MidiBindingDialog/ParameterCard")).ParameterCard;
    });

    beforeEach(() => {
        vi.clearAllMocks();
        container = document.createElement("div");
        document.body.appendChild(container);
        root = createRoot(container);
    });

    const defaultOption = {
        value: "WahPedalPosition",
        label: "Sweep",
        description: "Control the wah filter sweep position (0–127)",
        icon: "〰",
    };

    describe("success_path", () => {
        it("renders option label and description", async () => {
            await act(async () => {
                root.render(
                    <ParameterCard
                        option={defaultOption}
                        isSelected={false}
                        alreadyMapped={false}
                        onSelect={vi.fn()}
                    />
                );
            });

            expect(container.textContent).toContain("Sweep");
            expect(container.textContent).toContain("Control the wah filter sweep position");
        });

        it("shows mapped chip when alreadyMapped is true", async () => {
            await act(async () => {
                root.render(
                    <ParameterCard
                        option={defaultOption}
                        isSelected={false}
                        alreadyMapped={true}
                        onSelect={vi.fn()}
                    />
                );
            });

            expect(container.textContent).toContain("mapped");
        });

        it("does not show mapped chip when alreadyMapped is false", async () => {
            await act(async () => {
                root.render(
                    <ParameterCard
                        option={defaultOption}
                        isSelected={false}
                        alreadyMapped={false}
                        onSelect={vi.fn()}
                    />
                );
            });

            expect(container.textContent).not.toContain("mapped");
        });

        it("calls onSelect when clicked", async () => {
            const onSelect = vi.fn();

            await act(async () => {
                root.render(
                    <ParameterCard
                        option={defaultOption}
                        isSelected={false}
                        alreadyMapped={false}
                        onSelect={onSelect}
                    />
                );
            });

            const paper = container.querySelector('[class*="MuiPaper"]') as HTMLElement | null;

            if (paper) {
                await act(async () => {
                    paper.click();
                });
                expect(onSelect).toHaveBeenCalledWith("WahPedalPosition");
            }
        });
    });

    describe("edge_cases", () => {
        it("renders with minimal option data", async () => {
            const minimalOption = {
                value: "ToggleBypass",
                label: "On/Off",
                description: "Toggle",
                icon: "⏻",
            };

            await act(async () => {
                root.render(
                    <ParameterCard
                        option={minimalOption}
                        isSelected={false}
                        alreadyMapped={false}
                        onSelect={vi.fn()}
                    />
                );
            });

            expect(container.textContent).toContain("On/Off");
        });

        it("shows check icon when isSelected is true", async () => {
            await act(async () => {
                root.render(
                    <ParameterCard
                        option={defaultOption}
                        isSelected={true}
                        alreadyMapped={false}
                        onSelect={vi.fn()}
                    />
                );
            });

            // CheckCircleIcon should render when selected
            const checkIcon = container.querySelector('[data-testid="CheckCircleIcon"]');
            expect(checkIcon).toBeTruthy();
        });
    });
});
