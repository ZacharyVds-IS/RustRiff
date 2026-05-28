import {Box, Paper, Typography} from "@mui/material";

interface PitchScaleProps {
    renderCents: number;
    isSignalActive: boolean;
    is_in_tune: boolean;
}

export function PitchScale({ renderCents, isSignalActive, is_in_tune }: PitchScaleProps) {
    return (
        <Paper
            elevation={3}
            sx={{
                p: 4,
                borderRadius: 4,
                width: "100%",
                display: "flex",
                flexDirection: "column",
                alignItems: "center"
            }}
        >
            <Typography
                variant="body1"
                color={
                    renderCents > 1.5
                        ? "error.main"
                        : renderCents < -1.5
                            ? "info.main"
                            : "success.main"
                }
                sx={{ fontWeight: "medium", mb: 2 }}
            >
                {
                    Math.abs(renderCents) < 1
                        ? "Perfect Tune!"
                        : `${renderCents > 0 ? "+" : ""}${renderCents.toFixed(0)} cents`
                }
            </Typography>

            {/* Wide Visual needle tracker box */}
            <Box
                sx={{
                    width: "100%",
                    height: 28,
                    bgcolor: "action.disabledBackground",
                    borderRadius: 1,
                    position: "relative",
                    overflow: "visible",
                    display: "flex",
                    alignItems: "center",

                    // Pitch lines at -5 and 5 cents, indicating the "in-tune" threshold.
                    "&::after": {
                        content: '""',
                        position: "absolute",
                        left: "50%",
                        transform: "translateX(-50%)",
                        width: "calc(9% + 2px)",
                        height: "100%",
                        borderLeft: `2px solid rgba(0, 0, 0, 0.15)`,
                        borderRight: `2px solid rgba(0, 0, 0, 0.15)`,
                        borderColor: "success.main",
                        pointerEvents: "none",
                        zIndex: 1,
                        transition: "border-color 0.2s ease"
                    },

                    // The Perfect Pitch Center Index Line (Overflowing Analog Marker)
                    "&::before": {
                        content: '""',
                        position: "absolute",
                        left: "50%",
                        transform: "translateX(-50%)",
                        width: "3px",
                        height: "42px",
                        bgcolor: isSignalActive ? "success.main" : "text.disabled",
                        zIndex: 1,
                        opacity: 0.8,
                        transition: "background-color 0.2s ease",
                    }
                }}
            >
                {/* Outer Boundary ticks (-10 and 10 cents lines) */}
                <Box
                    sx={{
                        position: "absolute",
                        left: "50%",
                        transform: "translateX(-50%)",
                        width: "calc(18% + 2px)",
                        height: "100%",
                        pointerEvents: "none",
                        zIndex: 1,
                        "&::before": {
                            content: '""',
                            position: "absolute",
                            width: "100%",
                            height: "100%",
                            borderLeft: `2px dashed rgba(0, 0, 0, 0.25)`,
                            borderRight: `2px dashed rgba(0, 0, 0, 0.25)`,
                        }
                    }}
                />

                {/* Smoothed Floating Needle */}
                <Box
                    sx={{
                        width: 4,
                        height: 36,
                        bgcolor: is_in_tune ? "primary.main" : "text.disabled",
                        borderRadius: 1.5,
                        position: "absolute",
                        zIndex: 2,
                        left: `calc(50% + ${Math.min(50, Math.max(-50, renderCents)) * 0.9}%)`,
                        transform: "translateX(-50%)",
                        transition: "bgcolor 0.2s ease"
                    }}
                />
            </Box>
        </Paper>
    );
}