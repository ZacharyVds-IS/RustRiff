import {Box, IconButton, Stack, Tooltip, Typography} from "@mui/material";
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
    toggleEffect,
} from "../domain";
import {useEffect, useState} from "react";
import {useAmpStore} from "../state/AmpConfigStore.tsx";
import {HCDistortionControls} from "./pedals/HCDistortionControls.tsx";
import {SCDistortionControls} from "./pedals/SCDistortionControls.tsx";
import {DelayControls} from "./pedals/DelayControls.tsx";
import SettingsInputHdmiIcon from '@mui/icons-material/SettingsInputHdmi';
import {MidiBindingDialog} from "./dialogs/MidiBindingDialog/MidiBindingDialog.tsx";

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

function knobsForEffect(
    effect: EffectDto,
    handlers: EffectHandlers
): React.ReactNode {
    switch (effect.kind) {
        case "HCDistortion":
            return <HCDistortionControls data={effect.data as HcDistortionDto} handlers={handlers}/>;

        case "SCDistortion":
            return <SCDistortionControls data={effect.data as ScDistortionDto} handlers={handlers}/>;

        case "Delay":
            return <DelayControls data={effect.data as DelayDto} handlers={handlers}/>;

        default:
            return null;
    }
}

export function EffectPedal({effect, onToggle}: EffectPedalProps) {
    const [isActive, setIsActive] = useState(effect.data.is_active);
    const [midiModalOpen, setMidiModalOpen] = useState(false);

    const updateEffectActiveState = useAmpStore((state) => state.updateEffectActiveState);
    const updateHcDistortionParams = useAmpStore((state) => state.updateHcDistortionParams);
    const updateScDistortionParams = useAmpStore((state) => state.updateScDistortionParams);
    const updateDelayParams = useAmpStore((state) => state.updateDelayParams);
    const chassisColor = chroma(effect.data.color).hex();
    const textColor = chroma.contrast(chassisColor, "#111111") >= 4.5
        ? "rgba(0, 0, 0, 0.84)"
        : "rgba(255, 255, 255, 0.94)";
    const textShadow = chroma(textColor).luminance() > 0.5
        ? "0 1px 2px rgba(0,0,0,0.45)"
        : "0 1px 2px rgba(255,255,255,0.2)";

    useEffect(() => {
        setIsActive(effect.data.is_active);
    }, [effect.data.id, effect.data.is_active]);

    async function handleFootswitchClick() {
        try {
            const newActive = await toggleEffect({effectId: effect.data.id});
            setIsActive(newActive);
            updateEffectActiveState(effect.data.id, newActive);
            onToggle?.(effect.data.id, newActive);
        } catch (error) {
            console.error(`Failed to toggle effect ${effect.data.id}:`, error);
        }
    }

    function handleHCThresholdChange(effectId: string, threshold: number, previousThreshold: number) {
        updateHcDistortionParams(effectId, {threshold});
        void setHcDistortionThreshold({effectId, threshold}).catch((error) => {
            console.error("Failed to update HC distortion threshold:", error);
            updateHcDistortionParams(effectId, {threshold: previousThreshold});
        });
    }

    function handleSCThresholdChange(effectId: string, threshold: number, previousThreshold: number) {
        updateScDistortionParams(effectId, {threshold});
        void setScDistortionThreshold({effectId, threshold}).catch((error) => {
            console.error("Failed to update SC distortion threshold:", error);
            updateScDistortionParams(effectId, {threshold: previousThreshold});
        });
    }

    function handleHCDLevelChange(effectId: string, level: number, previousLevel: number) {
        updateHcDistortionParams(effectId, {level});
        void setHcDistortionLevel({effectId, level}).catch((error) => {
            console.error("Failed to update HC distortion level:", error);
            updateHcDistortionParams(effectId, {level: previousLevel});
        });
    }

    function handleSCLevelChange(effectId: string, level: number, previousLevel: number) {
        updateScDistortionParams(effectId, {level: level});
        void setScDistortionLevel({effectId, level}).catch((error) => {
            console.error("Failed to update SC distortion level:", error);
            updateScDistortionParams(effectId, {level: previousLevel});
        });
    }

    function handleSmoothingChange(effectId: string, smoothing: number, previousSmoothing: number) {
        updateScDistortionParams(effectId, {smoothing: smoothing});
        void setScDistortionSmoothing({effectId, smoothing}).catch((error) => {
            console.error("Failed to update SC smoothing:", error);
            updateScDistortionParams(effectId, {smoothing: previousSmoothing});
        });
    }

    function handleDelayLevelChange(effectId: string, level: number, previousLevel: number) {
        updateDelayParams(effectId, {level: level});
        void setDelayLevel({effectId, level}).catch((error) => {
            console.error("Failed to update Delay level:", error);
            updateDelayParams(effectId, {level: previousLevel});
        });
    }

    function handleDelayTimeChange(effectId: string, delayTime: number, previousDelayTime: number) {
        const sanitizedTime = Math.round(delayTime);
        updateDelayParams(effectId, {delay_time: sanitizedTime});
        void setDelayDelayTime({effectId, delayTime}).catch((error) => {
            console.error("Failed to update Delay delay time:", error);
            updateDelayParams(effectId, {delay_time: previousDelayTime});
        });
    }


    return (
        <>
            <Stack direction={"column"} sx={{alignItems: "center"}}>
                <Tooltip title="MIDI Mapping" arrow placement="top">
                    <IconButton
                        aria-label="midi config"
                        size="small"
                        sx={{ mb: 0.5 }}
                        onClick={() => setMidiModalOpen(true)}
                    >
                        <SettingsInputHdmiIcon />
                    </IconButton>
                </Tooltip>

                <Box
                    sx={{
                        width: 180,
                        minHeight: 280,
                        display: 'flex',
                        flexDirection: 'column',
                        alignItems: 'center',
                        position: 'relative',
                        filter: (theme) => theme.palette.mode === 'dark'
                            ? 'drop-shadow(0 12px 24px rgba(255, 255, 255, 0.3))'
                            : 'drop-shadow(0 6px 12px rgba(0,0,0,0.4))',
                    }}
                >
                    <Box
                        sx={{
                            width: '100%',
                            height: 'auto',
                            flexGrow: 1,
                            background: `linear-gradient(180deg, ${chroma(chassisColor).brighten(0.3)}, ${chassisColor})`,
                            borderRadius: '6px 6px 0 0',
                            border: '1px solid rgba(0,0,0,0.4)',
                            display: 'flex',
                            flexDirection: 'column',
                            alignItems: 'center',
                            pt: 2,
                            zIndex: 2
                        }}
                    >
                        <Box
                            sx={{
                                width: 8,
                                height: 8,
                                borderRadius: '50%',
                                border: '1px solid rgba(0,0,0,0.3)',
                                bgcolor: isActive ? '#00ff00' : '#ff0000',
                                boxShadow: isActive ? '0 0 6px #00ff00' : '0 0 6px #ff0000',
                                mb: 2,
                                transition: 'background-color 0.1s, box-shadow 0.1s',
                            }}
                        />

                        <Box sx={{color: textColor, textShadow}}>
                            <Stack direction="row" spacing={1} sx={{justifyContent: 'center'}}>
                                {knobsForEffect(effect, {
                                    onThresholdChange: effect.kind == "HCDistortion" ? handleHCThresholdChange : handleSCThresholdChange,
                                    onLevelChange: effect.kind == "Delay" ? handleDelayLevelChange : effect.kind == "HCDistortion" ? handleHCDLevelChange : handleSCLevelChange,
                                    onDelayTimeChange: handleDelayTimeChange,
                                    onSmoothingChange: handleSmoothingChange,
                                })}
                            </Stack>
                        </Box>

                        <Typography
                            sx={{
                                mt: 'auto',
                                mb: 2,
                                fontWeight: 900,
                                fontSize: '1.2rem',
                                color: textColor,
                                textShadow,
                                textTransform: 'uppercase',
                                fontStyle: 'italic'
                            }}
                            noWrap={true}
                        >
                            {effect.data.name}
                        </Typography>
                    </Box>

                    <Box
                        onClick={handleFootswitchClick}
                        sx={{
                            width: 'calc(100% + 8px)',
                            height: 110,
                            flexShrink: 0,
                            bgcolor: '#1a1a1a',
                            borderRadius: '2px 2px 8px 8px',
                            border: '2px solid #000',
                            boxShadow: 'inset 0 2px 4px rgba(255,255,255,0.1)',
                            display: 'flex',
                            justifyContent: 'center',
                            alignItems: 'flex-end',
                            pb: 1,
                            cursor: 'pointer',
                            zIndex: 3,
                            transition: 'transform 0.05s',
                            '&:active': {transform: 'scale(0.98) translateY(2px)'}
                        }}
                    >
                        <Box
                            sx={{
                                width: 12,
                                height: 12,
                                borderRadius: '50%',
                                background: 'radial-gradient(circle, #444, #000)',
                                border: '1px solid #333'
                            }}
                        />
                    </Box>
                </Box>
            </Stack>

            <MidiBindingDialog
                open={midiModalOpen}
                onClose={() => setMidiModalOpen(false)}
                effectId={effect.data.id}
                effectName={effect.data.name}
                effectKind={effect.kind}
            />
        </>
    );
}