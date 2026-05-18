import {HcDistortionDto} from "../../domain";
import {Knob} from "../selection/Knob.tsx";

interface HCDistortionProps {
    data: HcDistortionDto;
    handlers: any; // Replace 'any' with your actual handlers type
}

export const HCDistortionControls = ({ data, handlers }: HCDistortionProps) => {
    const THRESHOLD_CLEAN = 1.0;
    const THRESHOLD_HOT = 0.05;
    const driveKnobValue = (1 - (data.threshold - THRESHOLD_HOT) / (THRESHOLD_CLEAN - THRESHOLD_HOT)) * 100;
    const levelKnobValue = data.level * 100;

    return (
        <>
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
        </>
    );
};