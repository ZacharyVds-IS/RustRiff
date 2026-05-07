import {EffectDto} from "../domain";
import {Box, Stack, Typography} from "@mui/material";
import {Knob} from "./selection/Knob.tsx";
import chroma from "chroma-js";

interface AmpBoxProps {
    onSelectionChange: (selected: EffectDto | "amp") => void;
    isAmpSelected: () => boolean;
    selectedBorder: {
        border: string;
        borderColor: string;
        boxShadow: string;
    };
}

export function AmpBox({
                           onSelectionChange,
                           isAmpSelected,
                           selectedBorder
                       }: AmpBoxProps) {
    const cabBlackColor = "#1E1E1D";
    const selected = isAmpSelected();

    return (
        <Box
            onClick={() => onSelectionChange("amp")}
            sx={{
                display: 'flex',
                flexDirection: 'column',
                alignItems: 'center',
                cursor: 'pointer',
                position: 'relative'
            }}
        >
            <Box
                sx={{
                    display: 'flex',
                    flexDirection: "column",
                    alignItems: 'center',
                    height: 75,
                }}
            >
                <Box
                    sx={{
                        width: 80,
                        height: 50,
                        marginTop: 1,
                        bgcolor: 'background.paper',
                        border: '1px solid',
                        borderColor: 'text.secondary',
                        borderRadius: 2,
                        background: `linear-gradient(180deg, 
                        ${chroma(cabBlackColor).brighten(0.8).hex()} 0%, 
                        ${chroma(cabBlackColor).brighten(0.8).hex()} 50%, 
                        ${chroma(cabBlackColor).darken(1.2).hex()} 100%)`,
                        transition: 'border 0.15s, box-shadow 0.15s',
                        ...(selected && selectedBorder),
                        p: 0.2
                    }}
                >
                    <Box sx={{width: "100%", height: "100%", border: "2px solid white"}}>
                        <Box sx={{height: 20}}/>
                        <Stack direction={"row"}>
                            <Knob
                                key={"placeholder-1"}
                                label={""}
                                value={0}
                                size={6}
                                disabled
                            />
                            <Knob
                                key={"placeholder-2"}
                                label={""}
                                value={0}
                                size={6}
                                disabled
                            />
                            <Knob
                                key={"placeholder-3"}
                                label={""}
                                value={0}
                                size={6}
                                disabled
                            />
                            <Knob
                                key={"placeholder-4"}
                                label={""}
                                value={0}
                                size={6}
                                disabled
                            />
                            <Knob
                                key={"placeholder-5"}
                                label={""}
                                value={0}
                                size={6}
                                disabled
                            />
                            <Knob
                                key={"placeholder-6"}
                                label={""}
                                value={0}
                                size={6}
                                disabled
                            />
                        </Stack>
                    </Box>
                </Box>
            </Box>

            <Typography
                variant="caption"
                sx={{
                    mt: 1,
                    color: selected ? 'primary.main' : 'text.primary',
                    fontWeight: selected ? 700 : 500,
                    fontSize: '0.75rem',
                }}
            >
                Amp
            </Typography>
        </Box>
    );
}