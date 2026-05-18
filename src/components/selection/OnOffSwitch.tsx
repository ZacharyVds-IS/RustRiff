import {Box} from "@mui/material";

interface OnOffSwitchProps {
    isActive: boolean;
    onClick: () => void;
}

export function OnOffSwitch({ isActive, onClick }: OnOffSwitchProps) {
    return (
        <Box
            sx={{
                display: "flex",
                flexDirection: "column",
                alignItems: "center",
                gap: "8px",
                width: 24,
            }}
        >
            {/* 1. VOX-STYLE JEWEL PILOT LIGHT */}
            <Box
                sx={{
                    width: 12,
                    height: 12,
                    borderRadius: "50%",
                    position: "relative",
                    border: "2px solid #333333",
                    boxShadow: "0 1px 2px rgba(0,0,0,0.6), inset 0 1px 1px rgba(0,0,0,0.8)",

                    background: isActive
                        ? "radial-gradient(circle at 35% 35%, #99ff99 0%, #00aa00 60%, #004400 100%)"
                        : "radial-gradient(circle at 35% 35%, #ff9999 0%, #cc0000 60%, #440000 100%)",

                    transition: "background 0.15s ease-in-out, box-shadow 0.15s ease-in-out",

                    "&::after": {
                        content: '""',
                        position: "absolute",
                        inset: -2,
                        borderRadius: "50%",
                        boxShadow: isActive
                            ? "0 0 8px rgba(0, 255, 0, 0.7)"
                            : "0 0 6px rgba(255, 0, 0, 0.5)",
                        opacity: 1,
                        zIndex: 1,
                        transition: "box-shadow 0.15s ease-in-out",
                    }
                }}
            >
                {/* Internal glass reflection glint */}
                <Box
                    sx={{
                        position: "absolute",
                        top: "1.5px",
                        left: "2px",
                        width: "3px",
                        height: "1.5px",
                        bgcolor: "rgba(255,255,255,0.6)",
                        borderRadius: "50%",
                        transform: "rotate(-15deg)",
                        zIndex: 2
                    }}
                />
            </Box>

            {/* 2. MATTE BLACK ROCKER SWITCH HOUSING */}
            <Box
                component="button"
                onClick={onClick}
                sx={{
                    width: 20,
                    height: 28,
                    borderRadius: "2px",
                    border: "1px solid #111111",
                    cursor: "pointer",
                    padding: "3px 2px",
                    minWidth: 0,
                    flexShrink: 0,
                    display: "flex",
                    flexDirection: "column",
                    bgcolor: "#1a1a1a", // Outer frame color
                    boxShadow: "inset 0 1px 3px rgba(0,0,0,0.8), 0 1px 1px rgba(255,255,255,0.05)",
                }}
            >
                {/* TOP HALF OF ROCKER (I / ON Side) */}
                <Box
                    sx={{
                        width: "100%",
                        height: isActive ? "60%" : "40%",
                        transition: "height 0.12s ease-in-out, background 0.12s ease-in-out",
                        borderRadius: "2px 2px 0 0",
                        background: isActive
                            ? "linear-gradient(180deg, #151515 0%, #222222 100%)"
                            : "linear-gradient(180deg, #444444 0%, #2d2d2d 100%)",
                        borderBottom: isActive ? "none" : "1px solid #111111"
                    }}
                />

                {/* BOTTOM HALF OF ROCKER (O / OFF Side) */}
                <Box
                    sx={{
                        width: "100%",
                        height: isActive ? "40%" : "60%",
                        transition: "height 0.12s ease-in-out, background 0.12s ease-in-out",
                        borderRadius: "0 0 2px 2px",
                        background: isActive
                            ? "linear-gradient(180deg, #2d2d2d 0%, #444444 100%)"
                            : "linear-gradient(180deg, #222222 0%, #151515 100%)",
                        borderTop: isActive ? "1px solid #111111" : "none"
                    }}
                />
            </Box>
        </Box>
    );
}