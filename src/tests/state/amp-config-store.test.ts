import {beforeEach, describe, expect, it, vi} from "vitest";
import * as domain from "../../domain";
import {AMP_ACTIVE_CHANGED_EVENT, useAmpStore} from "../../state/AmpConfigStore";

vi.mock("../../domain", () => ({
    addChannel: vi.fn(),
    addEffect: vi.fn(),
    applyEffectOrderChange: vi.fn(),
    getAmpConfig: vi.fn(),
    removeChannel: vi.fn(),
    removeEffect: vi.fn(),
    setBass: vi.fn(),
    setChannelId: vi.fn(),
    setGain: vi.fn(),
    setMasterVolume: vi.fn(),
    setMiddle: vi.fn(),
    setTreble: vi.fn(),
    setVolume: vi.fn(),
    toggleOnOff: vi.fn().mockResolvedValue(undefined),
}));

vi.mock("@tauri-apps/api/event", () => ({
    emit: vi.fn().mockResolvedValue(undefined),
}));

async function flushMicrotasks() {
    await Promise.resolve();
    await Promise.resolve();
}

const distortionEffect = {
    kind: "HCDistortion",
    data: {
        id: 10,
        name: "Drive",
        is_active: true,
        threshold: 0.4,
        level: 0.5,
    },
} as const;

const delayEffect = {
    kind: "Delay",
    data: {
        id: 11,
        name: "Delay",
        is_active: true,
        delay_time: 220,
        level: 0.4,
    },
} as const;

function resetStore() {
    useAmpStore.setState({
        master_volume: 1,
        is_active: false,
        current_channel: 0,
        chain_snapshot: null,
        channels: [
            {
                id: 0,
                name: "Clean",
                gain: 1,
                tone_stack: {bass: 0.5, middle: 0.5, treble: 0.5},
                volume: 1,
                effect_chain: [distortionEffect, delayEffect] as any,
            },
            {
                id: 1,
                name: "Lead",
                gain: 1.2,
                tone_stack: {bass: 0.6, middle: 0.6, treble: 0.6},
                volume: 0.8,
                effect_chain: [],
            },
        ],
    });
}

describe("AmpConfigStore", () => {
    beforeEach(() => {
        vi.clearAllMocks();
        resetStore();
    });

    describe("success_path", () => {
        it("init hydrates store from backend", async () => {
            // Arrange
            const backendConfig = {
                master_volume: 0.7,
                is_active: true,
                current_channel: 1,
                channels: [{id: 1, name: "Lead", gain: 1.1, tone_stack: {bass: 0.4, middle: 0.6, treble: 0.8}, volume: 0.9, effect_chain: []}],
            };
            vi.mocked(domain.getAmpConfig).mockResolvedValueOnce(backendConfig as any);

            // Act
            await useAmpStore.getState().init();

            // Assert
            expect(useAmpStore.getState().master_volume).toBe(0.7);
            expect(useAmpStore.getState().is_active).toBe(true);
            expect(useAmpStore.getState().current_channel).toBe(1);
        });

        it("setChannelById updates current channel and refreshes config", async () => {
            // Arrange
            const logSpy = vi.spyOn(console, "log").mockImplementation(() => undefined);
            const backendConfig = {
                master_volume: 0.9,
                is_active: true,
                current_channel: 1,
                channels: [{id: 1, name: "Lead", gain: 1.2, tone_stack: {bass: 0.7, middle: 0.6, treble: 0.8}, volume: 0.75, effect_chain: []}],
            };
            vi.mocked(domain.setChannelId).mockResolvedValueOnce(undefined);
            vi.mocked(domain.getAmpConfig).mockResolvedValueOnce(backendConfig as any);

            // Act
            await useAmpStore.getState().setChannelById(1);

            // Assert
            expect(domain.setChannelId).toHaveBeenCalledWith({channelId: 1});
            expect(domain.getAmpConfig).toHaveBeenCalledTimes(1);
            expect(useAmpStore.getState().current_channel).toBe(1);
            expect(useAmpStore.getState().master_volume).toBe(0.9);
            expect(logSpy).toHaveBeenCalledWith("Channel changed, store updated:", backendConfig);
            logSpy.mockRestore();
        });

        it("setChannelById applies optimistic local selection before backend resolves", async () => {
            // Arrange
            let continueBackend: (value?: void | PromiseLike<void>) => void = () => undefined;
            const backendGate = new Promise<void>((resolve) => {
                continueBackend = resolve;
            });
            vi.mocked(domain.setChannelId).mockImplementationOnce(() => backendGate);
            vi.mocked(domain.getAmpConfig).mockResolvedValueOnce({
                master_volume: 0.5,
                is_active: false,
                current_channel: 1,
                channels: useAmpStore.getState().channels,
            } as any);

            // Act
            const pending = useAmpStore.getState().setChannelById(1);

            // Assert
            expect(useAmpStore.getState().current_channel).toBe(1);
            continueBackend();
            await pending;
        });

        it("addChannel calls backend", async () => {
            // Arrange
            const logSpy = vi.spyOn(console, "log").mockImplementation(() => undefined);
            vi.mocked(domain.addChannel).mockResolvedValueOnce(undefined);

            // Act
            await useAmpStore.getState().addChannel("Crunch");

            // Assert
            expect(domain.addChannel).toHaveBeenCalledWith({channelName: "Crunch"});
            expect(logSpy).toHaveBeenCalledWith("Adding channel with name:", "Crunch");
            logSpy.mockRestore();
        });

        it("addChannelFromBackend appends new channel", async () => {
            // Arrange
            const dto = {id: 5, name: "Rhythm", gain: 1, tone_stack: {bass: 0.5, middle: 0.5, treble: 0.5}, volume: 1, effect_chain: []};

            // Act
            await useAmpStore.getState().addChannelFromBackend(dto as any);

            // Assert
            expect(useAmpStore.getState().channels.some((c) => c.id === 5)).toBe(true);
            expect(useAmpStore.getState().current_channel).toBe(5);
        });

        it("removeChannel refreshes config after backend call", async () => {
            // Arrange
            const logSpy = vi.spyOn(console, "log").mockImplementation(() => undefined);
            vi.mocked(domain.removeChannel).mockResolvedValueOnce(undefined);
            vi.mocked(domain.getAmpConfig).mockResolvedValueOnce({
                master_volume: 1,
                is_active: false,
                current_channel: 0,
                channels: [useAmpStore.getState().channels[0]],
            } as any);

            // Act
            await useAmpStore.getState().removeChannel(1);

            // Assert
            expect(domain.removeChannel).toHaveBeenCalledWith({channelId: 1});
            expect(useAmpStore.getState().channels).toHaveLength(1);
            expect(logSpy).toHaveBeenCalledWith("Removing channel:", 1);
            expect(logSpy).toHaveBeenCalledWith("Channel removed, store updated:", expect.any(Object));
            logSpy.mockRestore();
        });

        it("setMasterVolume updates local state and calls backend", () => {
            // Arrange
            const setMasterVolume = useAmpStore.getState().setMasterVolume;

            // Act
            setMasterVolume(0.65);

            // Assert
            expect(useAmpStore.getState().master_volume).toBe(0.65);
            expect(domain.setMasterVolume).toHaveBeenCalledWith({masterVolume: 0.65});
        });

        it("setIsActive updates local state and calls backend toggle", async () => {
            // Arrange
            const setIsActive = useAmpStore.getState().setIsActive;
            const {emit} = await import("@tauri-apps/api/event");

            // Act
            setIsActive(true);
            await flushMicrotasks();

            // Assert
            expect(useAmpStore.getState().is_active).toBe(true);
            expect(domain.toggleOnOff).toHaveBeenCalledWith({isOn: true});
            expect(emit).toHaveBeenCalledWith("amp-active-changed", true);
            expect(emit).toHaveBeenCalledWith(AMP_ACTIVE_CHANGED_EVENT, true);
        });

        it("setGain/setVolume/setBass/setMiddle/setTreble update only current channel and call backend", () => {
            // Arrange
            const beforeOther = useAmpStore.getState().channels[1];

            // Act
            useAmpStore.getState().setGain(1.4);
            useAmpStore.getState().setVolume(0.77);
            useAmpStore.getState().setBass(0.2);
            useAmpStore.getState().setMiddle(0.3);
            useAmpStore.getState().setTreble(0.4);

            // Assert
            const current = useAmpStore.getState().channels.find((c) => c.id === 0)!;
            const other = useAmpStore.getState().channels.find((c) => c.id === 1)!;
            expect(current.gain).toBe(1.4);
            expect(current.volume).toBe(0.77);
            expect(current.tone_stack).toEqual({bass: 0.2, middle: 0.3, treble: 0.4});
            expect(other).toEqual(beforeOther);
            expect(domain.setGain).toHaveBeenCalledWith({gain: 1.4});
            expect(domain.setVolume).toHaveBeenCalledWith({volume: 0.77});
            expect(domain.setBass).toHaveBeenCalledWith({bass: 0.2});
            expect(domain.setMiddle).toHaveBeenCalledWith({middle: 0.3});
            expect(domain.setTreble).toHaveBeenCalledWith({treble: 0.4});
        });

        it("updateEffectActiveState updates matching effect only", () => {
            // Arrange
            const effectId = 10;
            useAmpStore.setState({
                channels: [
                    useAmpStore.getState().channels[0],
                    {
                        ...useAmpStore.getState().channels[1],
                        effect_chain: [{...distortionEffect}],
                    } as any,
                ],
            });

            // Act
            useAmpStore.getState().updateEffectActiveState(effectId, false);

            // Assert
            const effects = useAmpStore.getState().channels[0].effect_chain;
            const target = effects.find((e: any) => e.data.id === effectId);
            expect(target?.data.is_active).toBe(false);
            const untouchedOtherChannel = (useAmpStore.getState().channels[1].effect_chain as any[])[0];
            expect(untouchedOtherChannel.data.is_active).toBe(true);
        });

        it("updateHcDistortionParams and updateDelayParams patch matching effects", () => {
            // Arrange
            const store = useAmpStore.getState();

            // Act
            store.updateHcDistortionParams(10, {threshold: 0.8, level: 0.9});
            store.updateDelayParams(11, {delay_time: 300, level: 0.2});

            // Assert
            const effects = useAmpStore.getState().channels[0].effect_chain as any[];
            expect(effects.find((e) => e.data.id === 10)?.data).toMatchObject({threshold: 0.8, level: 0.9});
            expect(effects.find((e) => e.data.id === 11)?.data).toMatchObject({delay_time: 300, level: 0.2});

            // Cross-check kind/channel guards
            useAmpStore.setState({
                channels: [
                    {
                        ...useAmpStore.getState().channels[0],
                        effect_chain: [
                            distortionEffect,
                            delayEffect,
                            {kind: "Delay", data: {...delayEffect.data, id: 10}} as any,
                            {kind: "HCDistortion", data: {...distortionEffect.data, id: 11}} as any,
                        ],
                    } as any,
                    {
                        ...useAmpStore.getState().channels[1],
                        effect_chain: [{kind: "HCDistortion", data: {...distortionEffect.data}} as any],
                    } as any,
                ],
            });
            useAmpStore.getState().updateHcDistortionParams(10, {threshold: 0.33});
            useAmpStore.getState().updateDelayParams(11, {delay_time: 111});

            const updatedChain = useAmpStore.getState().channels[0].effect_chain as any[];
            expect(updatedChain.find((e) => e.kind === "Delay" && e.data.id === 10)?.data.delay_time).toBe(220);
            expect(updatedChain.find((e) => e.kind === "HCDistortion" && e.data.id === 11)?.data.threshold).toBe(0.4);
            expect((useAmpStore.getState().channels[1].effect_chain as any[])[0].data.threshold).toBe(0.4);
        });

        it("removeEffect refreshes config after backend call", async () => {
            // Arrange
            const logSpy = vi.spyOn(console, "log").mockImplementation(() => undefined);
            vi.mocked(domain.removeEffect).mockResolvedValueOnce(undefined);
            vi.mocked(domain.getAmpConfig).mockResolvedValueOnce({
                ...useAmpStore.getState(),
                channels: [{...useAmpStore.getState().channels[0], effect_chain: [delayEffect]}],
            } as any);

            // Act
            await useAmpStore.getState().removeEffect(10);

            // Assert
            expect(domain.removeEffect).toHaveBeenCalledWith({effectId: 10});
            expect(useAmpStore.getState().channels[0].effect_chain).toHaveLength(1);
            expect(logSpy).toHaveBeenCalledWith("Removing effect:", 10);
            expect(logSpy).toHaveBeenCalledWith("Effect removed, store updated:", expect.any(Object));
            logSpy.mockRestore();
        });

        it("addEffect calls backend and refreshes config", async () => {
            // Arrange
            const logSpy = vi.spyOn(console, "log").mockImplementation(() => undefined);
            const newEffect = {
                kind: "Delay",
                data: {id: 99, name: "Echo", is_active: true, delay_time: 180, level: 0.5},
            } as any;
            vi.mocked(domain.addEffect).mockResolvedValueOnce(undefined);
            vi.mocked(domain.getAmpConfig).mockResolvedValueOnce({
                ...useAmpStore.getState(),
                channels: [{...useAmpStore.getState().channels[0], effect_chain: [...useAmpStore.getState().channels[0].effect_chain, newEffect]}],
            } as any);

            // Act
            await useAmpStore.getState().addEffect(newEffect);

            // Assert
            expect(domain.addEffect).toHaveBeenCalledWith({effectDto: newEffect});
            expect(useAmpStore.getState().channels[0].effect_chain).toHaveLength(3);
            expect(logSpy).toHaveBeenCalledWith("Adding Effect with name:", "Echo");
            expect(logSpy).toHaveBeenCalledWith("Effect added, store updated:", expect.any(Object));
            logSpy.mockRestore();
        });

        it("moveEffect reorders effects in current channel", async () => {
            // Arrange
            const logSpy = vi.spyOn(console, "log").mockImplementation(() => undefined);
            const before = useAmpStore.getState().channels[0].effect_chain as any[];
            expect(before[0].data.id).toBe(10);

            // Act
            await useAmpStore.getState().moveEffect(0, 1);

            // Assert
            const after = useAmpStore.getState().channels[0].effect_chain as any[];
            expect(after[0].data.id).toBe(11);
            expect(after[1].data.id).toBe(10);
            expect(logSpy).toHaveBeenCalledWith("Moving effect from 0 to 1");
            logSpy.mockRestore();
        });

        it("startEditingChainOrder and cancelEditingChainOrder restore snapshot", () => {
            // Arrange
            useAmpStore.getState().startEditingChainOrder();
            useAmpStore.getState().moveEffect(0, 1);

            // Act
            useAmpStore.getState().cancelEditingChainOrder();

            // Assert
            const restored = useAmpStore.getState().channels[0].effect_chain as any[];
            expect(restored[0].data.id).toBe(10);
            expect(useAmpStore.getState().chain_snapshot).toBeNull();
        });

        it("applyChangesToChainOrder calls backend and clears snapshot", async () => {
            // Arrange
            const logSpy = vi.spyOn(console, "log").mockImplementation(() => undefined);
            useAmpStore.setState({chain_snapshot: [...useAmpStore.getState().channels[0].effect_chain] as any});
            vi.mocked(domain.applyEffectOrderChange).mockResolvedValueOnce(undefined);

            // Act
            await useAmpStore.getState().applyChangesToChainOrder();

            // Assert
            expect(domain.applyEffectOrderChange).toHaveBeenCalledTimes(1);
            expect(domain.applyEffectOrderChange).toHaveBeenCalledWith({
                effects: useAmpStore.getState().channels[0].effect_chain,
            });
            expect(useAmpStore.getState().chain_snapshot).toBeNull();
            expect(logSpy).toHaveBeenCalledWith("Changing effect order");
            expect(logSpy).toHaveBeenCalledWith("Successfully changed effect order");
            logSpy.mockRestore();
        });

        it("moveEffect only reorders the active channel", async () => {
            // Arrange
            useAmpStore.setState({
                current_channel: 1,
                channels: [
                    {
                        ...useAmpStore.getState().channels[0],
                        effect_chain: [distortionEffect, delayEffect] as any,
                    },
                    {
                        ...useAmpStore.getState().channels[1],
                        effect_chain: [
                            {...delayEffect, data: {...delayEffect.data, id: 21}},
                            {...distortionEffect, data: {...distortionEffect.data, id: 22}},
                        ] as any,
                    },
                ],
            });

            // Act
            await useAmpStore.getState().moveEffect(0, 1);

            // Assert
            const firstChannelIds = (useAmpStore.getState().channels[0].effect_chain as any[]).map((e) => e.data.id);
            const secondChannelIds = (useAmpStore.getState().channels[1].effect_chain as any[]).map((e) => e.data.id);
            expect(firstChannelIds).toEqual([10, 11]);
            expect(secondChannelIds).toEqual([22, 21]);
        });

        it("moveEffect allows moving to index 0", async () => {
            // Arrange
            await useAmpStore.getState().moveEffect(0, 1);

            // Act
            await useAmpStore.getState().moveEffect(1, 0);

            // Assert
            const ids = (useAmpStore.getState().channels[0].effect_chain as any[]).map((e) => e.data.id);
            expect(ids).toEqual([10, 11]);
        });

        it("moveEffect ignores indices that are exactly the chain length", async () => {
            const before = structuredClone(useAmpStore.getState().channels[0].effect_chain as any[]);
            const length = before.length;

            await useAmpStore.getState().moveEffect(length, 0);
            await useAmpStore.getState().moveEffect(0, length);

            expect(useAmpStore.getState().channels[0].effect_chain).toEqual(before);
        });

        it("exports expected amp-active event name", () => {
            expect(AMP_ACTIVE_CHANGED_EVENT).toBe("amp-active-changed");
        });
    });

    describe("failure_path", () => {
        it("init handles backend failure without mutating state", async () => {
            // Arrange
            const baseline = useAmpStore.getState().is_active;
            const expectedError = new Error("Backend unavailable");
            vi.mocked(domain.getAmpConfig).mockRejectedValueOnce(expectedError);
            const consoleErrorSpy = vi.spyOn(console, "error").mockImplementation(() => undefined);

            // Act
            await useAmpStore.getState().init();

            // Assert
            expect(consoleErrorSpy).toHaveBeenCalledWith("Failed to fetch init state from Rust:", expectedError);
            expect(useAmpStore.getState().is_active).toBe(baseline);
            consoleErrorSpy.mockRestore();
        });

        it("setChannelById logs error when backend setChannelId fails", async () => {
            // Arrange
            const expectedError = new Error("set channel failed");
            vi.mocked(domain.setChannelId).mockRejectedValueOnce(expectedError);
            const consoleErrorSpy = vi.spyOn(console, "error").mockImplementation(() => undefined);

            // Act
            await useAmpStore.getState().setChannelById(1);

            // Assert
            expect(consoleErrorSpy).toHaveBeenCalledWith("Failed to set channel index:", expectedError);
            consoleErrorSpy.mockRestore();
        });

        it("addChannel/removeChannel/removeEffect/addEffect handle backend failures", async () => {
            // Arrange
            const err = new Error("request failed");
            vi.mocked(domain.addChannel).mockRejectedValueOnce(err);
            vi.mocked(domain.removeChannel).mockRejectedValueOnce(err);
            vi.mocked(domain.removeEffect).mockRejectedValueOnce(err);
            vi.mocked(domain.addEffect).mockRejectedValueOnce(err);
            const consoleErrorSpy = vi.spyOn(console, "error").mockImplementation(() => undefined);

            // Act
            await useAmpStore.getState().addChannel("Bad");
            await useAmpStore.getState().removeChannel(9);
            await useAmpStore.getState().removeEffect(10);
            await useAmpStore.getState().addEffect(delayEffect as any);

            // Assert
            expect(consoleErrorSpy).toHaveBeenCalled();
            consoleErrorSpy.mockRestore();
        });

        it("logs exact error message when addChannel fails", async () => {
            const err = new Error("request failed");
            vi.mocked(domain.addChannel).mockRejectedValueOnce(err);
            const errorSpy = vi.spyOn(console, "error").mockImplementation(() => undefined);

            await useAmpStore.getState().addChannel("Bad");

            expect(errorSpy).toHaveBeenCalledWith("Failed to add channel:", err);
            errorSpy.mockRestore();
        });

        it("setIsActive rolls back local state if backend toggle fails", async () => {
            // Arrange
            useAmpStore.setState({is_active: false});
            const err = new Error("toggle failed");
            vi.mocked(domain.toggleOnOff).mockRejectedValueOnce(err);
            const errorSpy = vi.spyOn(console, "error").mockImplementation(() => undefined);

            // Act
            useAmpStore.getState().setIsActive(true);
            await flushMicrotasks();

            // Assert
            expect(useAmpStore.getState().is_active).toBe(false);
            expect(errorSpy).toHaveBeenCalledWith("Failed to toggle amp on/off, rolling back:", err);
            errorSpy.mockRestore();
        });

        it("addChannelFromBackend replaces existing channel instead of duplicating", async () => {
            // Arrange
            const existing = useAmpStore.getState().channels[0];
            const updated = {...existing, name: "Renamed"};

            // Act
            await useAmpStore.getState().addChannelFromBackend(updated as any);

            // Assert
            const channels = useAmpStore.getState().channels;
            expect(channels.filter((c) => c.id === existing.id)).toHaveLength(1);
            expect(channels.find((c) => c.id === existing.id)?.name).toBe("Renamed");
        });

        it("updateEffectActiveState and param updates ignore unknown ids", () => {
            // Arrange
            const before = structuredClone(useAmpStore.getState().channels[0].effect_chain as any[]);

            // Act
            useAmpStore.getState().updateEffectActiveState(999, false);
            useAmpStore.getState().updateHcDistortionParams(999, {threshold: 0.1});
            useAmpStore.getState().updateDelayParams(999, {delay_time: 999});

            // Assert
            expect(useAmpStore.getState().channels[0].effect_chain).toEqual(before);
        });

        it("updateHcDistortionParams/updateDelayParams do not update other kinds or channels", () => {
            useAmpStore.setState({
                channels: [
                    {
                        ...useAmpStore.getState().channels[0],
                        effect_chain: [
                            {kind: "HCDistortion", data: {...distortionEffect.data, id: 1}} as any,
                            {kind: "Delay", data: {...delayEffect.data, id: 1}} as any,
                            {kind: "Delay", data: {...delayEffect.data, id: 2}} as any,
                        ],
                    },
                    {
                        ...useAmpStore.getState().channels[1],
                        effect_chain: [{kind: "Delay", data: {...delayEffect.data, id: 1}} as any],
                    },
                ] as any,
                current_channel: 0,
            });

            useAmpStore.getState().updateHcDistortionParams(1, {threshold: 0.99});
            useAmpStore.getState().updateDelayParams(1, {delay_time: 999});

            const channel0 = useAmpStore.getState().channels[0].effect_chain as any[];
            const channel1 = useAmpStore.getState().channels[1].effect_chain as any[];

            const c0Distortion = channel0.find((e) => e.kind === "HCDistortion" && e.data.id === 1);
            const c0DelaySameId = channel0.find((e) => e.kind === "Delay" && e.data.id === 1);
            const c0DelayOtherId = channel0.find((e) => e.kind === "Delay" && e.data.id === 2);

            expect(c0Distortion.data.threshold).toBe(0.99);
            expect(c0Distortion.data.delay_time).toBeUndefined();
            expect(c0DelaySameId.data.threshold).toBeUndefined();
            expect(c0DelaySameId.data.delay_time).toBe(999);
            expect(c0DelayOtherId.data.delay_time).toBe(220);
            expect(channel1[0].data.delay_time).toBe(220);
        });

        it("moveEffect does nothing for out-of-range indices", async () => {
            // Arrange
            useAmpStore.setState({
                channels: [{
                    ...useAmpStore.getState().channels[0],
                    effect_chain: [
                        distortionEffect,
                        delayEffect,
                        {kind: "Delay", data: {...delayEffect.data, id: 12}} as any,
                    ],
                }, useAmpStore.getState().channels[1]],
            });
            const before = [...(useAmpStore.getState().channels[0].effect_chain as any[])];

            // Act
            await useAmpStore.getState().moveEffect(-1, 0);
            await useAmpStore.getState().moveEffect(0, 99);
            await useAmpStore.getState().moveEffect(0, -1);

            // Assert
            expect(useAmpStore.getState().channels[0].effect_chain).toEqual(before);
        });

        it("moveEffect does nothing when active channel does not exist", async () => {
            useAmpStore.setState({current_channel: 999});
            const before = structuredClone(useAmpStore.getState().channels);

            await useAmpStore.getState().moveEffect(0, 1);

            expect(useAmpStore.getState().channels).toEqual(before);
        });

        it("startEditingChainOrder logs warning if current channel is missing", () => {
            // Arrange
            useAmpStore.setState({current_channel: 999});
            const warnSpy = vi.spyOn(console, "warn").mockImplementation(() => undefined);

            // Act
            useAmpStore.getState().startEditingChainOrder();

            // Assert
            expect(warnSpy).toHaveBeenCalledWith("Could not find current channel to snapshot.");
            warnSpy.mockRestore();
        });

        it("applyChangesToChainOrder returns early when current channel is missing", async () => {
            // Arrange
            useAmpStore.setState({current_channel: 999});
            const errorSpy = vi.spyOn(console, "error").mockImplementation(() => undefined);

            // Act
            await useAmpStore.getState().applyChangesToChainOrder();

            // Assert
            expect(domain.applyEffectOrderChange).not.toHaveBeenCalled();
            expect(errorSpy).toHaveBeenCalledWith("No active channel found to apply order changes.");
            errorSpy.mockRestore();
        });

        it("applyChangesToChainOrder handles backend failure", async () => {
            // Arrange
            const err = new Error("persist order failed");
            vi.mocked(domain.applyEffectOrderChange).mockRejectedValueOnce(err);
            const errorSpy = vi.spyOn(console, "error").mockImplementation(() => undefined);
            const logSpy = vi.spyOn(console, "log").mockImplementation(() => undefined);

            // Act
            await useAmpStore.getState().applyChangesToChainOrder();

            // Assert
            expect(logSpy).toHaveBeenCalledWith("Changing effect order");
            expect(errorSpy).toHaveBeenCalledWith("Failed to change Effect order:", err);
            logSpy.mockRestore();
            errorSpy.mockRestore();
        });

        it("starts from expected default state when module is freshly imported", async () => {
            vi.resetModules();
            const fresh = await import("../../state/AmpConfigStore");
            const state = fresh.useAmpStore.getState();

            expect(state.is_active).toBe(false);
            expect(state.current_channel).toBe(0);
            expect(state.channels[0].name).toBe("Default");
            expect(state.channels[0].tone_stack).toEqual({bass: 1, middle: 1, treble: 1});
            expect(state.channels[0].effect_chain).toEqual([]);
        });

        it("removeChannel/removeEffect/addEffect log exact backend failure messages", async () => {
            const err = new Error("request failed");
            vi.mocked(domain.removeChannel).mockRejectedValueOnce(err);
            vi.mocked(domain.removeEffect).mockRejectedValueOnce(err);
            vi.mocked(domain.addEffect).mockRejectedValueOnce(err);
            const errorSpy = vi.spyOn(console, "error").mockImplementation(() => undefined);

            await useAmpStore.getState().removeChannel(9);
            await useAmpStore.getState().removeEffect(10);
            await useAmpStore.getState().addEffect(delayEffect as any);

            expect(errorSpy).toHaveBeenCalledWith("Failed to remove channel:", err);
            expect(errorSpy).toHaveBeenCalledWith("Failed to remove Effect:", err);
            expect(errorSpy).toHaveBeenCalledWith("Failed to add Effect:", err);
            errorSpy.mockRestore();
        });

        it("cancelEditingChainOrder only restores the active channel", () => {
            useAmpStore.setState({
                channels: [
                    {
                        ...useAmpStore.getState().channels[0],
                        effect_chain: [distortionEffect, delayEffect] as any,
                    },
                    {
                        ...useAmpStore.getState().channels[1],
                        effect_chain: [
                            {...delayEffect, data: {...delayEffect.data, id: 30}},
                            {...distortionEffect, data: {...distortionEffect.data, id: 31}},
                        ] as any,
                    },
                ],
                current_channel: 0,
                chain_snapshot: [delayEffect, distortionEffect] as any,
            });

            useAmpStore.getState().cancelEditingChainOrder();

            const firstIds = (useAmpStore.getState().channels[0].effect_chain as any[]).map((e) => e.data.id);
            const secondIds = (useAmpStore.getState().channels[1].effect_chain as any[]).map((e) => e.data.id);
            expect(firstIds).toEqual([11, 10]);
            expect(secondIds).toEqual([30, 31]);
        });
    });
});
