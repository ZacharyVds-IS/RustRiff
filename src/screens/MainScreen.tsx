import {Box} from "@mui/material";
import {EffectChain} from "../components/EffectChain.tsx";
import {DefaultAmpControls} from "../components/DefaultAmpControls.tsx";
import {EffectPedal} from "../components/EffectPedal.tsx";
import {CabinetEffect} from "../components/CabinetEffect.tsx";
import {KeybindsDialog} from "../components/dialogs/KeybindsDialog.tsx";
import {useAmpStore} from "../state/AmpConfigStore.tsx";
import {useEffect, useState} from "react";
import {EffectDto, toggleEffect} from "../domain";
import {useHotkeys} from "react-hotkeys-hook";

type EffectSelection =
    | "amp"
    | {
        kind: EffectDto["kind"];
        effectId: string;
    };

export function MainScreen() {
    const activeChannel = useAmpStore((state) =>
        state.channels.find((c) => c.id === state.current_channel)
    );
    const currentChannelId = useAmpStore((state) => state.current_channel);
    const updateEffectActiveState = useAmpStore((state) => state.updateEffectActiveState);
    const moveEffect = useAmpStore((state) => state.moveEffect);
    const applyChangesToChainOrder = useAmpStore((state) => state.applyChangesToChainOrder);
    const isActive = useAmpStore((state) => state.is_active);
    const setIsActive = useAmpStore((state) => state.setIsActive);
    const [selection, setSelection] = useState<EffectSelection>("amp");
    const [isKeybindsOpen, setIsKeybindsOpen] = useState(false);
    useEffect(() => {
        setSelection("amp");
    }, [currentChannelId]);

    const resolvedSelection: EffectDto | "amp" | undefined =
        selection === "amp"
            ? "amp"
            : activeChannel?.effect_chain.find(
                (effect) =>
                    effect.kind === selection.kind &&
                    effect.data.id === selection.effectId,
            );

    const effectChain = activeChannel?.effect_chain ?? [];
    function selectByIndex(index: number) {
        const effect = effectChain[index];
        if (effect) {
            setSelection({kind: effect.kind, effectId: effect.data.id});
        }
    }

    const selectedChainIndex =
        selection === "amp"
            ? -1
            : effectChain.findIndex(
                (e) => e.kind === selection.kind && e.data.id === selection.effectId,
            );

    useHotkeys(
        ["1", "2", "3", "4", "5", "6", "7", "8", "9", "0"],
        (_, handler) => {
            const digit = handler.keys?.[0];
            if (digit === undefined) return;
            if (digit === "1") {
                setSelection("amp");
                return;
            }
            const index = digit === "0" ? 8 : Number(digit) - 2;
            selectByIndex(index);
        },
        {preventDefault: true},
    );
    useHotkeys(
        "space",
        () => {
            if (selection === "amp") {
                setIsActive(!isActive);
                return;
            }
            if (!resolvedSelection || resolvedSelection === "amp") return;
            const effectId = resolvedSelection.data.id;
            const currentlyActive = resolvedSelection.data.is_active;
            const nextActive = !currentlyActive;

            updateEffectActiveState(effectId, nextActive);
            void toggleEffect({effectId}).catch(() => {
                updateEffectActiveState(effectId, currentlyActive);
            });
        },
        {preventDefault: true},
        [resolvedSelection, selection, isActive],
    );
    useHotkeys(
        ["arrowleft", "arrowright"],
        (_, handler) => {
            if (selectedChainIndex === -1) return;
            const direction = handler.keys?.[0] === "arrowleft" ? -1 : 1;
            const newIndex = selectedChainIndex + direction;
            if (newIndex < 0 || newIndex >= effectChain.length) return;

            void moveEffect(selectedChainIndex, newIndex).then(() =>
                applyChangesToChainOrder(),
            );
        },
        {preventDefault: true},
        [selectedChainIndex, effectChain.length],
    );

    return (
        <Box
            sx={{
                p: 4,
                display: "flex",
                flexDirection: "column",
                alignItems: "center",
                justifyContent: "start",
                minHeight: "100vh",
                gap: 4
            }}
        >
            {activeChannel &&
                <EffectChain
                    effects={activeChannel.effect_chain}
                    selected={resolvedSelection ?? "amp"}
                    onOpenKeybinds={() => setIsKeybindsOpen(true)}
                    onSelectionChange={(selected: EffectDto | "amp") => {
                        if (selected === "amp") {
                            setSelection("amp");
                            return;
                        }

                        setSelection({
                            kind: selected.kind,
                            effectId: selected.data.id,
                        });
                    }}
                />
            }
            {
                    (resolvedSelection === "amp" || !resolvedSelection)
                    ? <DefaultAmpControls/>
                    : resolvedSelection.kind === "Cabinet"
                    ? <CabinetEffect effect={resolvedSelection}/>
                    : <EffectPedal effect={resolvedSelection}/>
            }
            <KeybindsDialog open={isKeybindsOpen} onClose={() => setIsKeybindsOpen(false)}/>
        </Box>
    );
}