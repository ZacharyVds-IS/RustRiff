import {useEffect, useState} from "react";
import {toggleEffect} from "../domain";
import {useAmpStore} from "../state/AmpConfigStore.tsx";

export function useEffectToggle(effectId: string, initialIsActive: boolean, onToggle?: (effectId: string, isActive: boolean) => void) {
    const [isActive, setIsActive] = useState(initialIsActive);
    const updateEffectActiveState = useAmpStore((state) => state.updateEffectActiveState);

    useEffect(() => {
        setIsActive(initialIsActive);
    }, [effectId, initialIsActive]);

    async function handleToggle() {
        try {
            const newActive = await toggleEffect({effectId});
            setIsActive(newActive);
            updateEffectActiveState(effectId, newActive);
            onToggle?.(effectId, newActive);
        } catch (error) {
            console.error(`Failed to toggle effect ${effectId}:`, error);
        }
    }

    return {isActive, handleToggle};
}
