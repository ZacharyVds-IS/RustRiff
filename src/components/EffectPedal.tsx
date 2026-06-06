import {Box, IconButton, Stack, Tooltip} from "@mui/material";
import chroma from "chroma-js";
import {
    DelayDto,
    EffectDto,
    HcDistortionDto,
    ScDistortionDto,
    setDelayDelayTime,
    setDelayLevel,
    setHcDistortionLevel,
    setHcDistortionThreshold,
    setScDistortionLevel,
    setScDistortionSmoothing,
    setScDistortionThreshold,
} from "../domain";
import {useAmpStore} from "../state/AmpConfigStore.tsx";
import {HCDistortionControls} from "./pedals/HCDistortionControls.tsx";
import {SCDistortionControls} from "./pedals/SCDistortionControls.tsx";
import {DelayControls} from "./pedals/DelayControls.tsx";
import SettingsInputHdmiIcon from '@mui/icons-material/SettingsInputHdmi';
import {MidiBindingDialog} from "./dialogs/MidiBindingDialog/MidiBindingDialog.tsx";
import {PedalChassis} from "./PedalChassis.tsx";
import {FootswitchButton} from "./FootswitchButton.tsx";
import {useEffectToggle} from "../hooks/useEffectToggle.ts";
import {useMidiModal} from "../hooks/useMidiModal.ts";

interface EffectPedalProps {
    effect: EffectDto;
    onToggle?: (effectId: string, isActive: boolean) => void;
}

export interface EffectHandlers {
    onThresholdChange: (effectId: string, threshold: number, previousThreshold: number) => void;
    onLevelChange: (effectId: string, level: number, previousLevel: number) => void;
    onDelayTimeChange: (effectId: string, delayTime: number, previousDelayTime: number) => void;
    onSmoothingChange: (effectId: string, smoothing: number, previousSmoothing: number) => void;
}

function effectKnobs(effect: EffectDto): React.ReactNode {
    switch (effect.kind) {
        case "HCDistortion":
            return <HCDistortionControls
                data={effect.data as HcDistortionDto}
                handlers={hcDistortionHandlers()}
            />;

        case "SCDistortion":
            return <SCDistortionControls
                data={effect.data as ScDistortionDto}
                handlers={scDistortionHandlers()}
            />;

        case "Delay":
            return <DelayControls
                data={effect.data as DelayDto}
                handlers={delayHandlers()}
            />;

        default:
            return null;
    }
}

function hcDistortionHandlers(): EffectHandlers {
    return {
        onThresholdChange(id, threshold, previousThreshold) {
            const {updateHcDistortionParams} = useAmpStore.getState();
            updateHcDistortionParams(id, {threshold});
            void setHcDistortionThreshold({effectId: id, threshold}).catch(() => {
                updateHcDistortionParams(id, {threshold: previousThreshold});
            });
        },
        onLevelChange(id, level, previousLevel) {
            const {updateHcDistortionParams} = useAmpStore.getState();
            updateHcDistortionParams(id, {level});
            void setHcDistortionLevel({effectId: id, level}).catch(() => {
                updateHcDistortionParams(id, {level: previousLevel});
            });
        },
        onDelayTimeChange: () => {},
        onSmoothingChange: () => {},
    };
}

function scDistortionHandlers(): EffectHandlers {
    return {
        onThresholdChange(id, threshold, previousThreshold) {
            const {updateScDistortionParams} = useAmpStore.getState();
            updateScDistortionParams(id, {threshold});
            void setScDistortionThreshold({effectId: id, threshold}).catch(() => {
                updateScDistortionParams(id, {threshold: previousThreshold});
            });
        },
        onLevelChange(id, level, previousLevel) {
            const {updateScDistortionParams} = useAmpStore.getState();
            updateScDistortionParams(id, {level});
            void setScDistortionLevel({effectId: id, level}).catch(() => {
                updateScDistortionParams(id, {level: previousLevel});
            });
        },
        onSmoothingChange(id, smoothing, previousSmoothing) {
            const {updateScDistortionParams} = useAmpStore.getState();
            updateScDistortionParams(id, {smoothing});
            void setScDistortionSmoothing({effectId: id, smoothing}).catch(() => {
                updateScDistortionParams(id, {smoothing: previousSmoothing});
            });
        },
        onDelayTimeChange: () => {},
    };
}

function delayHandlers(): EffectHandlers {
    return {
        onLevelChange(id, level, previousLevel) {
            const {updateDelayParams} = useAmpStore.getState();
            updateDelayParams(id, {level});
            void setDelayLevel({effectId: id, level}).catch(() => {
                updateDelayParams(id, {level: previousLevel});
            });
        },
        onDelayTimeChange(id, delayTime, previousDelayTime) {
            const sanitizedTime = Math.round(delayTime);
            const {updateDelayParams} = useAmpStore.getState();
            updateDelayParams(id, {delay_time: sanitizedTime});
            void setDelayDelayTime({effectId: id, delayTime}).catch(() => {
                updateDelayParams(id, {delay_time: previousDelayTime});
            });
        },
        onThresholdChange: () => {},
        onSmoothingChange: () => {},
    };
}

export function EffectPedal({effect, onToggle}: EffectPedalProps) {
    const {isActive, handleToggle} = useEffectToggle(effect.data.id, effect.data.is_active, onToggle);
    const {midiModalOpen, openMidiModal, closeMidiModal} = useMidiModal();

    const chassisColor = chroma(effect.data.color).hex();
    const textColor = chroma.contrast(chassisColor, "#111111") >= 4.5
        ? "rgba(0, 0, 0, 0.84)"
        : "rgba(255, 255, 255, 0.94)";
    const textShadow = chroma(textColor).luminance() > 0.5
        ? "0 1px 2px rgba(0,0,0,0.45)"
        : "0 1px 2px rgba(255,255,255,0.2)";

    return (
        <>
            <Stack direction="column" sx={{alignItems: "center"}}>
                <Tooltip title="MIDI Mapping" arrow placement="top">
                    <IconButton aria-label="midi config" size="small" sx={{mb: 0.5}} onClick={openMidiModal}>
                        <SettingsInputHdmiIcon/>
                    </IconButton>
                </Tooltip>

                <PedalChassis
                    chassisColor={chassisColor}
                    isActive={isActive}
                    textColor={textColor}
                    textShadow={textShadow}
                    effectName={effect.data.name}
                    knobs={effectKnobs(effect)}
                />
                <Box sx={{mt: '-6px', zIndex: 3, width: 180}}>
                    <FootswitchButton onClick={handleToggle}/>
                </Box>
            </Stack>

            <MidiBindingDialog
                open={midiModalOpen}
                onClose={closeMidiModal}
                effectId={effect.data.id}
                effectName={effect.data.name}
                effectKind={effect.kind}
            />
        </>
    );
}
