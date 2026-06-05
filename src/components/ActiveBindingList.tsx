import {Box, IconButton, Paper, Stack, Tooltip, Typography} from "@mui/material";
import DeleteIcon from "@mui/icons-material/Delete";

interface Binding {
    channel: number;
    cc_number: number;
    effect_id: string;
    parameter: string;
}

interface ActiveBindingListProps {
    bindings: Binding[];
    paramOptions: { value: string; label: string }[];
    onRemove: (channel: number, ccNumber: number) => void;
}

export function ActiveBindingList({bindings, paramOptions, onRemove}: ActiveBindingListProps) {
    if (bindings.length === 0) return null;

    return (
        <>
            <Box>
                <Typography variant="overline" color="text.secondary"
                            sx={{letterSpacing: 1.5, fontSize: "0.65rem"}}>
                    Active Bindings on this Pedal
                </Typography>
                <Stack spacing={1} sx={{mt: 1}}>
                    {bindings.map((b, i) => {
                        const paramLabel = paramOptions.find((p) => p.value === b.parameter)?.label ?? b.parameter;
                        return (
                            <Paper key={i} variant="outlined"
                                   sx={{px: 1.5, py: 1, display: "flex", alignItems: "center", gap: 1.5}}>
                                <Typography variant="caption" sx={{fontFamily: "monospace", fontWeight: 700}}>
                                    CH {b.channel} · CC #{b.cc_number}
                                </Typography>
                                <Typography variant="caption" color="text.secondary" sx={{flex: 1}}>
                                    → {paramLabel}
                                </Typography>
                                <Tooltip title="Remove binding">
                                    <IconButton size="small" color="error"
                                                onClick={() => onRemove(b.channel, b.cc_number)}>
                                        <DeleteIcon fontSize="small"/>
                                    </IconButton>
                                </Tooltip>
                            </Paper>
                        );
                    })}
                </Stack>
            </Box>
        </>
    );
}
