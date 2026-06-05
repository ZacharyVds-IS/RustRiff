import {Box, CircularProgress, Paper, Typography} from "@mui/material";
import {FallbackText} from "../components/FallbackText.tsx";
import {PitchScale} from "../components/PitchScale.tsx";
import {useTunerPitch} from "../hooks/useTunerPitch.ts";
import {useTunerNeedle} from "../hooks/useTunerNeedle.ts";

export function TunerScreen() {
    const {pitch, loadError, lastValidPitch, stableNote, isSignalActive} = useTunerPitch();
    const renderCents = useTunerNeedle(pitch, isSignalActive);

    if (loadError) {
        return (
            <FallbackText
                title="Tuner Error"
                description="Could not access audio device for tuning"
                error={loadError}
            />
        );
    }

    const displayPitch = isSignalActive ? pitch : lastValidPitch;
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
                    Listening for audio input... Make sure you're amp is on and pluck a string!
                </Typography>
            </Box>
        );
    }

    const {frequency_hz} = displayPitch;
    const isInTune = Math.abs(renderCents) <= 5;

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
                maxWidth: 600,
                mx: "auto"
            }}
        >
            <Paper
                elevation={3}
                sx={{
                    p: 4,
                    borderRadius: 4,
                    textAlign: "center",
                    width: "100%",
                    border: 2,
                    borderColor: isInTune ? "success.main" : "transparent",
                    transition: "opacity 0.2s ease-in-out, border-color 0.15s ease",
                }}
            >
                <Typography
                    variant="h1"
                    color={isInTune ? "success.main" : "text.primary"}
                    sx={{fontWeight: "bold"}}
                >
                    {isSignalActive ? stableNote : displayPitch.note_name}
                </Typography>

                <Typography variant="h6" color="text.secondary" sx={{mt: 1}}>
                    {`${frequency_hz.toFixed(1)} Hz`}
                </Typography>
            </Paper>

            <PitchScale isSignalActive={isSignalActive} renderCents={renderCents} is_in_tune={isInTune}/>
        </Box>
    );
}
