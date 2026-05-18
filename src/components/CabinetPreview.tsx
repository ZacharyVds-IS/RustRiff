import {Box} from "@mui/material";
import chroma from "chroma-js";

interface CabinetPreviewProps {
    mainColor: string;
    isActive?: boolean;
}

export function CabinetPreview({ mainColor, isActive = false }: CabinetPreviewProps) {
    const cabBlackColor = "#1E1E1D";
    const baseColor = chroma(mainColor).desaturate(0.4).hex();

    return (
        <Box
            sx={{
                width: 80,
                height: 60,
                display: "flex",
                flexDirection: "column",
                position: "relative",
                filter: isActive ? 'drop-shadow(0 4px 6px rgba(0,0,0,0.5))': 'grayscale(60%)',
                opacity: isActive ? 1 : 0.75,
            }}
        >
            {/* Cabinet body */}
            <Box
                sx={{
                    width: "100%",
                    height: "100%",
                    background: `linear-gradient(180deg, 
                        ${chroma(cabBlackColor).brighten(0.8).hex()} 0%, 
                        ${chroma(cabBlackColor).brighten(0.8).hex()} 50%, 
                        ${chroma(cabBlackColor).darken(1.2).hex()} 100%)`,
                    border: "1px solid rgba(0,0,0,0.5)",
                    borderRadius: "4px 4px 6px 6px",
                    position: "relative",
                    overflow: "hidden",
                    display: "flex",
                    flexDirection: "column",
                    padding: "6px",
                    boxSizing: "border-box",
                }}
            >
                {/* Power LED - top right */}
                <Box
                    sx={{
                        position: "absolute",
                        top: 6,
                        right: 6,
                        width: 6,
                        height: 6,
                        borderRadius: "50%",
                        bgcolor: isActive ? "#00ff00" : "#ff0000",
                        boxShadow: isActive ? "0 0 4px #00ff00" : "0 0 4px #ff0000",
                        border: "0.5px solid rgba(0,0,0,0.3)",
                        zIndex: 2,
                    }}
                />

                {/* Speaker grille area with colored border */}
                <Box
                    sx={{
                        flex: 1,
                        mt:1,
                        borderRadius: 0.5,
                        border: `2px solid ${baseColor}`,
                        background: `repeating-linear-gradient(
                            90deg,
                            rgba(0,0,0,0.15) 0px,
                            rgba(0,0,0,0.15) 2px,
                            rgba(0,0,0,0.08) 2px,
                            rgba(0,0,0,0.08) 4px
                        )`,
                        boxShadow: "inset 0 1px 2px rgba(0,0,0,0.3)",
                    }}
                />
            </Box>
        </Box>
    );
}
