import {Box, Stack, Typography} from "@mui/material";
import {Knob} from "./selection/Knob.tsx";
import {useAmpStore} from "../state/AmpConfigStore.tsx";
import {FlipSwitch} from "./selection/FlipSwitch.tsx";
import {useUIStore} from "../state/UIStore.tsx";
import {
    type AlgorithmicLatencyDto,
    type BufferLatencyDto,
    type ExecutionTimingDto,
    measureAllDspAlgorithmicLatency,
    measureAllDspTimings,
    measureBufferLatency
} from "../domain";
import {useEffect, useState} from "react";

export function EffectControls() {
    const volume = useAmpStore((state) => state.master_volume);
    const gain = useAmpStore((state) => state.gain);
    const isActive = useAmpStore((state) => state.is_active);

    const setVolume = useAmpStore((state) => state.setVolume);
    const setGain = useAmpStore((state) => state.setGain);
    const setIsActive = useAmpStore((state) => state.setIsActive);

    const setBass = useAmpStore((state) => state.setBass);
    const setMiddle = useAmpStore((state) => state.setMiddle);
    const setTreble = useAmpStore((state) => state.setTreble);

    const showLatencyImpacts = useUIStore((state) => state.showLatencyImpacts);
    const [latency, setLatency] = useState<AlgorithmicLatencyDto[]>([]);
    const [bufferLatency, setBufferLatency] = useState<BufferLatencyDto | null>(null);
    const [cpuTimings, setCpuTimings] = useState<ExecutionTimingDto[]>([]);

    useEffect(() => {
        if (showLatencyImpacts) {
            const fetchTimings = async () => {
                try {
                    const [algorithmicLatency, systemBufferLatency, dspCpuTimings] = await Promise.all([
                        measureAllDspAlgorithmicLatency(),
                        measureBufferLatency(),
                        measureAllDspTimings(),
                    ]);
                    setLatency(algorithmicLatency);
                    setBufferLatency(systemBufferLatency);
                    setCpuTimings(dspCpuTimings);
                } catch (error) {
                    console.error("Failed to fetch latency metrics:", error);
                }
            };
            fetchTimings();
        } else {
            setBufferLatency(null);
            setCpuTimings([]);
        }
    }, [showLatencyImpacts]);

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
                display: 'inline-block',
                border: '1px solid',
                borderColor: 'divider',
                boxShadow: 8
            }}
        >
            {showLatencyImpacts && bufferLatency && (
                <Stack direction={"row"} sx={{pb: 2}}>
                    <Typography
                        variant="caption"
                        sx={{
                            mt: 1.25,
                            display: "block",
                            fontSize: "0.62rem",
                            color: "text.secondary",
                            textAlign: "center"
                        }}
                    >
                        Estimated buffer latency: in {bufferLatency.input_buffer_latency_ms.toFixed(3)} ms /
                        out {bufferLatency.output_buffer_latency_ms.toFixed(3)} ms /
                        total {bufferLatency.total_buffer_latency_ms.toFixed(3)} ms
                    </Typography>
                </Stack>
            )}
            <Stack direction="row" spacing={4}>
                <Box sx={{display: "flex", flexDirection: "column", alignItems: "center", gap: 0.5}}>
                    <FlipSwitch label={"On/Off"} value={isActive} onChange={setIsActive}/>
                </Box>

                <Stack>
                    <Box sx={{display: "flex", flexDirection: "column", alignItems: "center", gap: 0.5}}>
                        <Knob
                            label="Volume"
                            value={volume}
                            min={0}
                            max={11}
                            step={1}
                            onChange={setVolume}
                        />

                    </Box>
                    {showLatencyImpacts && (
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

                <Box sx={{display: "flex", flexDirection: "column", alignItems: "center", gap: 0.5}}>
                    <Knob
                        label="Gain"
                        min={0}
                        max={11}
                        step={0.1}
                        value={gain}
                        onChange={setGain}
                    />
                    {showLatencyImpacts && (
                        <Stack spacing={0}>
                            <Typography variant="caption" sx={{fontSize: "0.62rem", color: "text.secondary"}}>
                                latency: {getTimingValue("Gain")}
                            </Typography>
                            <Typography variant="caption" sx={{fontSize: "0.62rem", color: "text.secondary"}}>
                                cpu: {getCpuTimeValue("Gain")}
                            </Typography>
                        </Stack>
                    )}
                </Box>

                <Stack>
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
                    </Box>
                    {showLatencyImpacts && (
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
                </Stack>

            </Stack>
        </Box>
    );
}