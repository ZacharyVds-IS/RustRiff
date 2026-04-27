import {Box, Stack, Typography} from "@mui/material";
import {Knob} from "./selection/Knob.tsx";
import {useAmpStore} from "../state/AmpConfigStore.tsx";
import {FlipSwitch} from "./selection/FlipSwitch.tsx";
import {useUIStore} from "../state/UIStore.tsx";
import {type ExecutionTimingDto, measureAllDspTimings} from "../domain";
import {useEffect, useState} from "react";

export function EffectControls() {
    const volume = useAmpStore((state) => state.master_volume);
    const gain = useAmpStore((state) => state.gain);
    const isActive = useAmpStore((state) => state.is_active);

    const setVolume = useAmpStore((state) => state.setVolume);
    const setGain = useAmpStore((state) => state.setGain);
    const setIsActive = useAmpStore((state) => state.setIsActive);

    const setBass = useAmpStore((state) => state.setBass);
    const setMiddle= useAmpStore((state) => state.setMiddle);
    const setTreble= useAmpStore((state) => state.setTreble);

    const showLatencyImpacts = useUIStore((state) => state.showLatencyImpacts);
    const [timings, setTimings] = useState<ExecutionTimingDto[]>([]);

    useEffect(() => {
        if (showLatencyImpacts) {
            const fetchTimings = async () => {
                try {
                    const result = await measureAllDspTimings();
                    setTimings(result);
                } catch (error) {
                    console.error("Failed to fetch DSP timings:", error);
                }
            };
            fetchTimings();
        }
    }, [showLatencyImpacts]);

    const getTimingValue = (processorName: string): string => {
        const timing = timings.find(t => t.processor_name === processorName);
        return timing ? `${timing.execution_us_per_sample.toFixed(3)} u/s` : "-";
    };

    return (
        <Box
            sx={{
                p: 4,
                bgcolor: 'background.paper',
                borderRadius: 4,
                display: 'inline-block',
                border: '1px solid',
                borderColor: 'divider',
                boxShadow: 8
            }}
        >
            <Stack direction="row" spacing={4}>
                <Box sx={{ display: "flex", flexDirection: "column", alignItems: "center", gap: 0.5 }}>
                    <FlipSwitch label={"On/Off"} value={isActive} onChange={setIsActive}/>
                </Box>

                <Box sx={{ display: "flex", flexDirection: "column", alignItems: "center", gap: 0.5 }}>
                    <Knob
                        label="Volume"
                        value={volume}
                        min={0}
                        max={11}
                        step={1}
                        onChange={setVolume}
                    />
                    {showLatencyImpacts && (
                        <Typography variant="caption" sx={{ fontSize: "0.62rem", color: "text.secondary" }}>
                            {getTimingValue("Input Latency")} / {getTimingValue("Master Volume")} / {getTimingValue("Output Latency")}
                        </Typography>
                    )}
                </Box>

                <Box sx={{ display: "flex", flexDirection: "column", alignItems: "center", gap: 0.5 }}>
                    <Knob
                        label="Gain"
                        min={0}
                        max={11}
                        step={0.1}
                        value={gain}
                        onChange={setGain}
                    />
                    {showLatencyImpacts && (
                        <Typography variant="caption" sx={{ fontSize: "0.62rem", color: "text.secondary" }}>
                            {getTimingValue("Gain")}
                        </Typography>
                    )}
                </Box>

                <Box
                    sx={{
                        border: '1px solid',
                        borderColor: 'divider',
                        p: 2,
                        borderRadius: 2,
                        position: 'relative'
                    }}
                >
                    <Typography
                        sx={{
                            position: 'absolute',
                            top: -10,
                            left: 10,
                            bgcolor: 'background.paper',
                            px: 1,
                            fontSize: '0.7rem',
                            fontWeight: 'bold',
                            color: 'text.secondary',
                            textTransform: 'uppercase',
                            letterSpacing: '0.05rem'
                        }}
                    >
                        Tone stack
                    </Typography>

                    <Stack direction="row" spacing={2}>
                        <Knob label="Bass" min={0} max={100} value={100} size={50} onChange={setBass}/>
                        <Knob label="Middle" min={0} max={100} value={100} size={50} onChange={setMiddle}/>
                        <Knob label="Treble" min={0} max={100} value={100} size={50} onChange={setTreble}/>
                    </Stack>
                    {showLatencyImpacts && (
                        <Typography variant="caption" sx={{ fontSize: "0.62rem", color: "text.secondary", mt: 1, display: "block", textAlign: "center" }}>
                            {getTimingValue("Tone Stack")}
                        </Typography>
                    )}
                </Box>
            </Stack>
        </Box>
    );
}