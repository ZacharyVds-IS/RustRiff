import {Box, CircularProgress, Paper, Typography} from "@mui/material";
import {useLiveTuner} from "../hooks/useLiveTuner.ts";
import {FallbackText} from "../components/FallbackText.tsx";
import {useEffect, useRef, useState} from "react";
import {PitchScale} from "../components/PitchScale.tsx";

export function TunerScreen() {
    const {pitch, loadError} = useLiveTuner();
    // Cache memory for preserving the last valid tracking snapshot
    const [lastValidPitch, setLastValidPitch] = useState<typeof pitch>(null);

    // 1. Smooth Needle State using a mutable ref for real-time frame interpolation
    const [renderCents, setRenderCents] = useState(0);
    const centsRef = useRef(0);

    // 2. Hysteresis State for stabilizing note flicker
    const [stableNote, setStableNote] = useState("—");
    const noteCounter = useRef({name: "", count: 0});

    // Determine if a string is currently ringing right now
    const isSignalActive = pitch !== null && pitch.frequency_hz > 20 && pitch.note_name !== "---";

    // --- EFFECT 1: Handle Caching and Hysteresis (Note Debouncing) ---
    useEffect(() => {
        if (isSignalActive) {
            setLastValidPitch(pitch);

            // Requires the backend to register the exact same note for 3 consecutive
            // hook updates before swapping the text. Prevents flickering on borders.
            if (pitch.note_name === noteCounter.current.name) {
                noteCounter.current.count += 1;
                if (noteCounter.current.count >= 3) {
                    setStableNote(pitch.note_name);
                }
            } else {
                noteCounter.current.name = pitch.note_name;
                noteCounter.current.count = 1;
            }
        }
    }, [pitch, isSignalActive]);

    // --- EFFECT 2: The Shock Absorber Animation Loop (Lerp / EMA) ---
    useEffect(() => {
        let animationFrameId: number;

        const updateNeedle = () => {
            // Target destination based on live pitch or where the cached pitch left off
            const targetCents = isSignalActive && pitch ? pitch.cents_deviation : centsRef.current;

            // SMOOTHING COEFFICIENT: Lower = smoother/heavier, Higher = snappier/more responsive
            const alpha = 0.16;

            // Calculate new position
            const nextCents = centsRef.current + alpha * (targetCents - centsRef.current);

            centsRef.current = nextCents;
            setRenderCents(nextCents);

            animationFrameId = requestAnimationFrame(updateNeedle);
        };

        animationFrameId = requestAnimationFrame(updateNeedle);
        return () => cancelAnimationFrame(animationFrameId);
    }, [pitch, isSignalActive]);

    if (loadError) {
        return (
            <FallbackText
                title="Tuner Error"
                description="Could not access audio device for tuning"
                error={loadError}
            />
        );
    }

    // Read from live input if active; otherwise, drop back to the cached note history
    const displayPitch = isSignalActive ? pitch : lastValidPitch;

    // Show a clean idle interface if the user hasn't played a single note since startup
    if (!displayPitch) {
        return (
            <Box
                sx={{
                    display: "flex",
                    flexDirection: "column",
                    gap: 2,
                    justifyContent: "center",
                    alignItems: "center",
                    minHeight: "100vh",
                    bgcolor: "background.default"
                }}
            >
                <CircularProgress size={40} thickness={4}/>
                <Typography variant="body1" color="text.secondary" sx={{fontWeight: "medium"}}>
                    Listening for audio input... Pluck a string! 🎸
                </Typography>
            </Box>
        );
    }

    const {frequency_hz} = displayPitch;

    const is_in_tune = Math.abs(renderCents) <= 5;

    return (
        <Box
            sx={{
                p: 4,
                display: "flex",
                flexDirection: "column",
                alignItems: "center",
                justifyContent: "center",
                minHeight: "100vh",
                gap: 1,
                bgcolor: "background.default",
                width: "100%",
                maxWidth: 600, // Widened container to allow a broader tracking layout
                mx: "auto"
            }}
        >
            {/* Top Card: Main Note Display */}
            <Paper
                elevation={3}
                sx={{
                    p: 4,
                    borderRadius: 4,
                    textAlign: "center",
                    width: "100%",
                    border: 2,
                    borderColor: is_in_tune ? "success.main" : "transparent",
                    transition: "opacity 0.2s ease-in-out, border-color 0.15s ease",
                }}
            >
                {/* Big Note Indicator (Uses the stabilized state when active) */}
                <Typography
                    variant="h1"
                    color={is_in_tune ? "success.main" : "text.primary"}
                    sx={{fontWeight: "bold"}}
                >
                    {isSignalActive ? stableNote : displayPitch.note_name}
                </Typography>

                {/* Live Frequency Readout */}
                <Typography variant="h6" color="text.secondary" sx={{mt: 1}}>
                    {`${frequency_hz.toFixed(1)} Hz`}
                </Typography>
            </Paper>

            <PitchScale isSignalActive={isSignalActive} renderCents={renderCents} is_in_tune={is_in_tune}/>
        </Box>
    );
}