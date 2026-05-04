import {Box, Paper, Stack, Typography} from "@mui/material";
import {EffectPedalPreview} from "./EffectPedalPreview.tsx";
import {EffectDto} from "../domain";

export interface EffectChainProps {
    effects: EffectDto[];
    selected: EffectDto | "amp";
    /** "amp" = amp head selected, EffectDto = that effect is selected */
    onSelectionChange: (selected: EffectDto | "amp") => void;
}

export function EffectChain({effects, selected, onSelectionChange}: EffectChainProps) {
    function isAmpSelected() {
        return selected === "amp";
    }

    function isEffectSelected(effect: EffectDto) {
        return selected !== "amp" && selected.data.id === effect.data.id;
    }

    const selectedBorder = {
        border: '2px solid',
        borderColor: 'primary.main',
        boxShadow: '0 0 0 3px rgba(25,118,210,0.15)',
    };

    return (
        <Box
            component="section"
            sx={{
                width: '100%',
                bgcolor: 'background.paper',
                borderRadius: 4,
                p: 2,
                position: 'relative'
            }}
        >
            <Box sx={{display: 'flex', justifyContent: 'flex-end', mb: 4}}>
                <Paper
                    sx={{
                        bgcolor: 'background.paper',
                        color: 'primary.main',
                        borderRadius: 50,
                        textTransform: 'none',
                        fontSize: '0.875rem',
                        fontWeight: 500,
                        p: 1.2,
                        px: 3,
                        border: '1px solid',
                        borderColor: 'divider',
                        '&:hover': {
                            bgcolor: '#fdfdfd',
                            cursor: 'pointer'
                        }
                    }}
                >
                    Edit order
                </Paper>
            </Box>

            <Box
                sx={{
                    position: 'absolute',
                    left: 0,
                    right: 0,
                    top: '60%',
                    transform: 'translateY(-50%)',
                    height: '6px',
                    bgcolor: 'secondary.main',
                    zIndex: 1
                }}
            />
            <Stack
                direction="row"
                spacing={6}
                sx={{ width: '100%', position: 'relative', zIndex: 2 }}
            >
                {/* Amp node — selected by default */}
                <Box
                    key={0}
                    onClick={() => onSelectionChange("amp")}
                    sx={{ display: 'flex', flexDirection: 'column', alignItems: 'center', cursor: 'pointer' }}
                >
                    <Box sx={{ display: 'flex', alignItems: 'center', height: 75 }}>
                        <Box
                            sx={{
                                width: 60,
                                height: 60,
                                bgcolor: 'background.paper',
                                border: '1px solid',
                                borderColor: 'text.secondary',
                                borderRadius: 2,
                                transition: 'border 0.15s, box-shadow 0.15s',
                                ...(isAmpSelected() && selectedBorder),
                            }}
                        />
                    </Box>
                    <Typography
                        variant="caption"
                        sx={{
                            mt: 1,
                            color: isAmpSelected() ? 'primary.main' : 'text.primary',
                            fontWeight: isAmpSelected() ? 700 : 500,
                            fontSize: '0.75rem',
                        }}
                    >
                        Amp
                    </Typography>
                </Box>

                {effects.map((item) => (
                    <Box
                        key={"effect-" + item.data.id}
                        onClick={() => onSelectionChange(item)}
                        sx={{ display: 'flex', flexDirection: 'column', alignItems: 'center', cursor: 'pointer' }}
                    >
                        <Box sx={{ display: 'flex', alignItems: 'center', height: 75 }}>
                            <Box sx={{
                                borderRadius: 2,
                                transition: 'border 0.15s, box-shadow 0.15s',
                                ...(isEffectSelected(item) && selectedBorder),
                            }}>
                                <EffectPedalPreview mainColor={item.data.color} isActive={item.data.is_active}/>
                            </Box>
                        </Box>
                        <Typography
                            variant="caption"
                            sx={{
                                mt: 1,
                                color: isEffectSelected(item) ? 'primary.main' : 'text.primary',
                                fontWeight: isEffectSelected(item) ? 700 : 500,
                                fontSize: '0.75rem',
                            }}
                        >
                            {item.data.name}
                        </Typography>
                    </Box>
                ))}
            </Stack>
        </Box>
    );
}