import {Box, Typography} from "@mui/material";
import chroma from "chroma-js";
import {useEffect, useState} from "react";
import {EffectDto, toggleEffect} from "../domain";
import {useAmpStore} from "../state/AmpConfigStore.tsx";

interface CabinetEffectProps {
    effect: EffectDto;
    onToggle?: (effectId: number, isActive: boolean) => void;
}

export function CabinetEffect({ effect, onToggle }: CabinetEffectProps) {
    if (effect.kind !== "Cabinet") return null;

    const [isActive, setIsActive] = useState(effect.data.is_active);
    const updateEffectActiveState = useAmpStore((state) => state.updateEffectActiveState);
    const cabBlackColor = "#1E1E1D"
    const baseColor = chroma(effect.data.color).desaturate(0.4).hex();

    useEffect(() => {
        setIsActive(effect.data.is_active);
    }, [effect.data.id, effect.data.is_active]);

    async function handlePowerToggle() {
        try {
            const newActive = await toggleEffect({ effectId: effect.data.id });
            setIsActive(newActive);
            updateEffectActiveState(effect.data.id, newActive);
            onToggle?.(effect.data.id, newActive);
        } catch (error) {
            console.error(`Failed to toggle cabinet ${effect.data.id}:`, error);
        }
    }

    return (
        <Box
            sx={{
                width: 400,
                height: 300,
                display: "flex",
                flexDirection: "column",
                alignItems: "center",
                position: "relative",
                filter: "drop-shadow(0 8px 16px rgba(0,0,0,0.45))",
            }}
        >
            {/* Cabinet body with gradient slope - Marshall inspired */}
            <Box
                sx={{
                    width: "100%",
                    height: "100%",
                    background: `linear-gradient(180deg, 
                        ${chroma(cabBlackColor).brighten(0.8).hex()} 0%, 
                        ${chroma(cabBlackColor).brighten(0.8).hex()} 50%, 
                        ${chroma(cabBlackColor).darken(1.2).hex()} 100%)`,
                    border: "2px solid rgba(0,0,0,0.5)",
                    borderRadius: "8px 8px 12px 12px",
                    position: "relative",
                    overflow: "hidden",
                    display: "flex",
                    flexDirection: "column",
                }}
            >
                <Box
                    sx={{
                        px: 2,
                        pt: 2,
                        pb: 1,
                        display: "flex",
                        alignItems: "center",
                        justifyContent: "flex-end",
                    }}
                >
                    <Box
                        component={"button"}
                        onClick={handlePowerToggle}
                        sx={{
                            width: 10,
                            height: 10,
                            borderRadius: "50%",
                            bgcolor: isActive ? "#00ff00" : "#ff0000",
                            boxShadow: isActive ? "0 0 6px #00ff00" : "0 0 6px #ff0000",
                            border: "1px solid rgba(0,0,0,0.3)",
                            cursor: "pointer",
                            padding: 0,
                            minWidth: 0,
                            lineHeight: 0,
                            display: "block",
                            flexShrink: 0,
                        }}
                    />
                </Box>

                {/* Speaker grille area */}
                <Box
                    sx={{
                        flex: 1,
                        mx: 2,
                        my: 2,
                        borderRadius: 1,
                        display:"flex",
                        justifyContent:"center",
                        alignItems:"center",
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
                        {effect.data.name}
                    </Typography>
                </Box>
            </Box>
        </Box>
    );
}

