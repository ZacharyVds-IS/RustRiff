import {useEffect, useRef, useState} from "react";
import {PitchSnapshotDto} from "./useLiveTuner.ts";

export function useTunerNeedle(pitch: PitchSnapshotDto | null, isSignalActive: boolean) {
    const [renderCents, setRenderCents] = useState(0);
    const centsRef = useRef(0);

    useEffect(() => {
        let animationFrameId: number;

        const updateNeedle = () => {
            const targetCents = isSignalActive && pitch ? pitch.cents_deviation : centsRef.current;
            const alpha = 0.16;
            const nextCents = centsRef.current + alpha * (targetCents - centsRef.current);
            centsRef.current = nextCents;
            setRenderCents(nextCents);
            animationFrameId = requestAnimationFrame(updateNeedle);
        };

        animationFrameId = requestAnimationFrame(updateNeedle);
        return () => cancelAnimationFrame(animationFrameId);
    }, [pitch, isSignalActive]);

    return renderCents;
}
