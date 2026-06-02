// @vitest-environment jsdom
import React from "react";
import {act, cleanup, render, screen} from "@testing-library/react";
import {beforeEach, describe, expect, it, vi} from "vitest";
import {MainScreen} from "../../screens/MainScreen";

function createEffectA() {
    return {
        kind: "Delay",
        data: {
            id: "effect-a",
            name: "Effect A",
            is_active: true,
            delay_time: 200,
            level: 0.5,
            color: "#123456",
        },
    };
}

function createEffectB() {
    return {
        kind: "Delay",
        data: {
            id: "effect-b",
            name: "Effect B",
            is_active: true,
            delay_time: 260,
            level: 0.6,
            color: "#654321",
        },
    };
}

const storeState = vi.hoisted(() => ({
    channels: [
        {
            id: "channel-1",
            effect_chain: [
                {
                    kind: "Delay",
                    data: {
                        id: "effect-a",
                        name: "Effect A",
                        is_active: true,
                        delay_time: 200,
                        level: 0.5,
                        color: "#123456",
                    },
                },
                {
                    kind: "Delay",
                    data: {
                        id: "effect-b",
                        name: "Effect B",
                        is_active: true,
                        delay_time: 260,
                        level: 0.6,
                        color: "#654321",
                    },
                },
            ],
        },
    ],
    current_channel: "channel-1",
    is_active: false,
    updateEffectActiveState: vi.fn(),
    moveEffect: vi.fn().mockResolvedValue(undefined),
    applyChangesToChainOrder: vi.fn().mockResolvedValue(undefined),
    setIsActive: vi.fn((next: boolean) => {
        storeState.is_active = next;
    }),
}));

const useAmpStoreMock = vi.hoisted(() => {
    const fn: any = vi.fn((selector?: (state: any) => any) =>
        typeof selector === "function" ? selector(storeState) : storeState,
    );
    fn.getState = () => storeState;
    return fn;
});

type HotkeyBinding = {
    keys: string[];
    callback: (event?: unknown, handler?: {keys?: string[]}) => void;
};

const hotkeyBindings = vi.hoisted<HotkeyBinding[]>(() => []);

const toggleEffectMock = vi.hoisted(() => vi.fn().mockResolvedValue(true));

vi.mock("../../state/AmpConfigStore.tsx", () => ({
    useAmpStore: useAmpStoreMock,
}));

vi.mock("../../domain", () => ({
    toggleEffect: toggleEffectMock,
}));

vi.mock("react-hotkeys-hook", () => ({
    useHotkeys: (
        keys: string | string[],
        callback: (event?: unknown, handler?: {keys?: string[]}) => void,
    ) => {
        hotkeyBindings.push({
            keys: Array.isArray(keys) ? keys : [keys],
            callback,
        });
    },
}));

vi.mock("../../components/EffectChain.tsx", () => ({
    EffectChain: ({selected, onOpenKeybinds}: {selected: any; onOpenKeybinds?: () => void}) => (
        <div>
            <div data-testid="selected-chain-item">{selected === "amp" ? "amp" : selected.data.id}</div>
            <button onClick={onOpenKeybinds}>open-keybinds</button>
        </div>
    ),
}));

vi.mock("../../components/DefaultAmpControls.tsx", () => ({
    DefaultAmpControls: () => <div>amp-controls</div>,
}));

vi.mock("../../components/EffectPedal.tsx", () => ({
    EffectPedal: ({effect}: {effect: any}) => <div>{`effect-pedal-${effect.data.id}`}</div>,
}));

vi.mock("../../components/CabinetEffect.tsx", () => ({
    CabinetEffect: ({effect}: {effect: any}) => <div>{`cabinet-${effect.data.id}`}</div>,
}));

vi.mock("../../components/dialogs/KeybindsDialog.tsx", () => ({
    KeybindsDialog: ({open}: {open: boolean}) => <div>{open ? "keybinds-open" : "keybinds-closed"}</div>,
}));

function triggerHotkey(key: string, options?: {shiftKey?: boolean}) {
    const bindingKey = options?.shiftKey ? `shift+${key}` : key;
    const binding = [...hotkeyBindings].reverse().find((entry) => entry.keys.includes(bindingKey));
    if (!binding) {
        throw new Error(`No hotkey binding found for key: ${bindingKey}`);
    }
    binding.callback(
        {
            shiftKey: options?.shiftKey ?? false,
            key: key === "arrowleft" ? "ArrowLeft" : key === "arrowright" ? "ArrowRight" : key,
        },
        {keys: [bindingKey]},
    );
}

describe("MainScreen keybind logic", () => {
    beforeEach(() => {
        cleanup();
        vi.clearAllMocks();
        hotkeyBindings.length = 0;
        storeState.channels = [
            {
                id: "channel-1",
                effect_chain: [createEffectA(), createEffectB()],
            },
        ];
        storeState.current_channel = "channel-1";
        storeState.is_active = false;
    });

    describe("success_path", () => {
        describe("selection keys", () => {
            it("selects first effect with key 2 and returns to amp with key 1", () => {
                render(<MainScreen/>);
                expect(screen.getByText("amp-controls")).toBeTruthy();

                act(() => {
                    triggerHotkey("2");
                });
                expect(screen.getByText("effect-pedal-effect-a")).toBeTruthy();

                act(() => {
                    triggerHotkey("1");
                });
                expect(screen.getByText("amp-controls")).toBeTruthy();
            });

            it("selects the 9th effect with key 0", () => {
                storeState.channels = [
                    {
                        id: "channel-1",
                        effect_chain: Array.from({length: 9}, (_, index) => ({
                            kind: "Delay",
                            data: {
                                id: `effect-${index + 1}`,
                                name: `Effect ${index + 1}`,
                                is_active: true,
                                delay_time: 200 + index,
                                level: 0.5,
                                color: "#123456",
                            },
                        })),
                    },
                ];

                render(<MainScreen/>);

                act(() => {
                    triggerHotkey("0");
                });

                expect(screen.getByText("effect-pedal-effect-9")).toBeTruthy();
            });
        });

        describe("toggle keys", () => {
            it("toggles amp on/off with space when amp is selected", () => {
                render(<MainScreen/>);

                act(() => {
                    triggerHotkey("space");
                });

                expect(storeState.setIsActive).toHaveBeenCalledWith(true);
            });

            it("toggles selected effect with space and calls backend toggle", async () => {
                render(<MainScreen/>);

                act(() => {
                    triggerHotkey("2");
                });

                await act(async () => {
                    triggerHotkey("space");
                    await Promise.resolve();
                });

                expect(storeState.updateEffectActiveState).toHaveBeenCalledWith("effect-a", false);
                expect(toggleEffectMock).toHaveBeenCalledWith({effectId: "effect-a"});
            });
        });

        describe("navigation keys", () => {
            it("selects the first effect with ArrowRight when amp is selected", () => {
                render(<MainScreen/>);

                act(() => {
                    triggerHotkey("arrowright");
                });

                expect(screen.getByText("effect-pedal-effect-a")).toBeTruthy();
            });

            it("selects the last effect with ArrowLeft when amp is selected", () => {
                render(<MainScreen/>);

                act(() => {
                    triggerHotkey("arrowleft");
                });

                expect(screen.getByText("effect-pedal-effect-b")).toBeTruthy();
            });

            it("wraps from the first effect back to amp with ArrowLeft", () => {
                render(<MainScreen/>);

                act(() => {
                    triggerHotkey("2");
                });

                act(() => {
                    triggerHotkey("arrowleft");
                });

                expect(screen.getByText("amp-controls")).toBeTruthy();
            });

            it("wraps from the last effect back to amp with ArrowRight", () => {
                render(<MainScreen/>);

                act(() => {
                    triggerHotkey("3");
                });

                act(() => {
                    triggerHotkey("arrowright");
                });

                expect(screen.getByText("amp-controls")).toBeTruthy();
            });

            it("continues from amp to the last effect with ArrowLeft", () => {
                render(<MainScreen/>);

                act(() => {
                    triggerHotkey("2");
                });

                act(() => {
                    triggerHotkey("arrowleft");
                });

                act(() => {
                    triggerHotkey("arrowleft");
                });

                expect(screen.getByText("effect-pedal-effect-b")).toBeTruthy();
            });

            it("continues from amp to the first effect with ArrowRight", () => {
                render(<MainScreen/>);

                act(() => {
                    triggerHotkey("3");
                });

                act(() => {
                    triggerHotkey("arrowright");
                });

                act(() => {
                    triggerHotkey("arrowright");
                });

                expect(screen.getByText("effect-pedal-effect-a")).toBeTruthy();
            });

            it("wraps from the first effect to the last with ArrowLeft", () => {
                render(<MainScreen/>);

                act(() => {
                    triggerHotkey("2");
                });

                act(() => {
                    triggerHotkey("arrowleft");
                });

                act(() => {
                    triggerHotkey("arrowleft");
                });

                expect(screen.getByText("effect-pedal-effect-b")).toBeTruthy();
            });

            it("wraps from the last effect to the first with ArrowRight", () => {
                render(<MainScreen/>);

                act(() => {
                    triggerHotkey("3");
                });

                act(() => {
                    triggerHotkey("arrowright");
                });

                act(() => {
                    triggerHotkey("arrowright");
                });

                expect(screen.getByText("effect-pedal-effect-a")).toBeTruthy();
            });
        });

        describe("movement keys — reordering effect in chain", () => {
            it("moves selected effect right with Shift+ArrowRight and persists the new order", async () => {
                render(<MainScreen/>);

                act(() => {
                    triggerHotkey("2");
                });

                await act(async () => {
                    triggerHotkey("arrowright", {shiftKey: true});
                    await Promise.resolve();
                });

                // Store updated optimistically
                expect(storeState.moveEffect).toHaveBeenCalledWith(0, 1);
                // Backend persist was triggered
                expect(storeState.applyChangesToChainOrder).toHaveBeenCalledTimes(1);
            });

            it("moves the currently selected effect left by one slot with Shift+ArrowLeft", async () => {
                storeState.channels = [
                    {
                        id: "channel-1",
                        effect_chain: [
                            createEffectA(),
                            createEffectB(),
                            {
                                kind: "Delay",
                                data: {
                                    id: "effect-c",
                                    name: "Effect C",
                                    is_active: true,
                                    delay_time: 300,
                                    level: 0.7,
                                    color: "#abcdef",
                                },
                            },
                        ],
                    },
                ];

                render(<MainScreen/>);

                act(() => {
                    triggerHotkey("4");
                });

                await act(async () => {
                    triggerHotkey("arrowleft", {shiftKey: true});
                    await Promise.resolve();
                });

                expect(storeState.moveEffect).toHaveBeenCalledWith(2, 1);
                expect(storeState.applyChangesToChainOrder).toHaveBeenCalledTimes(1);
            });

            it("does not move the selected effect left past the first slot", async () => {
                render(<MainScreen/>);

                act(() => {
                    triggerHotkey("2");
                });

                await act(async () => {
                    triggerHotkey("arrowleft", {shiftKey: true});
                    await Promise.resolve();
                });

                expect(storeState.moveEffect).not.toHaveBeenCalled();
                expect(storeState.applyChangesToChainOrder).not.toHaveBeenCalled();
            });

            it("does not move the selected effect right past the last slot", async () => {
                render(<MainScreen/>);

                act(() => {
                    triggerHotkey("3");
                });

                await act(async () => {
                    triggerHotkey("arrowright", {shiftKey: true});
                    await Promise.resolve();
                });

                expect(storeState.moveEffect).not.toHaveBeenCalled();
                expect(storeState.applyChangesToChainOrder).not.toHaveBeenCalled();
            });
        });
        describe("movement keys — no-op when amp is selected", () => {
            it("does not move effects with Shift+arrows while amp is selected", () => {
                render(<MainScreen/>);

                act(() => {
                    triggerHotkey("arrowleft", {shiftKey: true});
                    triggerHotkey("arrowright", {shiftKey: true});
                });

                expect(storeState.moveEffect).not.toHaveBeenCalled();
                expect(storeState.applyChangesToChainOrder).not.toHaveBeenCalled();
            });
        });

        describe("toggle keys — rollback on backend rejection", () => {
            it("rolls back the optimistic active-state update when toggleEffect rejects", async () => {
                toggleEffectMock.mockRejectedValueOnce(new Error("backend error"));

                render(<MainScreen/>);

                // Select effect A (is_active: true)
                act(() => {
                    triggerHotkey("2");
                });

                await act(async () => {
                    triggerHotkey("space");
                    // flush the rejected promise
                    await Promise.resolve();
                    await Promise.resolve();
                });

                // First call: optimistic toggle off
                expect(storeState.updateEffectActiveState).toHaveBeenNthCalledWith(1, "effect-a", false);
                // Second call: rollback to original value
                expect(storeState.updateEffectActiveState).toHaveBeenNthCalledWith(2, "effect-a", true);
                expect(storeState.updateEffectActiveState).toHaveBeenCalledTimes(2);
            });
        });

        describe("movement keys — no-op when chain order persist rejects", () => {
            it("reverses the optimistic move when applyChangesToChainOrder rejects", async () => {
                storeState.applyChangesToChainOrder.mockRejectedValueOnce(new Error("persist error"));

                render(<MainScreen/>);

                act(() => {
                    triggerHotkey("2");
                });

                await act(async () => {
                    triggerHotkey("arrowright", {shiftKey: true});
                    await Promise.resolve();
                    await Promise.resolve();
                });

                // First call: optimistic move forward
                expect(storeState.moveEffect).toHaveBeenNthCalledWith(1, 0, 1);
                // Second call: rollback to original position
                expect(storeState.moveEffect).toHaveBeenNthCalledWith(2, 1, 0);
                expect(storeState.moveEffect).toHaveBeenCalledTimes(2);
            });
        });
    });
});