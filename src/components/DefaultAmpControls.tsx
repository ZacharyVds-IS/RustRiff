import {Box, Stack, Typography} from "@mui/material";
import {Knob} from "./selection/Knob.tsx";
import {useAmpStore} from "../state/AmpConfigStore.tsx";
import {FlipSwitch} from "./selection/FlipSwitch.tsx";
import {useUIStore} from "../state/UIStore.tsx";
import {
    type AlgorithmicLatencyDto, type ExecutionTimingDto,
    measureAllDspAlgorithmicLatency,
    measureAllDspTimings
} from "../domain";
import {useEffect, useState} from "react";

export function DefaultAmpControls() {

    const activeChannel = useAmpStore((state) =>
        state.channels.find((c) => c.id === state.current_channel)
    );

    const volume = activeChannel?.volume ?? 0;
    const gain = activeChannel?.gain ?? 0;
    const bass = activeChannel?.tone_stack.bass ?? 0;
    const middle = activeChannel?.tone_stack.middle ?? 0;
    const treble = activeChannel?.tone_stack.treble ?? 0;

    const masterVolume = useAmpStore((state) => state.master_volume);
    const isActive = useAmpStore((state) => state.is_active);

    const setVolume = useAmpStore((state) => state.setVolume);
    const setMasterVolume = useAmpStore((state) => state.setMasterVolume);
    const setGain = useAmpStore((state) => state.setGain);
    const setIsActive = useAmpStore((state) => state.setIsActive);

    const setBass = useAmpStore((state) => state.setBass);
    const setMiddle = useAmpStore((state) => state.setMiddle);
    const setTreble = useAmpStore((state) => state.setTreble);

    const developerMode = useUIStore((state) => state.developerMode);
    const [latency, setLatency] = useState<AlgorithmicLatencyDto[]>([]);
    const [cpuTimings, setCpuTimings] = useState<ExecutionTimingDto[]>([]);

    useEffect(() => {
        if (developerMode) {
            const fetchTimings = async () => {
                try {
                    const promises: Promise<any>[] = [
                        measureAllDspAlgorithmicLatency(),
                        measureAllDspTimings()
                    ];

                    const results = await Promise.all(promises);
                    setLatency(results[0] || []);
                    setCpuTimings(results[1] || []);
                } catch (error) {
                    console.error("Failed to fetch latency metrics:", error);
                }
            };
            fetchTimings();
        } else {
            setLatency([]);
            setCpuTimings([]);
        }
    }, [developerMode]);

    const getTimingValue = (processorName: string): string => {
        const timing = latency.find(t => t.processor_name === processorName);
        return timing ? `${timing.latency_ms.toFixed(3)} ms (${timing.latency_samples} samples)` : "-";
    };

    const getCpuTimeValue = (processorName: string): string => {
        const timing = cpuTimings.find(t => t.processor_name === processorName);
        return timing ? `${timing.execution_us_per_sample.toFixed(3)} us/sample` : "-";
    };

    return (
        <Box
            sx={{
                p: 4,
                bgcolor: 'background.paper',
                borderRadius: 4,
                border: '1px solid',
                borderColor: 'divider',
                boxShadow: 8,
                width: 'fit-content' // Keeps the panel tight around controls
            }}
        >
            <Stack direction="row" spacing={4} sx={{alignItems: 'center'}}>
                <FlipSwitch label={"On/Off"} value={isActive} onChange={setIsActive}/>
                <Knob
                    label="Volume"
                    value={volume}
                    min={0}
                    max={11}
                    step={1}
                    onChange={setVolume}
                />
                <Stack>
                    <Knob
                        label="Gain"
                        min={0}
                        max={11}
                        step={0.1}
                        value={gain}
                        onChange={setGain}
                    />
                    {developerMode && (
                        <Stack spacing={0}>
                            <Typography variant="caption" sx={{fontSize: "0.62rem", color: "text.secondary"}}>
                                latency: {getTimingValue("Gain")}
                            </Typography>
                            <Typography variant="caption" sx={{fontSize: "0.62rem", color: "text.secondary"}}>
                                cpu: {getCpuTimeValue("Gain")}
                            </Typography>
                        </Stack>
                    )}
                </Stack>
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
                        <Knob label="Bass" min={0} max={100} value={bass} size={50} onChange={setBass}/>
                        <Knob label="Middle" min={0} max={100} value={middle} size={50} onChange={setMiddle}/>
                        <Knob label="Treble" min={0} max={100} value={treble} size={50} onChange={setTreble}/>
                    </Stack>
                </Box>
                {developerMode && (
                    <Stack spacing={0}>
                        <Typography variant="caption" sx={{
                            fontSize: "0.62rem",
                            color: "text.secondary",
                            mt: 1,
                            display: "block",
                            textAlign: "center"
                        }}>
                            latency: {getTimingValue("Tone Stack")}
                        </Typography>
                        <Typography variant="caption" sx={{
                            fontSize: "0.62rem",
                            color: "text.secondary",
                            display: "block",
                            textAlign: "center"
                        }}>
                            cpu: {getCpuTimeValue("Tone Stack")}
                        </Typography>
                    </Stack>
                )}
                <Stack>
                    <Knob label={"Master"} min={0} max={11} step={1} value={masterVolume} onChange={setMasterVolume}/>
                    {developerMode && (
                        <Stack spacing={0}>
                            <Typography variant="caption" sx={{fontSize: "0.62rem", color: "text.secondary"}}>
                                latency: {getTimingValue("Master Volume")}
                            </Typography>
                            <Typography variant="caption" sx={{fontSize: "0.62rem", color: "text.secondary"}}>
                                cpu: {getCpuTimeValue("Master Volume")}
                            </Typography>
                        </Stack>
                    )}
                </Stack>
            </Stack>
        </Box>
    );
}