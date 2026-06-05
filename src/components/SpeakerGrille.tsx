import {Box, Typography} from "@mui/material";

interface SpeakerGrilleProps {
    baseColor: string;
    name: string;
}

export function SpeakerGrille({baseColor, name}: SpeakerGrilleProps) {
    return (
        <Box
            sx={{
                flex: 1,
                mx: 2,
                my: 2,
                borderRadius: 1,
                display: "flex",
                justifyContent: "center",
                alignItems: "center",
                border: `5px solid ${baseColor}`,
                background: `repeating-linear-gradient(
                    90deg,
                    rgba(0,0,0,0.15) 0px,
                    rgba(0,0,0,0.15) 3px,
                    rgba(0,0,0,0.08) 3px,
                    rgba(0,0,0,0.08) 6px
                )`,
                boxShadow: "inset 0 1px 3px rgba(0,0,0,0.2)",
            }}
        >
            <Typography
                sx={{
                    fontWeight: 900,
                    fontSize: "0.9rem",
                    textTransform: "uppercase",
                    letterSpacing: 0.5,
                    color: "white",
                    fontStyle: "italic",
                }}
                noWrap
            >
                {name}
            </Typography>
        </Box>
    );
}
