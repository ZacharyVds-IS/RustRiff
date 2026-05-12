// @vitest-environment jsdom
import React, {act, useEffect} from "react";
import {createRoot, Root} from "react-dom/client";
import {beforeAll, beforeEach, describe, expect, it, vi} from "vitest";
import {getAllChannels} from "../../domain";
import {useChannels} from "../../hooks/useChannels";

vi.mock("../../domain", () => ({
    getAllChannels: vi.fn(),
}));

type HookValue = ReturnType<typeof useChannels>;

function Probe({onChange, onFirstRender}: {
    onChange: (value: HookValue) => void;
    onFirstRender?: (value: HookValue) => void;
}) {
    const value = useChannels();
    const capturedFirst = React.useRef(false);

    // Capture value synchronously during first render BEFORE any effects fire
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

describe("useChannels", () => {
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
        it("starts with loading=true and empty channels before fetch resolves", async () => {
            // Arrange – gate the fetch so we can observe the initial state
            let resolve: (v: any) => void;
            vi.mocked(getAllChannels).mockReturnValueOnce(new Promise((res) => { resolve = res; }));
            let latest: HookValue | null = null;
            let firstRender: HookValue | null = null;

            // Act – render without flushing; onFirstRender captures state BEFORE effects fire
            await act(async () => {
                root.render(<Probe
                    onChange={(v) => (latest = v)}
                    onFirstRender={(v) => (firstRender = v)}
                />);
            });

            // Assert initial state from first synchronous render
            const initial = requireLatest(firstRender);
            expect(initial.loading).toBe(true);
            expect(initial.channels).toEqual([]);
            expect(initial.error).toBeNull();

            // Cleanup
            resolve!([]);
            await act(async () => { await flush(); await flush(); });
            await act(async () => { root.unmount(); });
        });

        it("sets loading=true at the start of the fetch", async () => {
            // Arrange – gated promise to observe loading=true mid-fetch
            let resolve: (v: any) => void;
            vi.mocked(getAllChannels).mockReturnValueOnce(new Promise((res) => { resolve = res; }));
            let latest: HookValue | null = null;

            await act(async () => {
                root.render(<Probe onChange={(v) => (latest = v)} />);
                // Allow microtask tick so sync state update fires but not the awaited promise
                await Promise.resolve();
            });

            // Assert loading is true during in-flight fetch
            expect(requireLatest(latest).loading).toBe(true);

            resolve!([]);
            await act(async () => { await flush(); await flush(); });
            await act(async () => { root.unmount(); });
        });

        it("loads channels successfully", async () => {
            // Arrange
            const data = [{id: 1, name: "Lead", gain: 1, tone_stack: {bass: 0.5, middle: 0.5, treble: 0.5}, volume: 1, effect_chain: []}];
            vi.mocked(getAllChannels).mockResolvedValueOnce(data as any);
            let latest: HookValue | null = null;

            // Act
            await act(async () => {
                root.render(<Probe onChange={(value) => (latest = value)} />);
                await flush();
                await flush();
            });

            const latestValue = requireLatest(latest);

            // Assert
            expect(getAllChannels).toHaveBeenCalledTimes(1);
            expect(latestValue.loading).toBe(false);
            expect(latestValue.error).toBeNull();
            expect(latestValue.channels).toEqual(data);

            await act(async () => { root.unmount(); });
        });

        it("logs fetch start and success messages with exact text", async () => {
            // Arrange
            const data = [{id: 1, name: "Lead", gain: 1, tone_stack: {bass: 0.5, middle: 0.5, treble: 0.5}, volume: 1, effect_chain: []}];
            vi.mocked(getAllChannels).mockResolvedValueOnce(data as any);
            const logSpy = vi.spyOn(console, "log").mockImplementation(() => undefined);

            // Act
            await act(async () => {
                root.render(<Probe onChange={vi.fn()} />);
                await flush();
                await flush();
            });

            // Assert exact log messages
            expect(logSpy).toHaveBeenCalledWith("useChannels: Fetching channels...");
            expect(logSpy).toHaveBeenCalledWith("useChannels: Channels fetched successfully:", data);

            logSpy.mockRestore();
            await act(async () => { root.unmount(); });
        });

        it("warns when backend returns an empty channel list", async () => {
            // Arrange
            vi.mocked(getAllChannels).mockResolvedValueOnce([] as any);
            const warnSpy = vi.spyOn(console, "warn").mockImplementation(() => undefined);
            const logSpy = vi.spyOn(console, "log").mockImplementation(() => undefined);
            let latest: HookValue | null = null;

            // Act
            await act(async () => {
                root.render(<Probe onChange={(v) => (latest = v)} />);
                await flush();
                await flush();
            });

            // Assert
            expect(requireLatest(latest).channels).toEqual([]);
            expect(warnSpy).toHaveBeenCalledWith("useChannels: No channels returned from backend");

            warnSpy.mockRestore();
            logSpy.mockRestore();
            await act(async () => { root.unmount(); });
        });

        it("does not warn when backend returns non-empty channel list", async () => {
            // Arrange
            const data = [{id: 1, name: "Lead", gain: 1, tone_stack: {bass: 0.5, middle: 0.5, treble: 0.5}, volume: 1, effect_chain: []}];
            vi.mocked(getAllChannels).mockResolvedValueOnce(data as any);
            const warnSpy = vi.spyOn(console, "warn").mockImplementation(() => undefined);
            const logSpy = vi.spyOn(console, "log").mockImplementation(() => undefined);

            // Act
            await act(async () => {
                root.render(<Probe onChange={vi.fn()} />);
                await flush();
                await flush();
            });

            // Assert – warn must NOT have been called
            expect(warnSpy).not.toHaveBeenCalled();

            warnSpy.mockRestore();
            logSpy.mockRestore();
            await act(async () => { root.unmount(); });
        });
    });

    describe("failure_path", () => {
        it("sets error when backend request fails", async () => {
            // Arrange
            vi.mocked(getAllChannels).mockRejectedValueOnce(new Error("channels failed"));
            let latest: HookValue | null = null;

            // Act
            await act(async () => {
                root.render(<Probe onChange={(value) => (latest = value)} />);
                await flush();
                await flush();
            });

            const latestValue = requireLatest(latest);

            // Assert
            expect(latestValue.loading).toBe(false);
            expect(latestValue.channels).toEqual([]);
            expect(latestValue.error).toBe("channels failed");

            await act(async () => { root.unmount(); });
        });

        it("logs exact error message when backend request fails", async () => {
            // Arrange
            const err = new Error("channels failed");
            vi.mocked(getAllChannels).mockRejectedValueOnce(err);
            const logSpy = vi.spyOn(console, "log").mockImplementation(() => undefined);
            const errorSpy = vi.spyOn(console, "error").mockImplementation(() => undefined);

            // Act
            await act(async () => {
                root.render(<Probe onChange={vi.fn()} />);
                await flush();
                await flush();
            });

            // Assert exact error log
            expect(errorSpy).toHaveBeenCalledWith("useChannels: Failed to fetch channels:", err);

            logSpy.mockRestore();
            errorSpy.mockRestore();
            await act(async () => { root.unmount(); });
        });

        it("uses String(err) for non-Error failures", async () => {
            // Arrange
            vi.mocked(getAllChannels).mockRejectedValueOnce("non-error-rejection");
            const logSpy = vi.spyOn(console, "log").mockImplementation(() => undefined);
            const errorSpy = vi.spyOn(console, "error").mockImplementation(() => undefined);
            let latest: HookValue | null = null;

            // Act
            await act(async () => {
                root.render(<Probe onChange={(v) => (latest = v)} />);
                await flush();
                await flush();
            });

            // String("non-error-rejection") === "non-error-rejection"
            expect(requireLatest(latest).error).toBe("non-error-rejection");

            logSpy.mockRestore();
            errorSpy.mockRestore();
            await act(async () => { root.unmount(); });
        });
    });
});
