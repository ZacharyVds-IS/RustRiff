import {Box} from "@mui/material";
import {Knob} from "./selection/Knob.tsx";

interface EffectPedalProps {
    mainColor: string;
}

export function EffectPedalPreview({ mainColor }: EffectPedalProps) {
    return (
        <Box
            sx={{
                width: 48,
                height: 85,
                display: 'flex',
                flexDirection: 'column',
                alignItems: 'center',
                position: 'relative',
                filter: 'drop-shadow(0 4px 6px rgba(0,0,0,0.5))',
            }}
        >
            <Box
                sx={{
                    width: '100%',
                    height: '55%',
                    background: mainColor,
                    borderRadius: '4px 4px 0 0',
                    border: '1px solid rgba(0,0,0,0.3)',
                    display: 'flex',
                    flexDirection: 'column',
                    alignItems: 'center',
                    pt: 1,
                    zIndex: 2
                }}
            >
                <Box
                    sx={{
                        width: 5,
                        height: 5,
                        borderRadius: '50%',
                        bgcolor: '#ff0000',
                        boxShadow: '0 0 4px #ff0000',
                        mb: 1
                    }}
                />
                <Box sx={{ display: 'flex', flexDirection: 'row', alignItems: 'center' }}>

                    <Knob
                        key={"placeholder-1"}
                        label={""}
                        value={0}
                        size={6}
                        disabled
                    />
                    <Knob
                        key={"placeholder-2"}
                        label={" "}
                        value={0}
                        size={6}
                        disabled
                    />
                </Box>
            </Box>
            <Box
                sx={{
                    width: 'calc(100% + 4px)',
                    height: '45%',
                    bgcolor: '#2d2d2d',
                    borderRadius: '2px 2px 4px 4px',
                    border: '1px solid #1a1a1a',
                    boxShadow: `
                        inset 0 1px 1px rgba(255,255,255,0.1),
                        0 2px 4px rgba(0,0,0,0.4)
                    `,
                    position: 'relative',
                    zIndex: 3,
                    transition: 'transform 0.05s ease-in-out',
                    display: 'flex',
                    justifyContent: 'center',
                    alignItems: 'flex-end',
                    pb: 0.5,
                }}
            />
        </Box>
    );
}