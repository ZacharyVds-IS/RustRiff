import React from "react";
import {Box, Typography, useTheme} from "@mui/material";
import {useKnobDrag} from "../../hooks/useKnobDrag.ts";

interface KnobProps {
    label: string;
    value?: number;
    min?: number;
    max?: number;
    step?: number;
    size?: number;
    onChange?: (newValue: number) => void;
    disabled?: boolean;
    valueDisplay?: "numeric" | "min-max";
}

export function Knob({
                         label,
                         value = 0,
                         min = 0,
                         max = 100,
                         step = 1,
                         size = 60,
                         onChange,
                         disabled = false,
                         valueDisplay = "numeric",
                     }: KnobProps) {
    const {localValue, rotation, handleMouseDown} = useKnobDrag({value, min, max, step, onChange, disabled});
    const theme = useTheme();

    const percentage = (localValue - min) / (max - min);

    const minMaxLabel = (() => {
        if (percentage <= 0.02) return "MIN";
        if (percentage >= 0.98) return "MAX";
        return null;
    })();

    return (
        <Box sx={{display: 'flex', flexDirection: 'column', alignItems: 'center', width: size + 20, userSelect: 'none'}}>
            <Typography variant="caption" sx={{
                color: 'inherit', mb: 1, fontWeight: 600, fontSize: '0.65rem', textTransform: 'uppercase'
            }}>
                {label}
            </Typography>

            <Box
                onMouseDown={handleMouseDown}
                sx={{
                    width: size,
                    height: size,
                    borderRadius: '50%',
                    bgcolor: 'grey.300',
                    border: '3px solid',
                    borderColor: 'background.default',
                    position: 'relative',
                    display: 'flex',
                    justifyContent: 'center',
                    transform: `rotate(${rotation}deg)`,
                    cursor: 'ns-resize',
                    boxShadow: theme.shadows[4],
                    transition: 'transform 0.05s linear',
                    '&:active': {cursor: 'grabbing'}
                }}
            >
                <Box sx={{
                    position: 'absolute', top: '10%', width: '4px', height: '20%',
                    bgcolor: 'common.black', borderRadius: '2px'
                }}/>
            </Box>

            {!disabled && (
                <Typography sx={{fontSize: '0.6rem', mt: 0.5, color: 'inherit', opacity: 0.85, fontFamily: 'monospace'}}>
                    {valueDisplay === "min-max"
                        ? (minMaxLabel ?? "·")
                        : (step < 1 ? localValue.toFixed(1) : Math.round(localValue))
                    }
                </Typography>
            )}
        </Box>
    );
}
