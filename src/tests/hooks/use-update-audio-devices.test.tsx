// @vitest-environment jsdom
import React, {act, useEffect} from "react";
import {createRoot, Root} from "react-dom/client";
import {beforeAll, beforeEach, describe, expect, it, vi} from "vitest";
import {setInputDevice, setOutputDevice} from "../../domain";
import {useUpdateAudioDevices} from "../../hooks/useUpdateAudioDevices";

vi.mock("../../domain", () => ({
    setInputDevice: vi.fn(),
    setOutputDevice: vi.fn(),
}));

type HookValue = ReturnType<typeof useUpdateAudioDevices>;

function Probe({onChange}: {onChange: (value: HookValue) => void}) {
    const value = useUpdateAudioDevices();

    useEffect(() => {
        onChange(value);
    }, [value, onChange]);

    return null;
}

const flush = () => new Promise((resolve) => setTimeout(resolve, 0));

function requireLatest(value: HookValue | null): HookValue {
    expect(value).not.toBeNull();
    return value as HookValue;
}

describe("useUpdateAudioDevices", () => {
    let container: HTMLDivElement;
    let root: Root;

    beforeAll(() => {
        (globalThis as {IS_REACT_ACT_ENVIRONMENT?: boolean}).IS_REACT_ACT_ENVIRONMENT = true;
    });

    beforeEach(() => {
        vi.clearAllMocks();
        container = document.createElement("div");
        document.body.appendChild(container);
        root = createRoot(container);
    });

    describe("success_path", () => {
        it("starts with isSetting=false and null error before any action", async () => {
            // Arrange & Act
            let latest: HookValue | null = null;
            await act(async () => {
                root.render(<Probe onChange={(v) => (latest = v)} />);
            });

            // Assert initial defaults
            const initial = requireLatest(latest);
            expect(initial.isSetting).toBe(false);
            expect(initial.error).toBeNull();

            await act(async () => { root.unmount(); });
        });

        it("sets isSetting=true while updateInputDevice is in-flight", async () => {
            // Arrange – gate the backend call
            let resolveSet: () => void;
            vi.mocked(setInputDevice).mockReturnValueOnce(
                new Promise<undefined>((res) => { resolveSet = () => res(undefined); })
            );
            let latest: HookValue | null = null;

            await act(async () => {
                root.render(<Probe onChange={(v) => (latest = v)} />);
            });

            // Act – start call, don't await completion
            act(() => { void requireLatest(latest).updateInputDevice("in-1"); });
            await act(async () => { await Promise.resolve(); });

            expect(requireLatest(latest).isSetting).toBe(true);

            resolveSet!();
            await act(async () => { await flush(); await flush(); });
            expect(requireLatest(latest).isSetting).toBe(false);

            await act(async () => { root.unmount(); });
        });

        it("sets isSetting=true while updateOutputDevice is in-flight", async () => {
            // Arrange – gate the backend call
            let resolveSet: () => void;
            vi.mocked(setOutputDevice).mockReturnValueOnce(
                new Promise<undefined>((res) => { resolveSet = () => res(undefined); })
            );
            let latest: HookValue | null = null;

            await act(async () => {
                root.render(<Probe onChange={(v) => (latest = v)} />);
            });

            // Act – start call, don't await completion
            act(() => { void requireLatest(latest).updateOutputDevice("out-1"); });
            await act(async () => { await Promise.resolve(); });

            expect(requireLatest(latest).isSetting).toBe(true);

            resolveSet!();
            await act(async () => { await flush(); await flush(); });
            expect(requireLatest(latest).isSetting).toBe(false);

            await act(async () => { root.unmount(); });
        });

        it("updates input and output devices successfully", async () => {
            // Arrange
            vi.mocked(setInputDevice).mockResolvedValue(undefined);
            vi.mocked(setOutputDevice).mockResolvedValue(undefined);
            let latest: HookValue | null = null;

            await act(async () => {
                root.render(<Probe onChange={(value) => (latest = value)} />);
                await flush();
            });

            const latestValue = requireLatest(latest);

            // Act
            await act(async () => {
                await latestValue.updateInputDevice("input-1");
                await latestValue.updateOutputDevice("output-1");
                await flush();
                await flush();
            });

            const updatedValue = requireLatest(latest);

            // Assert
            expect(setInputDevice).toHaveBeenCalledWith({deviceId: "input-1"});
            expect(setOutputDevice).toHaveBeenCalledWith({deviceId: "output-1"});
            expect(updatedValue.isSetting).toBe(false);
            expect(updatedValue.error).toBeNull();

            await act(async () => { root.unmount(); });
        });
    });

    describe("failure_path", () => {
        it("captures error when updateInputDevice fails (Error instance)", async () => {
            // Arrange
            vi.mocked(setInputDevice).mockRejectedValueOnce(new Error("set input failed"));
            let latest: HookValue | null = null;

            await act(async () => {
                root.render(<Probe onChange={(value) => (latest = value)} />);
                await flush();
            });

            const latestValue = requireLatest(latest);

            // Act
            await act(async () => {
                await latestValue.updateInputDevice("bad-input");
                await flush();
                await flush();
            });

            const updatedValue = requireLatest(latest);

            // Assert
            expect(setInputDevice).toHaveBeenCalledWith({deviceId: "bad-input"});
            expect(updatedValue.isSetting).toBe(false);
            expect(updatedValue.error).toBe("set input failed");

            await act(async () => { root.unmount(); });
        });

        it("captures error when updateOutputDevice fails", async () => {
            // Arrange
            vi.mocked(setOutputDevice).mockRejectedValueOnce(new Error("set output failed"));
            let latest: HookValue | null = null;

            await act(async () => {
                root.render(<Probe onChange={(v) => (latest = v)} />);
                await flush();
            });

            await act(async () => {
                await requireLatest(latest).updateOutputDevice("bad-output");
                await flush();
                await flush();
            });

            const updatedValue = requireLatest(latest);

            // Assert
            expect(setOutputDevice).toHaveBeenCalledWith({deviceId: "bad-output"});
            expect(updatedValue.isSetting).toBe(false);
            expect(updatedValue.error).toBe("set output failed");

            await act(async () => { root.unmount(); });
        });

        it("uses String(err) for non-Error input device failures", async () => {
            // Arrange – throw a plain string to exercise the `String(err)` branch
            vi.mocked(setInputDevice).mockRejectedValueOnce("device-busy");
            let latest: HookValue | null = null;

            await act(async () => {
                root.render(<Probe onChange={(v) => (latest = v)} />);
                await flush();
            });

            await act(async () => {
                await requireLatest(latest).updateInputDevice("dev");
                await flush();
                await flush();
            });

            // String("device-busy") === "device-busy"
            expect(requireLatest(latest).error).toBe("device-busy");
            expect(requireLatest(latest).isSetting).toBe(false);

            await act(async () => { root.unmount(); });
        });
    });
});
