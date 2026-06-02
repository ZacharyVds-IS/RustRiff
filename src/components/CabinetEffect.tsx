import {Box, IconButton, Stack, Tooltip, Typography} from "@mui/material";
import chroma from "chroma-js";
import {useEffect, useState} from "react";
import {EffectDto, toggleEffect} from "../domain";
import {useAmpStore} from "../state/AmpConfigStore.tsx";
import {OnOffSwitch} from "./selection/OnOffSwitch.tsx";
import SettingsInputHdmiIcon from '@mui/icons-material/SettingsInputHdmi';
import {MidiBindingDialog} from "./dialogs/MidiBindingDialog/MidiBindingDialog.tsx";

interface CabinetEffectProps {
    effect: EffectDto;
    onToggle?: (effectId: string, isActive: boolean) => void;
}

export function CabinetEffect({ effect, onToggle }: CabinetEffectProps) {
    const isCabinet = effect.kind === "Cabinet";
    const [isActive, setIsActive] = useState(effect.data.is_active);
    const [midiModalOpen, setMidiModalOpen] = useState(false);
    const updateEffectActiveState = useAmpStore((state) => state.updateEffectActiveState);

    useEffect(() => {
        if (!isCabinet) {
            return;
        }
        setIsActive(effect.data.is_active);
    }, [isCabinet, effect.data.id, effect.data.is_active]);

    if (!isCabinet) return null;

    const cabBlackColor = "#1E1E1D";
    const baseColor = chroma(effect.data.color).desaturate(0.4).hex();

    async function handlePowerToggle() {
        try {
            const newActive = await toggleEffect({ effectId: effect.data.id });
            setIsActive(newActive);
            updateEffectActiveState(effect.data.id, newActive);
            onToggle?.(effect.data.id, newActive);
        } catch (error) {
            console.error(`Failed to toggle cabinet ${effect.data.id}:`, error);
        }
    }

    return (
        <>
            <Stack direction={"column"} sx={{ alignItems: "center" }}>
                {/* MIDI Configuration Trigger Action */}
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
                    {/* Cabinet body with gradient slope - Marshall inspired */}
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
                            <OnOffSwitch isActive={isActive} onClick={handlePowerToggle} />
                        </Box>

                        {/* Speaker grille area */}
                        <Box
                            sx={{
                                flex: 1,
                                mx: 2,
                                my: 2,
                                borderRadius: 1,
                                display:"flex",
                                justifyContent:"center",
                                alignItems:"center",
                                border: `5px solid ${baseColor}`,
                                background: `repeating-linear-gradient(
                                    90deg,
                                    rgba(0,0,0,0.15) 0px,
                                    rgba(0,0,0,0.15) 3px,
                                    rgba(0,0,0,0.08) 3px,
                                    rgba(0,0,0,0.08) 6px
                                )`,
                                boxShadow: "inset 0 1px 3px rgba(0,0,0,0.2)",
                            }}
                        >
                            <Typography
                                sx={{
                                    fontWeight: 900,
                                    fontSize: "0.9rem",
                                    textTransform: "uppercase",
                                    letterSpacing: 0.5,
                                    color: "white",
                                    fontStyle: "italic",
                                }}
                                noWrap
                            >
                                {effect.data.name}
                            </Typography>
                        </Box>
                    </Box>
                </Box>
            </Stack>

            {/* Configured Modal Interface */}
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