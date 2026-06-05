import {useAmpStore} from "../state/AmpConfigStore.tsx";
import {DropResult} from "@hello-pangea/dnd";

export function useEffectMoves(effectsLength: number) {
    const {applyChangesToChainOrder, moveEffect} = useAmpStore();

    async function handleMovePedal(currentIndex: number, direction: "left" | "right") {
        const newIndex = direction === "left" ? currentIndex - 1 : currentIndex + 1;
        if (newIndex < 0 || newIndex >= effectsLength) return;

        moveEffect(currentIndex, newIndex);
        await applyChangesToChainOrder();
    }

    async function onDragEnd(result: DropResult) {
        if (!result.destination) return;

        const sourceIndex = result.source.index;
        const newIndex = result.destination.index;

        if (sourceIndex === newIndex) return;

        moveEffect(sourceIndex, newIndex);
        await applyChangesToChainOrder();
    }

    return {handleMovePedal, onDragEnd};
}
