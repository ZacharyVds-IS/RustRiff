import {Box, IconButton, Stack, Tooltip} from "@mui/material";
import chroma from "chroma-js";
import {EffectDto} from "../domain";
import {OnOffSwitch} from "./selection/OnOffSwitch.tsx";
import SettingsInputHdmiIcon from '@mui/icons-material/SettingsInputHdmi';
import {MidiBindingDialog} from "./dialogs/MidiBindingDialog/MidiBindingDialog.tsx";
import {SpeakerGrille} from "./SpeakerGrille.tsx";
import {useEffectToggle} from "../hooks/useEffectToggle.ts";
import {useMidiModal} from "../hooks/useMidiModal.ts";

interface CabinetEffectProps {
    effect: EffectDto;
    onToggle?: (effectId: string, isActive: boolean) => void;
}

export function CabinetEffect({effect, onToggle}: CabinetEffectProps) {
    const isCabinet = effect.kind === "Cabinet";
    const {isActive, handleToggle} = useEffectToggle(effect.data.id, effect.data.is_active, onToggle);
    const {midiModalOpen, openMidiModal, closeMidiModal} = useMidiModal();

    if (!isCabinet) return null;

    const cabBlackColor = "#1E1E1D";
    const baseColor = chroma(effect.data.color).desaturate(0.4).hex();

    return (
        <>
            <Stack direction="column" sx={{alignItems: "center"}}>
                <Tooltip title="MIDI Mapping" arrow placement="top">
                    <IconButton aria-label="midi config" size="small" sx={{mb: 0.5}} onClick={openMidiModal}>
                        <SettingsInputHdmiIcon/>
                    </IconButton>
                </Tooltip>

                <Box
                    sx={{
                        width: 400,
                        height: 300,
                        display: "flex",
                        flexDirection: "column",
                        alignItems: "center",
                        position: "relative",
                        filter: (theme) => theme.palette.mode === 'dark'
                            ? 'drop-shadow(0 12px 24px rgba(255, 255, 255, 0.3))'
                            : 'drop-shadow(0 6px 12px rgba(0,0,0,0.4))',
                    }}
                >
                    <Box
                        sx={{
                            width: "100%",
                            height: "100%",
                            background: `linear-gradient(180deg, 
                                ${chroma(cabBlackColor).brighten(0.8).hex()} 0%, 
                                ${chroma(cabBlackColor).brighten(0.8).hex()} 50%, 
                                ${chroma(cabBlackColor).darken(1.2).hex()} 100%)`,
                            border: "2px solid rgba(0,0,0,0.5)",
                            borderRadius: "8px 8px 12px 12px",
                            position: "relative",
                            overflow: "hidden",
                            display: "flex",
                            flexDirection: "column",
                        }}
                    >
                        <Box
                            sx={{
                                px: 2,
                                pt: 2,
                                pb: 1,
                                display: "flex",
                                alignItems: "center",
                                justifyContent: "flex-end",
                            }}
                        >
                            <OnOffSwitch isActive={isActive} onClick={handleToggle}/>
                        </Box>

                        <SpeakerGrille baseColor={baseColor} name={effect.data.name}/>
                    </Box>
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
