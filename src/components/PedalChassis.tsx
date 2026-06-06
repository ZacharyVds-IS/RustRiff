import {Box, Typography} from "@mui/material";
import chroma from "chroma-js";

interface PedalChassisProps {
    chassisColor: string;
    isActive: boolean;
    textColor: string;
    textShadow: string;
    effectName: string;
    knobs: React.ReactNode;
}

export function PedalChassis({chassisColor, isActive, textColor, textShadow, effectName, knobs}: PedalChassisProps) {
    return (
        <Box
            sx={{
                width: 180,
                minHeight: 280,
                display: 'flex',
                flexDirection: 'column',
                alignItems: 'center',
                position: 'relative',
                filter: (theme) => theme.palette.mode === 'dark'
                    ? 'drop-shadow(0 12px 24px rgba(255, 255, 255, 0.3))'
                    : 'drop-shadow(0 6px 12px rgba(0,0,0,0.4))',
            }}
        >
            <Box
                sx={{
                    width: '100%',
                    height: 'auto',
                    flexGrow: 1,
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
                <Box
                    sx={{
                        width: 8,
                        height: 8,
                        borderRadius: '50%',
                        border: '1px solid rgba(0,0,0,0.3)',
                        bgcolor: isActive ? '#00ff00' : '#ff0000',
                        boxShadow: isActive ? '0 0 6px #00ff00' : '0 0 6px #ff0000',
                        mb: 2,
                        transition: 'background-color 0.1s, box-shadow 0.1s',
                    }}
                />

                <Box sx={{color: textColor, textShadow}}>
                    {knobs}
                </Box>

                <Typography
                    sx={{
                        mt: 'auto',
                        mb: 2,
                        fontWeight: 900,
                        fontSize: '1.2rem',
                        color: textColor,
                        textShadow,
                        textTransform: 'uppercase',
                        fontStyle: 'italic'
                    }}
                    noWrap
                >
                    {effectName}
                </Typography>
            </Box>
        </Box>
    );
}
