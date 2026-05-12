// @vitest-environment jsdom
import React, {act, useEffect} from "react";
import {createRoot, Root} from "react-dom/client";
import {beforeAll, beforeEach, describe, expect, it, vi} from "vitest";
import {getInputDeviceList, getOutputDeviceList} from "../../domain";
import {useAudioDevices} from "../../hooks/useAudioDevices";

vi.mock("../../domain", () => ({
    getInputDeviceList: vi.fn(),
    getOutputDeviceList: vi.fn(),
}));

type HookValue = ReturnType<typeof useAudioDevices>;

function Probe({onChange, onFirstRender}: {
    onChange: (value: HookValue) => void;
    onFirstRender?: (value: HookValue) => void;
}) {
    const value = useAudioDevices();
    const capturedFirst = React.useRef(false);

    // Capture value synchronously during first render, BEFORE any effects run
    if (!capturedFirst.current) {
        capturedFirst.current = true;
        onFirstRender?.(value);
    }

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

describe("useAudioDevices", () => {
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
        it("starts with isLoading=true and empty arrays before fetch resolves", async () => {
            // Arrange – gate the fetch so we can observe the loading state
            let resolveInputs: (v: any) => void;
            let resolveOutputs: (v: any) => void;
            const inputsPromise = new Promise<any>((res) => { resolveInputs = res; });
            const outputsPromise = new Promise<any>((res) => { resolveOutputs = res; });
            vi.mocked(getInputDeviceList).mockReturnValueOnce(inputsPromise);
            vi.mocked(getOutputDeviceList).mockReturnValueOnce(outputsPromise);
            let latest: HookValue | null = null;
            let firstRender: HookValue | null = null;

            // Act – render but do not flush so fetch is still in-flight
            await act(async () => {
                root.render(<Probe onChange={(v) => (latest = v)} onFirstRender={(v) => (firstRender = v)} />);
            });

            // Assert initial state captured during FIRST RENDER (before effects ran)
            const initial = requireLatest(firstRender);
            expect(initial.isLoading).toBe(true);
            expect(initial.inputs).toEqual([]);
            expect(initial.outputs).toEqual([]);
            expect(initial.error).toBeNull();

            // Cleanup - resolve fetches then unmount
            resolveInputs!([]);
            resolveOutputs!([]);
            await act(async () => { await flush(); await flush(); });
            await act(async () => { root.unmount(); });
        });

        it("sets isLoading=true at the start of each fetch cycle", async () => {
            // Arrange – first call succeeds, then we track second call
            const inputs = [{id: "in-1", name: "Mic", sample_rate: 48000}];
            const outputs = [{id: "out-1", name: "Speaker", sample_rate: 48000}];
            vi.mocked(getInputDeviceList).mockResolvedValue(inputs as any);
            vi.mocked(getOutputDeviceList).mockResolvedValue(outputs as any);
            let latest: HookValue | null = null;

            await act(async () => {
                root.render(<Probe onChange={(v) => (latest = v)} />);
                await flush();
                await flush();
            });

            // Ensure settled first
            expect(requireLatest(latest).isLoading).toBe(false);

            // Now gate the refresh call
            let resolveRefreshInputs: (v: any) => void;
            let resolveRefreshOutputs: (v: any) => void;
            vi.mocked(getInputDeviceList).mockReturnValueOnce(
                new Promise<any>((res) => { resolveRefreshInputs = res; })
            );
            vi.mocked(getOutputDeviceList).mockReturnValueOnce(
                new Promise<any>((res) => { resolveRefreshOutputs = res; })
            );

            // Trigger refresh without resolving
            act(() => { requireLatest(latest).refresh(); });
            await act(async () => { await Promise.resolve(); });

            expect(requireLatest(latest).isLoading).toBe(true);

            resolveRefreshInputs!([]);
            resolveRefreshOutputs!([]);
            await act(async () => { await flush(); await flush(); });
            await act(async () => { root.unmount(); });
        });

        it("loads input and output devices", async () => {
            // Arrange
            const inputs = [{id: "in-1", name: "Input 1", sample_rate: 48000}];
            const outputs = [{id: "out-1", name: "Output 1", sample_rate: 48000}];
            vi.mocked(getInputDeviceList).mockResolvedValueOnce(inputs as any);
            vi.mocked(getOutputDeviceList).mockResolvedValueOnce(outputs as any);
            let latest: HookValue | null = null;

            // Act
            await act(async () => {
                root.render(<Probe onChange={(value) => (latest = value)} />);
                await flush();
                await flush();
            });

            const latestValue = requireLatest(latest);

            // Assert
            expect(getInputDeviceList).toHaveBeenCalledTimes(1);
            expect(getOutputDeviceList).toHaveBeenCalledTimes(1);
            expect(latestValue.isLoading).toBe(false);
            expect(latestValue.error).toBeNull();
            expect(latestValue.inputs).toEqual(inputs);
            expect(latestValue.outputs).toEqual(outputs);

            await act(async () => { root.unmount(); });
        });

        it("exposes a refresh function that re-runs the fetch", async () => {
            // Arrange
            const firstInputs = [{id: "in-1", name: "Input 1", sample_rate: 44100}];
            const secondInputs = [{id: "in-2", name: "Input 2", sample_rate: 48000}];
            vi.mocked(getInputDeviceList)
                .mockResolvedValueOnce(firstInputs as any)
                .mockResolvedValueOnce(secondInputs as any);
            vi.mocked(getOutputDeviceList).mockResolvedValue([] as any);
            let latest: HookValue | null = null;

            await act(async () => {
                root.render(<Probe onChange={(v) => (latest = v)} />);
                await flush();
                await flush();
            });

            expect(requireLatest(latest).inputs).toEqual(firstInputs);

            // Act – refresh
            await act(async () => {
                await requireLatest(latest).refresh();
                await flush();
            });

            // Assert
            expect(requireLatest(latest).inputs).toEqual(secondInputs);
            expect(getInputDeviceList).toHaveBeenCalledTimes(2);

            await act(async () => { root.unmount(); });
        });
    });

    describe("failure_path", () => {
        it("sets error when one device request fails", async () => {
            // Arrange
            vi.mocked(getInputDeviceList).mockRejectedValueOnce(new Error("input failed"));
            vi.mocked(getOutputDeviceList).mockResolvedValueOnce([] as any);
            let latest: HookValue | null = null;

            // Act
            await act(async () => {
                root.render(<Probe onChange={(value) => (latest = value)} />);
                await flush();
                await flush();
            });

            const latestValue = requireLatest(latest);

            // Assert
            expect(latestValue.isLoading).toBe(false);
            expect(latestValue.error).toBe("input failed");

            await act(async () => { root.unmount(); });
        });

        it("uses String(err) for non-Error rejections", async () => {
            // Arrange
            vi.mocked(getInputDeviceList).mockRejectedValueOnce("device-not-found");
            vi.mocked(getOutputDeviceList).mockResolvedValueOnce([] as any);
            let latest: HookValue | null = null;

            // Act
            await act(async () => {
                root.render(<Probe onChange={(v) => (latest = v)} />);
                await flush();
                await flush();
            });

            // Assert – String("device-not-found") === "device-not-found"
            expect(requireLatest(latest).error).toBe("device-not-found");

            await act(async () => { root.unmount(); });
        });
    });
});
