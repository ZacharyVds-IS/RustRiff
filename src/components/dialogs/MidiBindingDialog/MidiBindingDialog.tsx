import {useEffect, useState} from "react";
import {
    Alert,
    Box,
    Button,
    Dialog,
    DialogContent,
    DialogTitle,
    Divider,
    IconButton,
    Stack,
    Typography,
} from "@mui/material";
import CloseIcon from "@mui/icons-material/Close";
import CheckCircleIcon from "@mui/icons-material/CheckCircle";
import {getMidiBindings, MidiTargetParameter, registerMidiBinding} from "../../../domain";
import {StepCcAssignment} from "./steps/StepCcAssignment.tsx";
import {StepParameterSelection} from "./steps/StepParameterSelection.tsx";
import {ActiveBindingList} from "../../ActiveBindingList.tsx";
import {useMidiLearning} from "../../../hooks/useMidiLearning.ts";
import {FALLBACK_PARAMETERS, PARAMETER_OPTIONS} from "../../../config/midiParameters.ts";

interface MidiBindingModalProps {
    open: boolean;
    onClose: () => void;
    effectId: string;
    effectName: string;
    effectKind: string;
}

export function MidiBindingDialog({open, onClose, effectId, effectName, effectKind}: MidiBindingModalProps) {
    const [bindings, setBindings] = useState<Array<{ channel: number; cc_number: number; effect_id: string; parameter: MidiTargetParameter }>>([]);
    const [selectedParam, setSelectedParam] = useState<MidiTargetParameter>("ToggleBypass");
    const [error, setError] = useState<string | null>(null);
    const [successMessage, setSuccessMessage] = useState<string | null>(null);
    const [loading, setLoading] = useState(false);

    const {
        isLearning,
        setIsLearning,
        midiChannel,
        setMidiChannel,
        ccNumber,
        setCcNumber,
        learnedMessage,
        setLearnedMessage,
    } = useMidiLearning();

    const paramOptions = PARAMETER_OPTIONS[effectKind] ?? FALLBACK_PARAMETERS;
    const effectBindings = bindings.filter((b) => b.effect_id === effectId);

    useEffect(() => {
        if (open) {
            setSelectedParam(paramOptions[0]?.value ?? "ToggleBypass");
            setError(null);
            setSuccessMessage(null);
            fetchBindings();
        }
        if (!open) setIsLearning(false);
    }, [open, effectId, paramOptions, setIsLearning]);

    useEffect(() => {
        if (learnedMessage) {
            setSuccessMessage(learnedMessage);
            setLearnedMessage(null);
        }
    }, [learnedMessage, setLearnedMessage]);

    const fetchBindings = async () => {
        try {
            const all = await getMidiBindings();
            setBindings(all);
        } catch (err) {
            console.error("Failed to fetch MIDI bindings:", err);
        }
    };

    const handleSave = async () => {
        setError(null);
        setSuccessMessage(null);
        setLoading(true);
        try {
            await registerMidiBinding({
                mapping: {
                    cc_number: ccNumber,
                    channel: midiChannel,
                    effect_id: effectId,
                    parameter: selectedParam,
                },
            });
            setSuccessMessage(`Mapped: CC #${ccNumber} → ${selectedParam}`);
            await fetchBindings();
        } catch (err) {
            setError(typeof err === "string" ? err : "Failed to save binding.");
        } finally {
            setLoading(false);
        }
    };

    const handleRemove = async (channel: number, cc: number) => {
        setError(null);
        setSuccessMessage(null);
        try {
            const {removeMidiBinding} = await import("../../../domain");
            await removeMidiBinding({channel, ccNumber: cc});
            await fetchBindings();
        } catch (err) {
            setError(typeof err === "string" ? err : "Failed to remove binding.");
        }
    };

    const handleToggleLearning = () => {
        setIsLearning(!isLearning);
        setSuccessMessage(null);
        setError(null);
    };

    return (
        <Dialog open={open} onClose={onClose} maxWidth="sm" fullWidth>
            <DialogTitle sx={{display: "flex", alignItems: "center", justifyContent: "space-between", pb: 1.5, borderBottom: "1px solid", borderColor: "divider"}}>
                <Box>
                    <Typography variant="h6" sx={{fontWeight: 700, lineHeight: 1.2}}>
                        MIDI Mapping - {effectName} · {effectKind.replace("Dto", "")}
                    </Typography>
                </Box>
                <IconButton onClick={onClose} size="small">
                    <CloseIcon fontSize="small"/>
                </IconButton>
            </DialogTitle>

            <DialogContent sx={{pt: 3, pb: 3}}>
                <Stack spacing={3}>
                    {error && (
                        <Alert severity="error" onClose={() => setError(null)} sx={{py: 0.5}}>
                            {error}
                        </Alert>
                    )}
                    <StepParameterSelection
                        paramOptions={paramOptions}
                        selectedParam={selectedParam}
                        effectBindings={effectBindings}
                        onSelectParam={(val) => setSelectedParam(val as MidiTargetParameter)}
                    />

                    <Divider/>
                    <StepCcAssignment
                        isLearning={isLearning}
                        onToggleLearning={handleToggleLearning}
                        midiChannel={midiChannel}
                        onMidiChannelChange={setMidiChannel}
                        ccNumber={ccNumber}
                        onCcNumberChange={setCcNumber}
                        successMessage={successMessage}
                        onCloseSuccess={() => setSuccessMessage(null)}
                        selectedParamLabel={paramOptions.find((p) => p.value === selectedParam)?.label}
                        effectName={effectName}
                    />

                    <Button
                        variant="contained"
                        color="primary"
                        fullWidth
                        size="large"
                        startIcon={<CheckCircleIcon/>}
                        onClick={handleSave}
                        disabled={loading}
                        sx={{fontWeight: 700}}
                    >
                        Save Binding
                    </Button>

                    <ActiveBindingList
                        bindings={effectBindings}
                        paramOptions={paramOptions}
                        onRemove={handleRemove}
                    />
                </Stack>
            </DialogContent>
        </Dialog>
    );
}
