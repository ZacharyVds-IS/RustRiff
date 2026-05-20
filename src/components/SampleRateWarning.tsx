import {Box, Typography, useTheme} from "@mui/material";

interface SampleRateWarningProps {
    inputSampleRate: number | null;
    outputSampleRate: number | null;
}

export function SampleRateWarning({ inputSampleRate, outputSampleRate }: SampleRateWarningProps) {
    const theme = useTheme();

    if (!inputSampleRate || !outputSampleRate || inputSampleRate === outputSampleRate) {
        return null;
    }

    return (
        <Typography variant="body1">
            <Box component="span" sx={{ color: theme.palette.primary.main, fontWeight: "bold" }}>
                Sample rates do not match!
            </Box>{" "}
            Output will have a sample rate of:{" "}
            <Box component="span" sx={{ fontWeight: "bold", color: theme.palette.primary.main }}>
                {outputSampleRate} Hz
            </Box>
        </Typography>
    );
}