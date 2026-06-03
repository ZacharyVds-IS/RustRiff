import {Box} from "@mui/material";
import chroma from "chroma-js";

interface WahPedalPreviewProps {
    mainColor: string;
    isActive?: boolean;
    pedalPosition?: number;
}

export function WahPedalPreview({ mainColor, isActive = false, pedalPosition = 0.5 }: WahPedalPreviewProps) {
    const baseColor = chroma(mainColor);
    const safePosition = Math.max(0, Math.min(1, pedalPosition));
    const highlightStart = safePosition * 100;
    const highlightEnd = Math.min(100, highlightStart + 16);
    const background = `linear-gradient(180deg, ${baseColor.darken(1.1).hex()} 0%, ${baseColor.brighten(0.55).hex()} ${highlightStart}%, ${baseColor.brighten(0.2).hex()} ${highlightEnd}%, ${baseColor.darken(0.95).hex()} 100%)`;
    const stripePattern = `repeating-linear-gradient(90deg, ${baseColor.brighten(0.4).hex()} 0px, ${baseColor.brighten(0.4).hex()} 2px, ${baseColor.darken(0.8).hex()} 2px, ${baseColor.darken(0.8).hex()} 4px)`;

    return (
        <Box
            sx={{
                width: 48,
                height: 85,
                position: "relative",
                borderRadius: "6px",
                border: "1px solid rgba(0,0,0,0.55)",
                background,
                overflow: "hidden",
                boxShadow: isActive
                    ? "0 4px 8px rgba(0,0,0,0.4), inset 0 1px 2px rgba(255,255,255,0.15)"
                    : "0 2px 4px rgba(0,0,0,0.25), inset 0 1px 1px rgba(255,255,255,0.08)",
                filter: isActive ? "none" : "grayscale(55%)",
                opacity: isActive ? 1 : 0.78,
                transition: "background 0.1s ease-out, box-shadow 0.1s ease-out, opacity 0.1s ease-out",
            }}
        >
            <Box
                sx={{
                    position: "absolute",
                    inset: 4,
                    borderRadius: "4px",
                    border: "1px solid rgba(0,0,0,0.35)",
                    background: stripePattern,
                    opacity: 0.6,
                    pointerEvents: "none",
                }}
            />
        </Box>
    );
}

