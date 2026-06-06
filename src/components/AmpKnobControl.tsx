import {Stack, Typography} from "@mui/material";
import {Knob} from "./selection/Knob.tsx";

interface AmpKnobControlProps {
    label: string;
    value: number;
    min?: number;
    max?: number;
    step?: number;
    size?: number;
    onChange: (val: number) => void;
    devLatency?: string;
    devCpu?: string;
}

export function AmpKnobControl({label, value, min, max, step, size, onChange, devLatency, devCpu}: AmpKnobControlProps) {
    return (
        <Stack>
            <Knob
                label={label}
                value={value}
                min={min ?? 0}
                max={max ?? 11}
                step={step ?? 1}
                size={size}
                onChange={onChange}
            />
            {devLatency && (
                <Stack spacing={0}>
                    <Typography variant="caption" sx={{fontSize: "0.62rem", color: "text.secondary"}}>
                        latency: {devLatency}
                    </Typography>
                    <Typography variant="caption" sx={{fontSize: "0.62rem", color: "text.secondary"}}>
                        cpu: {devCpu}
                    </Typography>
                </Stack>
            )}
        </Stack>
    );
}
