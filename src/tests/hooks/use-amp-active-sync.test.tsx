// @vitest-environment jsdom
import React, {act} from "react";
import {createRoot, Root} from "react-dom/client";
import {beforeAll, beforeEach, describe, expect, it, vi} from "vitest";
import {useAmpActiveSync} from "../../hooks/useAmpActiveSync";
import {useAmpStore} from "../../state/AmpConfigStore";
import * as domain from "../../domain";
import {listen} from "@tauri-apps/api/event";

// Mock domain functions
vi.mock("../../domain", () => ({
    getAmpConfig: vi.fn(),
    addChannel: vi.fn(),
    addEffect: vi.fn(),
    applyEffectOrderChange: vi.fn(),
    removeChannel: vi.fn(),
    removeEffect: vi.fn(),
    setBass: vi.fn(),
    setChannelId: vi.fn(),
    setGain: vi.fn(),
    setMasterVolume: vi.fn(),
    setMiddle: vi.fn(),
    setTreble: vi.fn(),
    setVolume: vi.fn(),
    toggleOnOff: vi.fn(),
}));

// Mock Tauri event
vi.mock("@tauri-apps/api/event", () => ({
    listen: vi.fn((eventName, callback) => {
        // Store the callback for testing
        ;(listen as any).lastCallback = callback;
        return Promise.resolve(() => {});
    }),
}));

type HookSetup = {
    container: HTMLDivElement;
    root: Root;
    unmount: () => Promise<void>;
};

function setupHook(): HookSetup {
    const container = document.createElement("div");
    document.body.appendChild(container);
    const root = createRoot(container);

    function Component() {
        useAmpActiveSync();
        return null;
    }

    root.render(<Component />);

    return {
        container,
        root,
        unmount: async () => {
            await act(async () => {
                root.unmount();
            });
            document.body.removeChild(container);
        },
    };
}

const flush = () => new Promise((resolve) => setTimeout(resolve, 0));

describe("useAmpActiveSync", () => {
    beforeAll(() => {
        (globalThis as {IS_REACT_ACT_ENVIRONMENT?: boolean}).IS_REACT_ACT_ENVIRONMENT = true;
    });

    beforeEach(() => {
        vi.clearAllMocks();
        useAmpStore.setState({
            is_active: false,
            master_volume: 1,
            current_channel: 0,
            chain_snapshot: null,
            channels: [
                {
                    id: 0,
                    name: "Clean",
                    gain: 1,
                    tone_stack: {bass: 0.5, middle: 0.5, treble: 0.5},
                    volume: 1,
                    effect_chain: [],
                },
            ],
        });
    });

    describe("success_path", () => {
        it("initializes amp store on mount", async () => {
            // Arrange
            const mockConfig = {
                master_volume: 0.8,
                is_active: true,
                current_channel: 0,
                channels: [
                    {
                        id: 0,
                        name: "Clean",
                        gain: 1,
                        tone_stack: {bass: 0.5, middle: 0.5, treble: 0.5},
                        volume: 1,
                        effect_chain: [],
                    },
                ],
            };
            vi.mocked(domain.getAmpConfig).mockResolvedValueOnce(mockConfig as any);

            // Act
            const {unmount} = setupHook();
            await act(async () => {
                await flush();
                await flush();
            });

            // Assert
            expect(domain.getAmpConfig).toHaveBeenCalledTimes(1);
            await unmount();
        });

        it("listens to amp state change events", async () => {
            // Arrange
            vi.mocked(domain.getAmpConfig).mockResolvedValueOnce({
                master_volume: 1,
                is_active: false,
                current_channel: 0,
                channels: [],
            } as any);

            // Act
            const {unmount} = setupHook();
            await act(async () => {
                await flush();
                await flush();
            });

            // Trigger event callback
            const callback = (listen as any).lastCallback;
            if (callback) {
                callback({payload: true});
            }

            // Assert
            expect(listen).toHaveBeenCalled();
            await unmount();
        });

        it("updates store state when amp active event is received", async () => {
            // Arrange
            vi.mocked(domain.getAmpConfig).mockResolvedValueOnce({
                master_volume: 1,
                is_active: false,
                current_channel: 0,
                channels: [],
            } as any);
            const initialState = useAmpStore.getState().is_active;

            // Act
            const {unmount} = setupHook();
            await act(async () => {
                await flush();
            });

            // Simulate event from backend
            const callback = (listen as any).lastCallback;
            if (callback) {
                callback({payload: true});
            }

            // Assert
            expect(useAmpStore.getState().is_active).toBe(true);
            expect(initialState).toBe(false);
            await unmount();
        });

        it("handles multiple state changes from events", async () => {
            // Arrange
            vi.mocked(domain.getAmpConfig).mockResolvedValueOnce({
                master_volume: 1,
                is_active: false,
                current_channel: 0,
                channels: [],
            } as any);

            // Act
            const {unmount} = setupHook();
            await act(async () => {
                await flush();
            });

            const callback = (listen as any).lastCallback;

            // Simulate multiple events
            callback({payload: true});
            callback({payload: false});
            callback({payload: true});

            // Assert
            expect(useAmpStore.getState().is_active).toBe(true);
            await unmount();
        });
    });

    describe("failure_path", () => {
        it("logs exact error message from useAmpActiveSync when listen() itself throws", async () => {
            // Arrange – init() succeeds, but listen() rejects (the catch in useAmpActiveSync
            // is only reached if something in sync() after init() throws, e.g. listen itself)
            const listenError = new Error("listen registration failed");
            vi.mocked(domain.getAmpConfig).mockResolvedValueOnce({
                master_volume: 1, is_active: false, current_channel: 0, channels: [],
            } as any);
            vi.mocked(listen).mockRejectedValueOnce(listenError);
            const consoleErrorSpy = vi.spyOn(console, "error").mockImplementation(() => undefined);

            // Act
            const {unmount} = setupHook();
            await act(async () => {
                await flush();
                await flush();
            });

            // Assert – the useAmpActiveSync catch block fires with exact message
            expect(consoleErrorSpy).toHaveBeenCalledWith(
                "[useAmpActiveSync] Failed to initialize amp state sync:",
                listenError,
            );

            consoleErrorSpy.mockRestore();
            await unmount();
        });

        it("does not call listen when disposed=true before init completes (unmount during init)", async () => {
            // Arrange – block getAmpConfig indefinitely so init() never resolves during this test
            vi.mocked(domain.getAmpConfig).mockImplementation(() => new Promise(() => { /* never resolves */ }));

            // Act – mount then unmount immediately within the same tick
            const container2 = document.createElement("div");
            document.body.appendChild(container2);
            const root2 = createRoot(container2);
            await act(async () => {
                root2.render(React.createElement(() => { useAmpActiveSync(); return null; }));
                root2.unmount();   // cleanup fires, disposed=true
            });
            document.body.removeChild(container2);
            await act(async () => { await flush(); await flush(); });

            // listen must NOT have been called since init never resolved and disposed=true
            expect(listen).not.toHaveBeenCalled();

            // Reset the mock so subsequent tests aren't affected
            vi.mocked(domain.getAmpConfig).mockReset();
            vi.mocked(domain.getAmpConfig).mockImplementation(() => Promise.resolve(undefined as any));
        });

        it("logs error when init fails but continues listening", async () => {
            // Arrange
            const consoleErrorSpy = vi.spyOn(console, "error").mockImplementation(() => undefined);
            const error = new Error("init failed");
            vi.mocked(domain.getAmpConfig).mockRejectedValueOnce(error);

            // Act
            const {unmount} = setupHook();
            await act(async () => {
                await flush();
                await flush();
            });

            // Assert
            // The hook logs from useAmpStore's init, which calls getAmpConfig
            // So we'll see "Failed to fetch init state from Rust" from the store
            expect(consoleErrorSpy).toHaveBeenCalled();
            const errorCalls = consoleErrorSpy.mock.calls;
            const hasInitError = errorCalls.some((call) =>
                call[0]?.includes("Failed to fetch init state from Rust") && call[1] instanceof Error
            );
            expect(hasInitError).toBe(true);
            consoleErrorSpy.mockRestore();
            await unmount();
        });

        it("does not update state if component unmounts during init", async () => {
            // Arrange
            // Create a promise that we can control when it resolves
            let resolveInit: any = null;
            const initPromise = new Promise((resolve: any) => {
                resolveInit = () =>
                    resolve({
                        master_volume: 1,
                        is_active: true,
                        current_channel: 0,
                        channels: [],
                    });
            });

            vi.mocked(domain.getAmpConfig).mockReturnValueOnce(initPromise as any);
            const initialIsActive = useAmpStore.getState().is_active;

            // Act
            const {unmount} = setupHook();

            // Unmount before init completes
            await unmount();

            // Now resolve the init promise
            if (resolveInit) {
                resolveInit();
            }
            await act(async () => {
                await flush();
            });

            // Assert - State should not have changed even though init resolved
            expect(useAmpStore.getState().is_active).toBe(initialIsActive);
        });

        it("ignores events received after unmount", async () => {
            // Arrange
            vi.mocked(domain.getAmpConfig).mockResolvedValueOnce({
                master_volume: 1,
                is_active: false,
                current_channel: 0,
                channels: [],
            } as any);

            // Act
            const {unmount} = setupHook();
            await act(async () => {
                await flush();
            });

            const initialIsActive = useAmpStore.getState().is_active;

            // Unmount
            await unmount();

            // Try to trigger event after unmount
            const callback = (listen as any).lastCallback;
            if (callback) {
                callback({payload: true});
            }

            // Assert - State should not change
            expect(useAmpStore.getState().is_active).toBe(initialIsActive);
        });
    });
});


