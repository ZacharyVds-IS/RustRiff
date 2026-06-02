import {useState} from "react";
import {Alert, Box, Button, Collapse, Paper, Stack, TextField, Typography,} from "@mui/material";
import RadioButtonCheckedIcon from "@mui/icons-material/RadioButtonChecked";
import KeyboardArrowDownIcon from "@mui/icons-material/KeyboardArrowDown";

interface StepCcAssignmentProps {
    isLearning: boolean;
    onToggleLearning: () => void;
    midiChannel: number;
    onMidiChannelChange: (val: number) => void;
    ccNumber: number;
    onCcNumberChange: (val: number) => void;
    successMessage: string | null;
    onCloseSuccess: () => void;
    selectedParamLabel?: string;
    effectName: string;
}

export function StepCcAssignment({
                                     isLearning,
                                     onToggleLearning,
                                     midiChannel,
                                     onMidiChannelChange,
                                     ccNumber,
                                     onCcNumberChange,
                                     successMessage,
                                     onCloseSuccess,
                                     selectedParamLabel,
                                     effectName,
                                 }: StepCcAssignmentProps) {
    const [showAdvanced, setShowAdvanced] = useState(false);

    return (
        <Box>
            <Typography
                variant="overline"
                color="text.secondary"
                sx={{ letterSpacing: 1.5, fontSize: "0.65rem" }}
            >
                Step 2 — Assign CC Input
            </Typography>
            {successMessage && (
                <Alert
                    severity="success"
                    onClose={onCloseSuccess}
                    sx={{ py: 0.5, mt: 1, mb: 1 }}
                >
                    {successMessage}
                </Alert>
            )}
            <Button
                variant={isLearning ? "contained" : "outlined"}
                color={isLearning ? "warning" : "secondary"}
                fullWidth
                startIcon={<RadioButtonCheckedIcon />}
                onClick={onToggleLearning}
                sx={{
                    mt: 1,
                    mb: 1.5,
                    height: 44,
                    fontWeight: 700,
                    letterSpacing: 0.5,
                    animation: isLearning ? "pulse 1.2s infinite" : "none",
                    "@keyframes pulse": {
                        "0%": { opacity: 0.65 },
                        "50%": { opacity: 1 },
                        "100%": { opacity: 0.65 },
                    },
                }}
            >
                {isLearning ? "Listening… press a MIDI controller now" : "Recognize button"}
            </Button>
            <Button
                size="small"
                variant="text"
                color="inherit"
                endIcon={
                    <KeyboardArrowDownIcon
                        sx={{
                            transform: showAdvanced ? "rotate(180deg)" : "rotate(0deg)",
                            transition: "0.2s"
                        }}
                    />
                }
                onClick={() => setShowAdvanced(!showAdvanced)}
                sx={{ fontSize: "0.75rem", color: "text.secondary", mb: showAdvanced ? 1.5 : 0 }}
            >
                Advanced Manual Settings
            </Button>
            <Collapse in={showAdvanced} timeout="auto" unmountOnExit>
                <Stack direction="row" spacing={2} sx={{ mb: 2, pt: 0.5 }}>
                    <TextField
                        label="MIDI Channel"
                        type="number"
                        size="small"
                        fullWidth
                        slotProps={{ htmlInput: { min: 1, max: 16 } }}
                        value={midiChannel}
                        onChange={(e) => onMidiChannelChange(parseInt(e.target.value) || 1)}
                    />
                    <TextField
                        label="CC Number"
                        type="number"
                        size="small"
                        fullWidth
                        slotProps={{ htmlInput: { min: 0, max: 127 } }}
                        value={ccNumber}
                        onChange={(e) => onCcNumberChange(parseInt(e.target.value) || 0)}
                    />
                </Stack>
            </Collapse>

            {selectedParamLabel && (
                <Paper
                    variant="outlined"
                    sx={{
                        mt: 1,
                        p: 1.5,
                        bgcolor: "action.hover",
                        borderStyle: "dashed",
                        display: "flex",
                        alignItems: "center",
                        gap: 1,
                    }}
                >
                    <Typography variant="caption" color="text.secondary" sx={{ flex: 1 }}>
                        {" "}
                        <strong>
                            CH {midiChannel} · CC #{ccNumber}
                        </strong>{" "}
                        → <strong>{selectedParamLabel}</strong> on{" "}
                        <strong>{effectName}</strong>
                    </Typography>
                </Paper>
            )}
        </Box>
    );
}