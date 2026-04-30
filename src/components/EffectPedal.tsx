import {Box, Stack, Typography} from "@mui/material";
import chroma from "chroma-js";
import {Knob} from "./selection/Knob.tsx";

interface EffectPedalProps {
    mainColor: string;
    name: string;
}

export function EffectPedal({ mainColor, name }: EffectPedalProps) {
    const chassisColor = chroma(mainColor).hex();

    return (
        <Box
            sx={{
                width: 180,
                height: 280,
                display: 'flex',
                flexDirection: 'column',
                alignItems: 'center',
                position: 'relative',
                filter: 'drop-shadow(0 6px 12px rgba(0,0,0,0.4))',
            }}
        >
            {/* Top Chassis */}
            <Box
                sx={{
                    width: '100%',
                    height: '60%',
                    background: `linear-gradient(180deg, ${chroma(chassisColor).brighten(0.3)}, ${chassisColor})`,
                    borderRadius: '6px 6px 0 0',
                    border: '1px solid rgba(0,0,0,0.4)',
                    display: 'flex',
                    flexDirection: 'column',
                    alignItems: 'center',
                    pt: 2,
                    zIndex: 2
                }}
            >
                {/* Check LED */}
                <Box
                    sx={{
                        width: 8,
                        height: 8,
                        borderRadius: '50%',
                        bgcolor: '#ff0000',
                        boxShadow: '0 0 6px #ff0000',
                        mb: 2
                    }}
                />

                {/* Knobs Row */}
                <Stack direction="row" spacing={1} sx={{ justifyContent: 'center' }}>
                    <Knob label="Level" value={50} size={40} disabled />
                    <Knob label="Tone" value={50} size={40} disabled />
                    <Knob label="Dist" value={50} size={40} disabled />
                </Stack>

                <Typography
                    sx={{
                        mt: 'auto',
                        mb: 2,
                        fontWeight: 900,
                        fontSize: '1.2rem',
                        color: 'rgba(0,0,0,0.7)',
                        textTransform: 'uppercase',
                        fontStyle: 'italic'
                    }}
                >
                    {name}
                </Typography>
            </Box>

            {/* Wider Boss-Style Footswitch */}
            <Box
                sx={{
                    width: 'calc(100% + 8px)', // Slightly wider than chassis
                    height: '40%',
                    bgcolor: '#1a1a1a', // Black rubber pad
                    borderRadius: '2px 2px 8px 8px',
                    border: '2px solid #000',
                    boxShadow: 'inset 0 2px 4px rgba(255,255,255,0.1)',
                    display: 'flex',
                    justifyContent: 'center',
                    alignItems: 'flex-end',
                    pb: 1,
                    cursor: 'pointer',
                    zIndex: 3,
                    transition: 'transform 0.05s',
                    '&:active': {
                        transform: 'scale(0.98) translateY(2px)'
                    }
                }}
            >
                {/* Thumb Screw Detail */}
                <Box
                    sx={{
                        width: 12,
                        height: 12,
                        borderRadius: '50%',
                        background: 'radial-gradient(circle, #444, #000)',
                        border: '1px solid #333'
                    }}
                />
            </Box>
        </Box>
    );
}