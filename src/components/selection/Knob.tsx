import React, {useEffect, useState} from "react";
import {Box, Typography, useTheme} from "@mui/material";

interface KnobProps {
    label: string;
    value?: number;
    min?: number;
    max?: number;
    step?: number;
    size?: number;
    onChange?: (newValue: number) => void;
    disabled?: boolean;
}

export function Knob({
                         label,
                         value = 0,
                         min = 0,
                         max = 100,
                         step = 1,
                         size = 60,
                         onChange,
                         disabled = false
                     }: KnobProps) {
    const [localValue, setLocalValue] = useState(value);
    const theme = useTheme();

    useEffect(() => {
        setLocalValue(value);
    }, [value]);
    const percentage = (localValue - min) / (max - min);
    const rotation = percentage * 270 - 135;

    const handleMouseDown = (e: React.MouseEvent<HTMLDivElement>) => {
        if (disabled) return;
        const startY = e.clientY;
        const startValue = localValue;
        const sensitivity = 200;
        const handleMouseMove = (moveEvent: MouseEvent) => {
            const deltaY = startY - moveEvent.clientY;

            const range = max - min;
            const change = (deltaY / sensitivity) * range;

            let newValue = startValue + change;

            if (step > 0) {
                newValue = Math.round(newValue / step) * step;
            }

            const clampedValue = Math.min(Math.max(newValue, min), max);

            if (clampedValue !== localValue) {
                setLocalValue(clampedValue);
                if (onChange) onChange(clampedValue);
            }
        };

        const handleMouseUp = () => {
            window.removeEventListener("mousemove", handleMouseMove);
            window.removeEventListener("mouseup", handleMouseUp);
        };

        window.addEventListener("mousemove", handleMouseMove);
        window.addEventListener("mouseup", handleMouseUp);
    };

    return (
        <Box
            sx={{
                display: 'flex',
                flexDirection: 'column',
                alignItems: 'center',
                width: size + 20,
                userSelect: 'none'
            }}
        >
            <Typography
                variant="caption"
                sx={{
                    color: 'text.primary',
                    mb: 1,
                    fontWeight: 600,
                    fontSize: '0.65rem',
                    textTransform: 'uppercase'
                }}
            >
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
                <Box
                    sx={{
                        position: 'absolute',
                        top: '10%',
                        width: '4px',
                        height: '20%',
                        bgcolor: 'common.black',
                        borderRadius: '2px'
                    }}
                />
            </Box>

            {!disabled &&
                <Typography
                    sx={{
                        fontSize: '0.6rem',
                        mt: 0.5,
                        color: 'text.secondary',
                        fontFamily: 'monospace'
                    }}
                >
                    {step < 1 ? localValue.toFixed(1) : Math.round(localValue)}
                </Typography>
            }
        </Box>
    );
}