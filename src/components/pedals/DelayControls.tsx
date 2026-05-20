import {DelayDto} from "../../domain";
import {Knob} from "../selection/Knob.tsx";
import {EffectHandlers} from "../EffectPedal.tsx";

interface DelayProps {
    data: DelayDto;
    handlers: EffectHandlers;
}

export const DelayControls = ({ data, handlers }: DelayProps) => {
    const MIN_DELAY_MS = 20;
    const MAX_DELAY_MS = 800;
    const levelKnobValue = (data.level / 0.95) * 100;

    return (
        <>
            <Knob
                label="Time"
                value={data.delay_time}
                min={MIN_DELAY_MS}
                max={MAX_DELAY_MS}
                step={1}
                size={40}
                valueDisplay="min-max"
                onChange={(v) => {
                    handlers.onDelayTimeChange(data.id, v, data.delay_time);
                }}
            />
            <Knob
                label="Intensity"
                value={Math.max(0, Math.min(100, levelKnobValue))}
                min={0}
                max={100}
                step={0.5}
                size={40}
                valueDisplay="min-max"
                onChange={(v) => {
                    const level = (v / 100) * 0.95;
                    handlers.onLevelChange(data.id, level, data.level);
                }}
            />
        </>
    );
};