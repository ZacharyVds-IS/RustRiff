import {WahDto} from "../../domain";
import {Stack} from "@mui/material";
import {Knob} from "../selection/Knob.tsx";
import {EffectHandlers} from "../EffectPedal.tsx";

interface WahControlsProps {
    data: WahDto;
    handlers: EffectHandlers;
}

export const WahControls = ({ data, handlers }: WahControlsProps) => {
    const knobValue = data.pedal_position * 100;

    return (
        <Stack sx={{ width: 120, alignItems: "center" }}>
            <Knob
                label="Pedal"
                value={Math.max(0, Math.min(100, knobValue))}
                min={0}
                max={100}
                step={1}
                size={40}
                valueDisplay="min-max"
                onChange={(v) => {
                    const pedalPos = v / 100;
                    handlers.onPedalPositionChange(data.id, pedalPos, data.pedal_position);
                }}
            />
        </Stack>
    );
};
