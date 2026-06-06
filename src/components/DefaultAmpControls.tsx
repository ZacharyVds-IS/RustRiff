import {Box, Stack, Typography} from "@mui/material";
import {Knob} from "./selection/Knob.tsx";
import {useAmpStore} from "../state/AmpConfigStore.tsx";
import {FlipSwitch} from "./selection/FlipSwitch.tsx";
import {useUIStore} from "../state/UIStore.tsx";
import {useDspMetrics} from "../hooks/useDspMetrics.ts";
import {AmpKnobControl} from "./AmpKnobControl.tsx";

export function DefaultAmpControls() {
    const activeChannel = useAmpStore((state) =>
        state.channels.find((c) => c.id === state.current_channel)
    );

    const volume = activeChannel?.volume ?? 0;
    const gain = activeChannel?.gain ?? 0;
    const bass = activeChannel?.tone_stack.bass ?? 0;
    const middle = activeChannel?.tone_stack.middle ?? 0;
    const treble = activeChannel?.tone_stack.treble ?? 0;

    const bassUi = bass * 100;
    const middleUi = middle * 100;
    const trebleUi = treble * 100;

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
    const {getTimingValue, getCpuTimeValue} = useDspMetrics(developerMode);

    return (
        <Box
            sx={{
                p: 4,
                bgcolor: 'background.paper',
                borderRadius: 4,
                border: '1px solid',
                borderColor: 'divider',
                boxShadow: 8,
                width: 'fit-content'
            }}
        >
            <Stack direction="row" spacing={4} sx={{alignItems: 'center'}}>
                <FlipSwitch label="On/Off" value={isActive} onChange={setIsActive}/>

                <AmpKnobControl
                    label="Volume" value={volume} max={11} step={1} onChange={setVolume}
                    devLatency={developerMode ? getTimingValue("Volume") : undefined}
                    devCpu={developerMode ? getCpuTimeValue("Volume") : undefined}
                />

                <AmpKnobControl
                    label="Gain" min={0} max={11} step={0.1} value={gain} onChange={setGain}
                    devLatency={developerMode ? getTimingValue("Gain") : undefined}
                    devCpu={developerMode ? getCpuTimeValue("Gain") : undefined}
                />

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
                            <Knob label="Bass" min={0} max={100} value={bassUi} size={50}
                                  onChange={(val) => setBass(val / 100)}/>
                            <Knob label="Middle" min={0} max={100} value={middleUi} size={50}
                                  onChange={(val) => setMiddle(val / 100)}/>
                            <Knob label="Treble" min={0} max={100} value={trebleUi} size={50}
                                  onChange={(val) => setTreble(val / 100)}/>
                        </Stack>
                    </Box>
                    {developerMode && (
                        <Stack spacing={0}>
                            <Typography variant="caption" sx={{
                                fontSize: "0.62rem", color: "text.secondary", mt: 1,
                                display: "block", textAlign: "center"
                            }}>
                                latency: {getTimingValue("Tone Stack")}
                            </Typography>
                            <Typography variant="caption" sx={{
                                fontSize: "0.62rem", color: "text.secondary",
                                display: "block", textAlign: "center"
                            }}>
                                cpu: {getCpuTimeValue("Tone Stack")}
                            </Typography>
                        </Stack>
                    )}
                </Stack>

                <AmpKnobControl
                    label="Master" min={0} max={11} step={1} value={masterVolume} onChange={setMasterVolume}
                    devLatency={developerMode ? getTimingValue("Master Volume") : undefined}
                    devCpu={developerMode ? getCpuTimeValue("Master Volume") : undefined}
                />
            </Stack>
        </Box>
    );
}
