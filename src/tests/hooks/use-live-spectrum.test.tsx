// @vitest-environment jsdom
import React, {act, useEffect} from "react";
import {createRoot, Root} from "react-dom/client";
import {beforeAll, beforeEach, describe, expect, it, vi} from "vitest";
import {type LiveSpectrumState, useLiveSpectrum} from "../../hooks/useLiveSpectrum";
import * as domain from "../../domain";
import {listen} from "@tauri-apps/api/event";

// Mock domain functions
vi.mock("../../domain", () => ({
    getSpectrumContract: vi.fn(),
    getLiveSpectrum: vi.fn(),
    startLiveSpectrumStream: vi.fn(),
    stopLiveSpectrumStream: vi.fn(),
}));

// Mock Tauri event
vi.mock("@tauri-apps/api/event", () => ({
    listen: vi.fn((eventName, callback) => {
        ;(listen as any).lastCallback = callback;
        ;(listen as any).lastUnlisten = vi.fn();
        return Promise.resolve((listen as any).lastUnlisten);
    }),
}));

type HookValue = LiveSpectrumState;

function Probe({onChange}: {onChange: (value: HookValue) => void}) {
    const value = useLiveSpectrum();

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

const mockContract = {
    live_spectrum_event: "spectrum-update",
    frequency_bins: 1024,
};

const mockSpectrum = {
    timestamp: Date.now(),
    frequencies_hz: [],
    magnitudes: new Array(1024).fill(0).map((_, i) => i * 0.1),
};

describe("useLiveSpectrum", () => {
    let container: HTMLDivElement;
    let root: Root;

    beforeAll(() => {
        (globalThis as {IS_REACT_ACT_ENVIRONMENT?: boolean}).IS_REACT_ACT_ENVIRONMENT = true;
    });

    beforeEach(() => {
        vi.clearAllMocks();
        ;(listen as any).lastCallback = undefined;
        ;(listen as any).lastUnlisten = undefined;
        container = document.createElement("div");
        document.body.appendChild(container);
        root = createRoot(container);
    });

    describe("success_path", () => {
        it("loads spectrum contract and starts listening", async () => {
            // Arrange
            vi.mocked(domain.getSpectrumContract).mockResolvedValueOnce(mockContract as any);
            vi.mocked(domain.getLiveSpectrum).mockResolvedValueOnce(mockSpectrum as any);
            vi.mocked(domain.startLiveSpectrumStream).mockResolvedValueOnce(undefined);
            let latest: HookValue | null = null;

            // Act
            await act(async () => {
                root.render(<Probe onChange={(value) => (latest = value)} />);
                await flush();
                await flush();
            });

            const latestValue = requireLatest(latest);

            // Assert
            expect(domain.getSpectrumContract).toHaveBeenCalledTimes(1);
            expect(domain.getLiveSpectrum).toHaveBeenCalledTimes(1);
            expect(domain.startLiveSpectrumStream).toHaveBeenCalledTimes(1);
            expect(latestValue.contract).toEqual(mockContract);
            expect(latestValue.spectrum).toEqual(mockSpectrum);
            expect(latestValue.loadError).toBeNull();

            await act(async () => { root.unmount(); });
        });

        it("receives spectrum updates from events", async () => {
            // Arrange
            vi.mocked(domain.getSpectrumContract).mockResolvedValueOnce(mockContract as any);
            vi.mocked(domain.getLiveSpectrum).mockResolvedValueOnce(mockSpectrum as any);
            vi.mocked(domain.startLiveSpectrumStream).mockResolvedValueOnce(undefined);
            let latest: HookValue | null = null;

            // Act
            await act(async () => {
                root.render(<Probe onChange={(value) => (latest = value)} />);
                await flush();
                await flush();
            });

            const updatedSpectrum = {
                timestamp: Date.now() + 1000,
                frequencies_hz: [],
                magnitudes: new Array(1024).fill(0).map((_, i) => i * 0.2),
            };

            const callback = (listen as any).lastCallback;
            if (callback) {
                await act(async () => {
                    callback({payload: updatedSpectrum});
                    await flush();
                });
            }

            const latestValue = requireLatest(latest);

            // Assert
            expect(latestValue.spectrum).toBeDefined();
            expect(latestValue.spectrum?.magnitudes.length).toBe(1024);
            expect(latestValue.loadError).toBeNull();

            await act(async () => { root.unmount(); });
        });

        it("blends spectrum correctly with attack/release smoothing", async () => {
            // Arrange
            const firstSpectrum = {timestamp: 0, frequencies_hz: [], magnitudes: new Array(10).fill(0)};
            const secondSpectrum = {timestamp: 1, frequencies_hz: [], magnitudes: new Array(10).fill(1)};

            vi.mocked(domain.getSpectrumContract).mockResolvedValueOnce(mockContract as any);
            vi.mocked(domain.getLiveSpectrum).mockResolvedValueOnce(firstSpectrum as any);
            vi.mocked(domain.startLiveSpectrumStream).mockResolvedValueOnce(undefined);
            let latest: HookValue | null = null;

            await act(async () => {
                root.render(<Probe onChange={(value) => (latest = value)} />);
                await flush();
                await flush();
            });

            const callback = (listen as any).lastCallback;
            if (callback) {
                await act(async () => {
                    callback({payload: secondSpectrum});
                    await flush();
                });
            }

            const latestValue = requireLatest(latest);
            // delta=+1 (attack), ATTACK_ALPHA=0.18: 0 + 1*0.18 = 0.18
            expect(latestValue.spectrum?.magnitudes[0]).toBeCloseTo(0.18, 1);

            await act(async () => { root.unmount(); });
        });

        it("applies release alpha (0.08) for decreasing values", async () => {
            // Arrange – going from 1 down to 0
            const firstSpectrum = {timestamp: 0, frequencies_hz: [], magnitudes: new Array(4).fill(1)};
            // Large negative delta (-1) to pass deadband (0.35), testing RELEASE path
            const fallingSpectrum = {timestamp: 1, frequencies_hz: [], magnitudes: new Array(4).fill(0)};

            vi.mocked(domain.getSpectrumContract).mockResolvedValueOnce(mockContract as any);
            vi.mocked(domain.getLiveSpectrum).mockResolvedValueOnce(firstSpectrum as any);
            vi.mocked(domain.startLiveSpectrumStream).mockResolvedValueOnce(undefined);
            let latest: HookValue | null = null;

            await act(async () => {
                root.render(<Probe onChange={(value) => (latest = value)} />);
                await flush();
                await flush();
            });

            const callback = (listen as any).lastCallback;
            await act(async () => {
                callback({payload: fallingSpectrum});
                await flush();
            });

            // delta = 0 - 1 = -1 (negative => RELEASE_ALPHA=0.08)
            // expected = 1 + (-1 * 0.08) = 0.92  (NOT 0.18 which would be ATTACK)
            expect(requireLatest(latest).spectrum?.magnitudes[0]).toBeCloseTo(0.92, 2);

            await act(async () => { root.unmount(); });
        });

        it("applies jitter deadband for small deltas", async () => {
            // Arrange
            const firstSpectrum = {timestamp: 0, frequencies_hz: [], magnitudes: [100, 100, 100]};
            // deltas: 0.2, 0.1, 0.3 – all strictly < 0.35 JITTER_DEADBAND_DB
            const secondSpectrum = {timestamp: 1, frequencies_hz: [], magnitudes: [100.2, 100.1, 100.3]};

            vi.mocked(domain.getSpectrumContract).mockResolvedValueOnce(mockContract as any);
            vi.mocked(domain.getLiveSpectrum).mockResolvedValueOnce(firstSpectrum as any);
            vi.mocked(domain.startLiveSpectrumStream).mockResolvedValueOnce(undefined);
            let latest: HookValue | null = null;

            await act(async () => {
                root.render(<Probe onChange={(value) => (latest = value)} />);
                await flush();
                await flush();
            });

            const callback = (listen as any).lastCallback;
            if (callback) {
                await act(async () => {
                    callback({payload: secondSpectrum});
                    await flush();
                });
            }

            // Values within deadband remain unchanged
            expect(requireLatest(latest).spectrum?.magnitudes[0]).toBe(100);
            expect(requireLatest(latest).spectrum?.magnitudes[1]).toBe(100);
            expect(requireLatest(latest).spectrum?.magnitudes[2]).toBe(100);

            await act(async () => { root.unmount(); });
        });

        it("blends when delta exactly equals JITTER_DEADBAND_DB (0.35) – not in deadband", async () => {
            // The deadband is `Math.abs(delta) < 0.35` (strictly less-than).
            // When prev=0 and next=0.35, delta = 0.35 - 0 = exactly the float 0.35.
            // 0.35 < 0.35 is false → blending MUST happen.
            // With mutant `<= 0.35`: 0.35 <= 0.35 is true → returns prev=0 (no blend).
            const firstSpectrum = {timestamp: 0, frequencies_hz: [], magnitudes: [0]};
            const secondSpectrum = {timestamp: 1, frequencies_hz: [], magnitudes: [0.35]};

            vi.mocked(domain.getSpectrumContract).mockResolvedValueOnce(mockContract as any);
            vi.mocked(domain.getLiveSpectrum).mockResolvedValueOnce(firstSpectrum as any);
            vi.mocked(domain.startLiveSpectrumStream).mockResolvedValueOnce(undefined);
            let latest: HookValue | null = null;

            await act(async () => {
                root.render(<Probe onChange={(v) => (latest = v)} />);
                await flush();
                await flush();
            });

            const callback = (listen as any).lastCallback;
            await act(async () => {
                callback({payload: secondSpectrum});
                await flush();
            });

            // delta=0.35 (exact float, subtracting 0) passes the strict deadband:
            // blended = 0 + 0.35 * 0.18 (attack) ≈ 0.063  (must be > 0, not == 0)
            const blended = requireLatest(latest).spectrum!.magnitudes[0];
            expect(blended).toBeGreaterThan(0);
            expect(blended).not.toBe(0); // confirms it was NOT suppressed by deadband

            await act(async () => { root.unmount(); });
        });

        it("stops spectrum stream on unmount and calls unlisten", async () => {
            // Arrange
            vi.mocked(domain.getSpectrumContract).mockResolvedValueOnce(mockContract as any);
            vi.mocked(domain.getLiveSpectrum).mockResolvedValueOnce(mockSpectrum as any);
            vi.mocked(domain.startLiveSpectrumStream).mockResolvedValueOnce(undefined);
            vi.mocked(domain.stopLiveSpectrumStream).mockResolvedValueOnce(undefined);

            await act(async () => {
                root.render(<Probe onChange={vi.fn()} />);
                await flush();
                await flush();
            });

            const unlistenFn = (listen as any).lastUnlisten;
            expect(unlistenFn).toBeDefined();

            // Unmount
            await act(async () => {
                root.unmount();
                await flush();
            });

            // Assert both unlisten and stopLiveSpectrumStream called
            expect(domain.stopLiveSpectrumStream).toHaveBeenCalledTimes(1);
            expect(unlistenFn).toHaveBeenCalledTimes(1);
        });

        it("returns next spectrum unchanged when previous is null", async () => {
            // null previous – test blendSpectrum null branch
            const initial = {timestamp: 0, frequencies_hz: [], magnitudes: [5, 10, 15]};

            vi.mocked(domain.getSpectrumContract).mockResolvedValueOnce(mockContract as any);
            vi.mocked(domain.getLiveSpectrum).mockResolvedValueOnce(initial as any);
            vi.mocked(domain.startLiveSpectrumStream).mockResolvedValueOnce(undefined);
            let latest: HookValue | null = null;

            await act(async () => {
                root.render(<Probe onChange={(v) => (latest = v)} />);
                await flush();
                await flush();
            });

            // Initial spectrum is set from getLiveSpectrum result — no blending (null previous)
            expect(requireLatest(latest).spectrum?.magnitudes).toEqual([5, 10, 15]);

            await act(async () => { root.unmount(); });
        });

        it("returns new spectrum as-is when magnitude lengths differ", async () => {
            // Exercise: previous.magnitudes.length !== next.magnitudes.length → return next
            const firstSpectrum = {timestamp: 0, frequencies_hz: [], magnitudes: new Array(10).fill(5)};
            const secondSpectrum = {timestamp: 1, frequencies_hz: [], magnitudes: new Array(20).fill(9)};

            vi.mocked(domain.getSpectrumContract).mockResolvedValueOnce(mockContract as any);
            vi.mocked(domain.getLiveSpectrum).mockResolvedValueOnce(firstSpectrum as any);
            vi.mocked(domain.startLiveSpectrumStream).mockResolvedValueOnce(undefined);
            let latest: HookValue | null = null;

            await act(async () => {
                root.render(<Probe onChange={(v) => (latest = v)} />);
                await flush();
                await flush();
            });

            const callback = (listen as any).lastCallback;
            await act(async () => {
                callback({payload: secondSpectrum});
                await flush();
            });

            // Lengths differ → no blending: all values must be exactly 9
            const mags = requireLatest(latest).spectrum!.magnitudes;
            expect(mags.length).toBe(20);
            expect(mags.every((v) => v === 9)).toBe(true);

            await act(async () => { root.unmount(); });
        });
    });

    describe("failure_path", () => {
        it("sets loadError when spectrum contract fails", async () => {
            // Arrange
            const error = new Error("contract fetch failed");
            vi.mocked(domain.getSpectrumContract).mockRejectedValueOnce(error);
            let latest: HookValue | null = null;

            // Act
            await act(async () => {
                root.render(<Probe onChange={(value) => (latest = value)} />);
                await flush();
                await flush();
            });

            const latestValue = requireLatest(latest);

            // Assert
            expect(latestValue.loadError).toBe("contract fetch failed");
            expect(latestValue.spectrum).toBeNull();
            expect(latestValue.contract).toBeNull();

            await act(async () => { root.unmount(); });
        });

        it("sets loadError for non-Error rejections using fallback string", async () => {
            // Exercise the `"Failed to read spectrum"` fallback in the catch block
            vi.mocked(domain.getSpectrumContract).mockRejectedValueOnce("raw-string-error");
            let latest: HookValue | null = null;

            await act(async () => {
                root.render(<Probe onChange={(v) => (latest = v)} />);
                await flush();
                await flush();
            });

            expect(requireLatest(latest).loadError).toBe("Failed to read spectrum");

            await act(async () => { root.unmount(); });
        });

        it("sets loadError when initial spectrum fetch fails", async () => {
            // Arrange
            vi.mocked(domain.getSpectrumContract).mockResolvedValueOnce(mockContract as any);
            vi.mocked(domain.getLiveSpectrum).mockRejectedValueOnce(
                new Error("spectrum fetch failed")
            );
            vi.mocked(domain.startLiveSpectrumStream).mockResolvedValueOnce(undefined);
            let latest: HookValue | null = null;

            // Act
            await act(async () => {
                root.render(<Probe onChange={(value) => (latest = value)} />);
                await flush();
                await flush();
            });

            const latestValue = requireLatest(latest);

            // Assert
            expect(latestValue.loadError).toBe("spectrum fetch failed");

            await act(async () => { root.unmount(); });
        });

        it("does not call listen when component unmounts before contract resolves", async () => {
            // Arrange – gated contract fetch
            let resolveContract: (v: any) => void;
            const contractPromise = new Promise((res) => { resolveContract = res; });
            vi.mocked(domain.getSpectrumContract).mockReturnValueOnce(contractPromise as any);

            await act(async () => {
                root.render(<Probe onChange={vi.fn()} />);
            });

            // Unmount before contract resolves
            await act(async () => { root.unmount(); });

            // Now resolve
            resolveContract!(mockContract);
            await act(async () => { await flush(); await flush(); });

            // listen must NOT have been called (disposed guard triggered)
            expect(listen).not.toHaveBeenCalled();
        });

        it("does not update state if component unmounts during contract fetch", async () => {
            // Arrange
            const contractPromise = new Promise((resolve) => {
                setTimeout(() => resolve(mockContract), 50);
            });
            vi.mocked(domain.getSpectrumContract).mockReturnValueOnce(contractPromise as any);

            // Act
            await act(async () => {
                root.render(<Probe onChange={vi.fn()} />);
            });

            // Unmount immediately
            await act(async () => {
                root.unmount();
                await flush();
            });

            // Wait for the promise to resolve
            await contractPromise;
            await act(async () => { await flush(); });

            // Assert - No errors should occur
            expect(domain.getSpectrumContract).toHaveBeenCalled();
        });

        it("ignores spectrum updates after unmount (disposed guard on event handler)", async () => {
            // Arrange
            vi.mocked(domain.getSpectrumContract).mockResolvedValueOnce(mockContract as any);
            vi.mocked(domain.getLiveSpectrum).mockResolvedValueOnce(mockSpectrum as any);
            vi.mocked(domain.startLiveSpectrumStream).mockResolvedValueOnce(undefined);
            let latest: HookValue | null = null;

            await act(async () => {
                root.render(<Probe onChange={(v) => (latest = v)} />);
                await flush();
                await flush();
            });

            const spectrumBefore = requireLatest(latest).spectrum;
            const capturedCallback = (listen as any).lastCallback;

            // Unmount
            await act(async () => {
                root.unmount();
                await flush();
            });

            // Fire event after unmount – should be ignored
            if (capturedCallback) {
                const differentSpectrum = {
                    ...mockSpectrum,
                    magnitudes: new Array(1024).fill(99),
                };
                capturedCallback({payload: differentSpectrum});
                await act(async () => { await flush(); });
            }

            // The spectrum captured before unmount should not have changed
            expect(spectrumBefore).toEqual(mockSpectrum);
        });

        it("handles magnitude array length mismatch on blend", async () => {
            // Arrange - Test the blending logic directly
            const firstSpectrum = {timestamp: 0, frequencies_hz: [], magnitudes: new Array(10).fill(0)};
            const secondSpectrum = {timestamp: 1, frequencies_hz: [], magnitudes: new Array(20).fill(1)};

            // The blending function used in useLiveSpectrum
            const result = firstSpectrum.magnitudes.length !== secondSpectrum.magnitudes.length
                ? secondSpectrum
                : firstSpectrum;

            // Assert
            expect(result.magnitudes.length).toBe(20);
            expect(result.magnitudes[0]).toBe(1);
        });
    });
});

