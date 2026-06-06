import {useCallback, useEffect, useRef, useState} from "react";

interface UseKnobDragOptions {
    value: number;
    min: number;
    max: number;
    step: number;
    onChange?: (newValue: number) => void;
    disabled?: boolean;
}

export function useKnobDrag({value, min, max, step, onChange, disabled}: UseKnobDragOptions) {
    const [localValue, setLocalValue] = useState(value);
    const startRef = useRef({value: 0, y: 0});
    const sensitivity = 200;

    useEffect(() => {
        setLocalValue(value);
    }, [value]);

    const handleMouseDown = useCallback((e: React.MouseEvent<HTMLDivElement>) => {
        if (disabled) return;
        startRef.current = {value: localValue, y: e.clientY};

        const handleMouseMove = (moveEvent: MouseEvent) => {
            const deltaY = startRef.current.y - moveEvent.clientY;
            const range = max - min;
            const change = (deltaY / sensitivity) * range;
            let newValue = startRef.current.value + change;
            if (step > 0) newValue = Math.round(newValue / step) * step;
            const clampedValue = Math.min(Math.max(newValue, min), max);
            setLocalValue(clampedValue);
            onChange?.(clampedValue);
        };

        const handleMouseUp = () => {
            window.removeEventListener("mousemove", handleMouseMove);
            window.removeEventListener("mouseup", handleMouseUp);
        };

        window.addEventListener("mousemove", handleMouseMove);
        window.addEventListener("mouseup", handleMouseUp);
    }, [disabled, localValue, min, max, step, onChange, sensitivity]);

    const percentage = (localValue - min) / (max - min);
    const rotation = percentage * 270 - 135;

    return {
        localValue,
        rotation,
        handleMouseDown,
    };
}
