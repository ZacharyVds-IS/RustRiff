import {Box, Stack, Typography} from "@mui/material";
import {ParameterCard} from "../ParameterCard";

interface ParameterOption {
    value: string;
    label: string;
    description: string;
    icon: string;
}

interface StepParameterSelectionProps {
    paramOptions: ParameterOption[];
    selectedParam: string;
    effectBindings: Array<{ parameter: string }>;
    onSelectParam: (value: any) => void;
}

export function StepParameterSelection({
                                           paramOptions,
                                           selectedParam,
                                           effectBindings,
                                           onSelectParam,
                                       }: StepParameterSelectionProps) {
    return (
        <Box>
            <Typography
                variant="overline"
                color="text.secondary"
                sx={{ letterSpacing: 1.5, fontSize: "0.65rem" }}
            >
                Step 1 - Parameter selection
            </Typography>
            <Stack spacing={1} sx={{ mt: 1 }}>
                {paramOptions.map((opt) => (
                    <ParameterCard
                        key={opt.value}
                        option={opt}
                        isSelected={selectedParam === opt.value}
                        alreadyMapped={effectBindings.some((b) => b.parameter === opt.value)}
                        onSelect={onSelectParam}
                    />
                ))}
            </Stack>
        </Box>
    );
}