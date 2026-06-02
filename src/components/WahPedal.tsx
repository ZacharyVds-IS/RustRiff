import {Box, Stack, Switch, Tooltip} from "@mui/material";
import chroma from "chroma-js";
import {setWahPedalPosition, toggleEffect, WahDto} from "../domain";
import {useEffect, useRef, useState} from "react";
import {useAmpStore} from "../state/AmpConfigStore.tsx";

interface WahPedalProps {
    effect: {
        kind: "Wah";
        data: WahDto;
    };
    onToggle?: (effectId: string, isActive: boolean) => void;
}

export function WahPedal({ effect, onToggle }: WahPedalProps) {
    const sliderRef = useRef<HTMLInputElement>(null);
    const [isActive, setIsActive] = useState(effect.data.is_active);

    const updateEffectActiveState = useAmpStore((state) => state.updateEffectActiveState);
    const updateWahParams = useAmpStore((state) => state.updateWahParams);
    const chassisColor = chroma(effect.data.color).hex();

    useEffect(() => {
        setIsActive(effect.data.is_active);
    }, [effect.data.id, effect.data.is_active]);

    async function handleToggleChange() {
        try {
            const newActive = await toggleEffect({ effectId: effect.data.id });
            setIsActive(newActive);
            updateEffectActiveState(effect.data.id, newActive);
            onToggle?.(effect.data.id, newActive);
        } catch (error) {
            console.error(`Failed to toggle effect ${effect.data.id}:`, error);
        }
    }

    function handlePedalPositionChange(effectId: string, pedalPosition: number, previousPedalPosition: number) {
        updateWahParams(effectId, { pedal_position: pedalPosition });
        void setWahPedalPosition({ effectId, pedalPosition }).catch((error) => {
            console.error("Failed to update Wah pedal position:", error);
            updateWahParams(effectId, { pedal_position: previousPedalPosition });
        });
    }

    const handleSliderChange = (e: React.ChangeEvent<HTMLInputElement>) => {
        const newValue = parseFloat(e.target.value);
        const previousPosition = effect.data.pedal_position;
        handlePedalPositionChange(effect.data.id, newValue, previousPosition);
    };

    // Map pedal_position (0 to 1) to a realistic tilt angle in degrees.
    const currentRotationX = -12 + effect.data.pedal_position * 24;

    return (
        <Stack direction={"column"} sx={{ alignItems: "center" }}>
            <Tooltip title="Drag up/down to adjust wah">
                <Box
                    sx={{
                        width: 160,
                        height: 270,
                        background: `linear-gradient(180deg, 
                            ${chroma(chassisColor).darken(0.4)} 0%,
                            ${chroma(chassisColor).brighten(0.2)} 15%,
                            ${chroma(chassisColor).darken(0.8)} 100%)`,
                        borderRadius: '12px',
                        border: '2px solid rgba(0,0,0,0.6)',
                        boxShadow: '0 8px 16px rgba(0,0,0,0.4), inset 0 2px 4px rgba(255,255,255,0.2)',
                        cursor: 'ns-resize', // Changes cursor to up/down arrows to signal drag direction
                        padding: '12px',
                        paddingTop:"28px",
                        boxSizing: 'border-box',
                        position: 'relative',
                        filter: (theme) => theme.palette.mode === 'dark'
                            ? 'drop-shadow(0 8px 16px rgba(255, 255, 255, 0.1))'
                            : 'drop-shadow(0 4px 8px rgba(0,0,0,0.3))',
                        perspective: '600px',
                        transformStyle: 'preserve-3d',
                    }}
                >
                    {/* Inner Texture Foot Pad */}
                    <Box
                        sx={{
                            width: "100%",
                            height: "calc(100% - 34px)",
                            borderRadius: "6px",
                            border: '1px solid rgba(0,0,0,0.5)',
                            background: `repeating-linear-gradient(
                                90deg,
                                ${chroma(chassisColor).brighten(0.4)} 0px,
                                ${chroma(chassisColor).brighten(0.4)} 4px,
                                ${chroma(chassisColor).darken(0.8)} 4px,
                                ${chroma(chassisColor).darken(0.8)} 8px
                            )`,
                            transform: `rotateX(${currentRotationX}deg)`,
                            transition: 'transform 0.05s ease-out, filter 0.05s ease-out',
                            transformOrigin: 'center bottom',
                            filter: `drop-shadow(0px ${4 + (effect.data.pedal_position * 4)}px 5px rgba(0, 0, 0, 0.5))`,
                        }}
                    />

                    <Box
                        sx={{
                            position: 'absolute',
                            top: 4,
                            left: '50%',
                            transform: 'translateX(-50%)',
                            width: 8,
                            height: 8,
                            borderRadius: '50%',
                            border: '1px solid rgba(0,0,0,0.35)',
                            bgcolor: isActive ? '#00ff00' : '#ff0000',
                            boxShadow: isActive ? '0 0 6px #00ff00' : '0 0 6px #ff0000',
                            zIndex: 15,
                            transition: 'background-color 0.1s, box-shadow 0.1s',
                        }}
                    />

                    <Box
                        sx={{
                            position: 'absolute',
                            left: 0,
                            right: 0,
                            bottom: 2,
                            display: 'flex',
                            justifyContent: 'center',
                            alignItems: 'center',
                            zIndex: 20,
                        }}
                    >
                        <Switch
                            size="small"
                            checked={isActive}
                            onChange={handleToggleChange}
                            onClick={(e) => e.stopPropagation()}
                            slotProps={{ input: { 'aria-label': 'Toggle wah pedal' } }}
                            sx={{
                                '& .MuiSwitch-track': {
                                    bgcolor: 'rgba(100,100,100,0.5) !important',
                                    opacity: '1 !important',
                                },
                                '& .MuiSwitch-thumb': {
                                    bgcolor: '#cccccc',
                                },
                            }}
                        />
                    </Box>

                    {/* Invisible full-surface VERTICAL slider overlay */}
                    <input
                        ref={sliderRef}
                        type="range"
                        min="0"
                        max="1"
                        step="0.01"
                        value={effect.data.pedal_position}
                        onChange={handleSliderChange}
                        onClick={(e) => e.stopPropagation()}
                        style={{
                            position: 'absolute',
                            top: 0,
                            left: 0,
                            width: '100%',
                            height: 'calc(100% - 34px)',
                            opacity: 0,
                            zIndex: 10,
                            margin: 0,
                            padding: 0,
                            borderRadius: '12px',

                            // FORCE VERTICAL DRAGGING LAYOUT
                            WebkitAppearance: 'slider-vertical', // Support for older WebKit behaviors
                            writingMode: 'vertical-lr',          // Standard modern vertical layout
                            direction: 'rtl',                     // Ensures dragging UP increases value (0 at bottom, 1 at top)
                            cursor: 'ns-resize',
                        }}
                        title="Drag up or down to adjust wah sweep"
                    />
                </Box>
            </Tooltip>
        </Stack>
    );
}