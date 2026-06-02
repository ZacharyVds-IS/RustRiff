import {Box, Chip, Paper, Stack, Typography} from "@mui/material";
import CheckCircleIcon from "@mui/icons-material/CheckCircle";

interface ParameterOption {
    value: string;
    label: string;
    description: string;
    icon: string;
}

interface ParameterCardProps {
    option: ParameterOption;
    isSelected: boolean;
    alreadyMapped: boolean;
    onSelect: (value: string) => void;
}

export function ParameterCard({
                                  option,
                                  isSelected,
                                  alreadyMapped,
                                  onSelect,
                              }: ParameterCardProps) {
    return (
        <Paper
            variant="outlined"
            onClick={() => onSelect(option.value)}
            sx={{
                p: 1.5,
                cursor: "pointer",
                borderColor: isSelected ? "primary.main" : "divider",
                bgcolor: isSelected ? "action.selected" : "transparent",
                transition: "all 0.15s",
                "&:hover": {
                    borderColor: "primary.light",
                    bgcolor: "action.hover"
                },
            }}
        >
            <Stack direction="row">
                <Typography sx={{ fontSize: "1.2rem", minWidth: 24, textAlign: "center" }}>
                    {option.icon}
                </Typography>
                <Box sx={{ flex: 1 }}>
                    <Stack direction="row">
                        <Typography variant="body2" sx={{ fontWeight: 600 }}>
                            {option.label}
                        </Typography>
                        {alreadyMapped && (
                            <Chip
                                label="mapped"
                                size="small"
                                color="success"
                                variant="filled"
                                sx={{ height: 18, fontSize: "0.6rem", ml:1 }}
                            />
                        )}
                    </Stack>
                    <Typography variant="caption" color="text.secondary">
                        {option.description}
                    </Typography>
                </Box>
                {isSelected && (
                    <CheckCircleIcon color="primary" sx={{ fontSize: 18 }} />
                )}
            </Stack>
        </Paper>
    );
}