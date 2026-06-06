import {useEffect, useState} from "react";
import {useHotkeys} from "react-hotkeys-hook";
import {useAmpStore} from "../state/AmpConfigStore.tsx";
import {EffectDto, toggleEffect} from "../domain";

export type EffectSelection =
    | "amp"
    | {
    kind: EffectDto["kind"];
    effectId: string;
};

export function useEffectSelection() {
    const activeChannel = useAmpStore((state) =>
        state.channels.find((c) => c.id === state.current_channel)
    );
    const currentChannelId = useAmpStore((state) => state.current_channel);
    const isActive = useAmpStore((state) => state.is_active);
    const setIsActive = useAmpStore((state) => state.setIsActive);
    const updateEffectActiveState = useAmpStore((state) => state.updateEffectActiveState);
    const moveEffect = useAmpStore((state) => state.moveEffect);
    const applyChangesToChainOrder = useAmpStore((state) => state.applyChangesToChainOrder);

    const [selection, setSelection] = useState<EffectSelection>("amp");
    const [isKeybindsOpen, setIsKeybindsOpen] = useState(false);

    useEffect(() => {
        setSelection("amp");
    }, [currentChannelId]);

    const effectChain = activeChannel?.effect_chain ?? [];

    const resolvedSelection: EffectDto | "amp" | undefined =
        selection === "amp"
            ? "amp"
            : effectChain.find(
                (effect) =>
                    effect.kind === selection.kind &&
                    effect.data.id === selection.effectId,
            );

    const selectedChainIndex =
        selection === "amp"
            ? -1
            : effectChain.findIndex(
                (e) => e.kind === selection.kind && e.data.id === selection.effectId,
            );

    function selectByIndex(index: number) {
        const effect = effectChain[index];
        if (effect) {
            setSelection({kind: effect.kind, effectId: effect.data.id});
        }
    }

    function navigateSelection(direction: -1 | 1) {
        if (effectChain.length === 0) {
            setSelection("amp");
            return;
        }

        const selectableItemCount = effectChain.length + 1;
        const currentSelectableIndex = selection === "amp" ? 0 : selectedChainIndex + 1;
        const nextSelectableIndex =
            (currentSelectableIndex + direction + selectableItemCount) % selectableItemCount;

        if (nextSelectableIndex === 0) {
            setSelection("amp");
            return;
        }

        selectByIndex(nextSelectableIndex - 1);
    }

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
        (event, handler) => {
            if (event.shiftKey) return;

            const direction = handler.keys?.[0] === "arrowleft" ? -1 : 1;

            navigateSelection(direction);
        },
        {preventDefault: true},
        [selection, selectedChainIndex, effectChain.length],
    );
    useHotkeys(
        ["shift+arrowleft", "shift+arrowright"],
        (event) => {
            if (selectedChainIndex === -1) return;

            const direction = event.key === "ArrowLeft" ? -1 : 1;
            const newIndex = selectedChainIndex + direction;
            if (newIndex < 0 || newIndex >= effectChain.length) return;

            const fromIndex = selectedChainIndex;
            const toIndex = newIndex;
            void moveEffect(fromIndex, toIndex);
            void applyChangesToChainOrder().catch(() => {
                void moveEffect(toIndex, fromIndex);
            });
        },
        {preventDefault: true},
        [selectedChainIndex, effectChain.length],
    );

    return {
        selection,
        setSelection,
        resolvedSelection,
        selectedChainIndex,
        effectChain,
        isKeybindsOpen,
        setIsKeybindsOpen,
        activeChannel,
    };
}
