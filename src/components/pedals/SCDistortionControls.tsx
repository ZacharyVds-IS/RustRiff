import {ScDistortionDto} from "../../domain";
import {Stack} from "@mui/material";
import {Knob} from "../selection/Knob.tsx";
import {EffectHandlers} from "../EffectPedal.tsx";

interface SCDistortionProps {
    data: ScDistortionDto;
    handlers: EffectHandlers;
}

export const SCDistortionControls = ({ data, handlers }: SCDistortionProps) => {
    const THRESHOLD_CLEAN = 1.0;
    const THRESHOLD_HOT = 0.05;
    const driveKnobValue = (1 - (data.threshold - THRESHOLD_HOT) / (THRESHOLD_CLEAN - THRESHOLD_HOT)) * 100;
    const levelKnobValue = data.level * 100;

    const SMOOTHING_MIN = 1.0;
    const SMOOTHING_MAX = 10.0;
    const smoothingKnobValue = ((SMOOTHING_MAX - data.smoothing) / (SMOOTHING_MAX - SMOOTHING_MIN)) * 100;

    return (
        <Stack sx={{ width: 200 }}>
            <Stack direction="row" sx={{ justifyContent: "space-around" }}>
                <Knob
                    label="Drive"
                    value={Math.max(0, Math.min(100, driveKnobValue))}
                    min={0}
                    max={100}
                    step={0.5}
                    size={40}
                    valueDisplay="min-max"
                    onChange={(v) => {
                        const threshold = THRESHOLD_CLEAN - (v / 100) * (THRESHOLD_CLEAN - THRESHOLD_HOT);
                        handlers.onThresholdChange(data.id, threshold, data.threshold);
                    }}
                />
                <Knob
                    label="Level"
                    value={Math.max(0, Math.min(100, levelKnobValue))}
                    min={0}
                    max={100}
                    step={0.5}
                    size={40}
                    valueDisplay="min-max"
                    onChange={(v) => {
                        const level = v / 100;
                        handlers.onLevelChange(data.id, level, data.level);
                    }}
                />
            </Stack>
            <Stack sx={{ alignItems: "center" }}>
                <Knob
                    label="Smoothing"
                    value={Math.max(0, Math.min(100, smoothingKnobValue))}
                    min={0}
                    max={100}
                    step={0.5}
                    size={30}
                    valueDisplay="min-max"
                    onChange={(v) => {
                        const smoothing = SMOOTHING_MAX - (v / 100) * (SMOOTHING_MAX - SMOOTHING_MIN);
                        handlers.onSmoothingChange(data.id, smoothing, data.smoothing);
                    }}
                />
            </Stack>
        </Stack>
    );
};