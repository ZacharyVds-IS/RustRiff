import {
    addChannel,
    addEffect,
    AmpConfigDto,
    applyEffectOrderChange,
    ChannelDto,
    EffectDto,
    getAmpConfig,
    HcDistortionDto,
    removeChannel,
    removeEffect,
    setBass,
    setChannelId,
    setGain,
    setMasterVolume,
    setMiddle,
    setTreble,
    setVolume,
    toggleOnOff
} from "../domain";
import {create} from "zustand/react";

interface AmpState extends AmpConfigDto {
    init: () => Promise<void>;
    setChannelById: (index: number) => Promise<void>;
    addChannel: (channelName: string) => Promise<void>;
    addChannelFromBackend: (channelDto: ChannelDto) => Promise<void>;
    removeChannel: (channelId: number) => void;
    setGain: (val: number) => void;
    setVolume: (val: number) => void;
    setMasterVolume: (val: number) => void;
    setIsActive: (val: boolean) => void;
    setBass: (val: number) => void;
    setMiddle: (val: number) => void;
    setTreble: (val: number) => void;
    updateEffectActiveState: (effectId: number, isActive: boolean) => void;
    updateHcDistortionParams: (effectId: number, patch: Partial<Pick<HcDistortionDto, "threshold" | "level">>) => void;
    removeEffect: (effectId: number) => void;
    addEffect: (effectDto: EffectDto) => Promise<void>;
    moveEffect: (effectId: number, newIndex: number) => Promise<void>;
    chain_snapshot: EffectDto[] | null;
    startEditingChainOrder: () => void;
    cancelEditingChainOrder: () => void;
    applyChangesToChainOrder: () => Promise<void>;
}

export const useAmpStore = create<AmpState>((set, get) => ({
        master_volume: 1,
        is_active: false,
        channels: [{
            id: 0,
            name: "Default",
            gain: 1.0,
            tone_stack: {
                bass: 1.0,
                middle: 1.0,
                treble: 1.0,
            },
            volume: 1,
            effect_chain: [],
        }],
        current_channel: 0,
        chain_snapshot: null,

        init: async () => {
            try {
                const config = await getAmpConfig();
                set({
                    ...config
                });
                console.log("Store hydrated from Rust:", config);
            } catch (error) {
                console.error("Failed to fetch init state from Rust:", error);
            }
        },

        setChannelById: async (id: number) => {
            try {
                set({current_channel: id});

                await setChannelId({channelId: id});

                const config = await getAmpConfig();
                set({...config});

                console.log("Channel changed, store updated:", config);
            } catch (error) {
                console.error("Failed to set channel index:", error);
            }
        },

        addChannel: async (channelName: string) => {
            try {
                console.log("Adding channel with name:", channelName);
                await addChannel({channelName});
            } catch (error) {
                console.error("Failed to add channel:", error);
            }
        },

        addChannelFromBackend: async (channelDto: ChannelDto) => {
            set((state) => {
                const exists = state.channels.some(
                    (c) => c.id === channelDto.id
                );

                if (exists) {
                    return {
                        channels: state.channels.map((channel) =>
                            channel.id === channelDto.id ? channelDto : channel
                        ),
                        current_channel: channelDto.id,
                    };
                }

                return {
                    channels: [...state.channels, channelDto],
                    current_channel: channelDto.id,
                };
            });
        },

        removeChannel: async (channelId: number) => {
            try {
                console.log("Removing channel:", channelId);

                await removeChannel({channelId});

                const config = await getAmpConfig();
                set({...config});

                console.log("Channel removed, store updated:", config);
            } catch (error) {
                console.error("Failed to remove channel:", error);
            }
        },

        setMasterVolume: (val: number) => {
            set({master_volume: val});
            setMasterVolume({masterVolume: val})
        },

        setIsActive: (val: boolean) => {
            set({is_active: val});
            toggleOnOff({isOn: val});
        },

        setGain: (val: number) => {
            setGain({gain: val});

            set((state) => ({
                channels: state.channels.map((c) =>
                    c.id === state.current_channel
                        ? {...c, gain: val}
                        : c
                ),
            }));
        },

        setVolume: (val: number) => {
            setVolume({volume: val});

            set((state) => ({
                channels: state.channels.map((c) =>
                    c.id === state.current_channel
                        ? {...c, volume: val}
                        : c
                ),
            }));
        },

        setBass: (val: number) => {
            setBass({bass: val});

            set((state) => ({
                channels: state.channels.map((c) =>
                    c.id === state.current_channel
                        ? {
                            ...c,
                            tone_stack: {
                                ...c.tone_stack,
                                bass: val,
                            },
                        }
                        : c
                ),
            }));
        },


        setMiddle: (val: number) => {
            setMiddle({middle: val});

            set((state) => ({
                channels: state.channels.map((c) =>
                    c.id === state.current_channel
                        ? {
                            ...c,
                            tone_stack: {
                                ...c.tone_stack,
                                middle: val,
                            },
                        }
                        : c
                ),
            }));
        },

        setTreble: (val: number) => {
            setTreble({treble: val});

            set((state) => ({
                channels: state.channels.map((c) =>
                    c.id === state.current_channel
                        ? {
                            ...c,
                            tone_stack: {
                                ...c.tone_stack,
                                treble: val,
                            },
                        }
                        : c
                ),
            }));
        },

        updateEffectActiveState: (effectId: number, isActive: boolean) => {
            set((state) => ({
                channels: state.channels.map((c) =>
                    c.id === state.current_channel
                        ? {
                            ...c,
                            effect_chain: c.effect_chain.map((effect) => {
                                if (effect.data.id === effectId) {
                                    if (effect.kind === "Cabinet") {
                                        return {
                                            ...effect,
                                            data: {...effect.data, is_active: isActive} as typeof effect.data,
                                        };
                                    } else if (effect.kind === "HCDistortion") {
                                        return {
                                            ...effect,
                                            data: {...effect.data, is_active: isActive} as typeof effect.data,
                                        };
                                    }
                                }
                                return effect;
                            }),
                        }
                        : c
                ),
            }));
        },

        updateHcDistortionParams: (effectId, patch) => {
            set((state) => ({
                channels: state.channels.map((c) =>
                    c.id === state.current_channel
                        ? {
                            ...c,
                            effect_chain: c.effect_chain.map((effect) =>
                                effect.kind === "HCDistortion" && effect.data.id === effectId
                                    ? {
                                        ...effect,
                                        data: {
                                            ...effect.data,
                                            ...patch,
                                        },
                                    }
                                    : effect
                            ),
                        }
                        : c
                ),
            }));
        },

        removeEffect: async (effectId: number) => {
            try {
                console.log("Removing effect:", effectId);

                await removeEffect({effectId: effectId});

                const config = await getAmpConfig();
                set({...config});

                console.log("Effect removed, store updated:", config);
            } catch (error) {
                console.error("Failed to remove Effect:", error);
            }
        },

        addEffect: async (effectDto: EffectDto) => {
            try {
                console.log("Adding Effect with name:", effectDto.data.name);
                await addEffect({effectDto: effectDto});
                const config = await getAmpConfig();
                set({...config});
                console.log("Effect added, store updated:", config);
            } catch (error) {
                console.error("Failed to add Effect:", error);
            }
        },
        moveEffect: async (effectId: number, newIndex: number) => {
            set((state) => {
                const channelIndex = state.channels.findIndex(c => c.id === state.current_channel);
                if (channelIndex === -1) return state;

                const currentChannel = state.channels[channelIndex];
                const effectChain = currentChannel.effect_chain;

                const currentIndex = effectChain.findIndex(e => e.data.id === effectId);
                if (currentIndex === -1) return state;

                console.log(`Moving effect ${effectId} from ${currentIndex} to ${newIndex}`);

                const updatedChain = [...effectChain];
                const [movedItem] = updatedChain.splice(currentIndex, 1);
                updatedChain.splice(newIndex, 0, movedItem);

                return {
                    channels: state.channels.map((c, idx) =>
                        idx === channelIndex
                            ? {...c, effect_chain: updatedChain}
                            : c
                    )
                };
            });
        },
        startEditingChainOrder: () => {
            set((state) => {
                const currentChannel = state.channels.find(c => c.id === state.current_channel);

                if (!currentChannel) {
                    console.warn("Could not find current channel to snapshot.");
                    return state;
                }

                return {
                    chain_snapshot: [...currentChannel.effect_chain]
                };
            });
        },

        cancelEditingChainOrder: () => {
            // Restore the chain from the snapshot
            set((state) => ({
                channels: state.channels.map((c) =>
                    c.id === state.current_channel
                        ? {...c, effect_chain: state.chain_snapshot!}
                        : c
                ),
                chain_snapshot: null,
            }));
        },

        applyChangesToChainOrder: async () => {
            const state = get();

            const currentChannel = state.channels.find(c => c.id === state.current_channel);

            if (!currentChannel) {
                console.error("No active channel found to apply order changes.");
                return;
            }

            const currentEffects = currentChannel.effect_chain;
            try {
                console.log("Changing effect order", );
                await applyEffectOrderChange({effects: currentEffects});
                console.log("Successfully changed effect order", );
                set({chain_snapshot: null});
            } catch (error) {
                console.error("Failed to change Effect order:", error);
            }

        },
    }))
;