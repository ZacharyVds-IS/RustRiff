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
    Paper,
    Stack,
    Tooltip,
    Typography,
} from "@mui/material";
import CloseIcon from "@mui/icons-material/Close";
import DeleteIcon from "@mui/icons-material/Delete";
import CheckCircleIcon from "@mui/icons-material/CheckCircle";
import {listen} from "@tauri-apps/api/event";
import {getMidiBindings, MidiTargetParameter, registerMidiBinding, removeMidiBinding} from "../../../domain";
import {StepCcAssignment} from "./steps/StepCcAssignment.tsx";
import {StepParameterSelection} from "./steps/StepParameterSelection.tsx";

interface ActiveBinding {
    channel: number;
    cc_number: number;
    effect_id: string;
    parameter: MidiTargetParameter;
}

interface ParameterOption {
    value: MidiTargetParameter;
    label: string;
    description: string;
    icon: string;
}

const PARAMETER_OPTIONS: Record<string, ParameterOption[]> = {
    Wah: [
        { value: "ToggleBypass", label: "On / Off", description: "Toggle the wah effect on and off", icon: "⏻" },
        { value: "WahPedalPosition", label: "Sweep", description: "Control the wah filter sweep position (0–127)", icon: "〰" },
    ],
    Delay: [
        { value: "ToggleBypass", label: "On / Off", description: "Toggle the delay effect on and off", icon: "⏻" },
        { value: "DelayLevel", label: "Wet Level", description: "Control the delay wet/dry mix", icon: "≋" },
        { value: "DelayTime", label: "Delay Time", description: "Control the delay time in milliseconds", icon: "⧗" },
    ],
    HCDistortion: [
        { value: "ToggleBypass", label: "On / Off", description: "Toggle the distortion effect on and off", icon: "⏻" },
        { value: "DistortionLevel", label: "Clip Gain", description: "Control the distortion output level", icon: "▲" },
        { value: "DistortionThreshold", label: "Threshold", description: "Control the clipping threshold", icon: "⌇" },
    ],
    SCDistortion: [
        { value: "ToggleBypass", label: "On / Off", description: "Toggle the distortion effect on and off", icon: "⏻" },
        { value: "DistortionLevel", label: "Clip Gain", description: "Control the distortion output level", icon: "▲" },
        { value: "DistortionThreshold", label: "Threshold", description: "Control the clipping threshold", icon: "⌇" },
    ],
};

const FALLBACK_PARAMETERS: ParameterOption[] = [
    { value: "ToggleBypass", label: "On / Off", description: "Toggle the effect on and off", icon: "⏻" },
];

interface MidiBindingModalProps {
    open: boolean;
    onClose: () => void;
    effectId: string;
    effectName: string;
    effectKind: string;
}

export function MidiBindingDialog({
                                      open,
                                      onClose,
                                      effectId,
                                      effectName,
                                      effectKind,
                                  }: MidiBindingModalProps) {
    const [bindings, setBindings] = useState<ActiveBinding[]>([]);
    const [selectedParam, setSelectedParam] = useState<MidiTargetParameter>("ToggleBypass");
    const [ccNumber, setCcNumber] = useState<number>(11);
    const [midiChannel, setMidiChannel] = useState<number>(1);
    const [isLearning, setIsLearning] = useState<boolean>(false);
    const [error, setError] = useState<string | null>(null);
    const [successMessage, setSuccessMessage] = useState<string | null>(null);
    const [loading, setLoading] = useState(false);

    const paramOptions = PARAMETER_OPTIONS[effectKind] ?? FALLBACK_PARAMETERS;
    const effectBindings = bindings.filter((b) => b.effect_id === effectId);
    const selectedParamOption = paramOptions.find((p) => p.value === selectedParam);

    useEffect(() => {
        if (open) {
            setSelectedParam(paramOptions[0]?.value ?? "ToggleBypass");
            setError(null);
            setSuccessMessage(null);
            fetchBindings();
        }
        if (!open) setIsLearning(false);
    }, [open, effectId,paramOptions]);

    useEffect(() => {
        const unlistenPromise = listen<[number, number]>("midi-raw-sniff", (event) => {
            if (isLearning) {
                const [payloadChannel, payloadCc] = event.payload;
                setMidiChannel(payloadChannel);
                setCcNumber(payloadCc);
                setIsLearning(false);
                setSuccessMessage(`Detected — Channel ${payloadChannel}, CC #${payloadCc}. Confirm to save.`);
            }
        });
        return () => {
            unlistenPromise.then((cleanup) => cleanup());
        };
    }, [isLearning]);

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
            setSuccessMessage(`Mapped: CC #${ccNumber} → ${selectedParamOption?.label}`);
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
            await removeMidiBinding({ channel, ccNumber: cc });
            await fetchBindings();
        } catch (err) {
            setError(typeof err === "string" ? err : "Failed to remove binding.");
        }
    };

    return (
        <Dialog open={open} onClose={onClose} maxWidth="sm" fullWidth>
            <DialogTitle sx={{ display: "flex", alignItems: "center", justifyContent: "space-between", pb: 1.5, borderBottom: "1px solid", borderColor: "divider" }}>
                <Box>
                    <Typography variant="h6" sx={{ fontWeight: 700, lineHeight: 1.2 }}>
                        MIDI Mapping - {effectName} · {effectKind.replace("Dto", "")}
                    </Typography>
                </Box>
                <IconButton onClick={onClose} size="small">
                    <CloseIcon fontSize="small" />
                </IconButton>
            </DialogTitle>

            <DialogContent sx={{ pt: 3, pb: 3 }}>
                <Stack spacing={3}>
                    {error && (
                        <Alert severity="error" onClose={() => setError(null)} sx={{ py: 0.5 }}>
                            {error}
                        </Alert>
                    )}
                    <StepParameterSelection
                        paramOptions={paramOptions}
                        selectedParam={selectedParam}
                        effectBindings={effectBindings}
                        onSelectParam={(val) => setSelectedParam(val as MidiTargetParameter)}
                    />

                    <Divider />
                    <StepCcAssignment
                        isLearning={isLearning}
                        onToggleLearning={() => {
                            setIsLearning(!isLearning);
                            setSuccessMessage(null);
                            setError(null);
                        }}
                        midiChannel={midiChannel}
                        onMidiChannelChange={setMidiChannel}
                        ccNumber={ccNumber}
                        onCcNumberChange={setCcNumber}
                        successMessage={successMessage}
                        onCloseSuccess={() => setSuccessMessage(null)}
                        selectedParamLabel={selectedParamOption?.label}
                        effectName={effectName}
                    />

                    {/* Persist/Save Submission */}
                    <Button
                        variant="contained"
                        color="primary"
                        fullWidth
                        size="large"
                        startIcon={<CheckCircleIcon />}
                        onClick={handleSave}
                        disabled={loading}
                        sx={{ fontWeight: 700 }}
                    >
                        Save Binding
                    </Button>

                    {/* Existing active associations layout list */}
                    {effectBindings.length > 0 && (
                        <>
                            <Divider />
                            <Box>
                                <Typography variant="overline" color="text.secondary" sx={{ letterSpacing: 1.5, fontSize: "0.65rem" }}>
                                    Active Bindings on this Pedal
                                </Typography>
                                <Stack spacing={1} sx={{ mt: 1 }}>
                                    {effectBindings.map((b, i) => {
                                        const paramLabel = paramOptions.find((p) => p.value === b.parameter)?.label ?? b.parameter;
                                        return (
                                            <Paper key={i} variant="outlined" sx={{ px: 1.5, py: 1, display: "flex", alignItems: "center", gap: 1.5 }}>
                                                <Typography variant="caption" sx={{ fontFamily: "monospace", fontWeight: 700 }}>
                                                    CH {b.channel} · CC #{b.cc_number}
                                                </Typography>
                                                <Typography variant="caption" color="text.secondary" sx={{ flex: 1 }}>
                                                    → {paramLabel}
                                                </Typography>
                                                <Tooltip title="Remove binding">
                                                    <IconButton size="small" color="error" onClick={() => handleRemove(b.channel, b.cc_number)}>
                                                        <DeleteIcon fontSize="small" />
                                                    </IconButton>
                                                </Tooltip>
                                            </Paper>
                                        );
                                    })}
                                </Stack>
                            </Box>
                        </>
                    )}
                </Stack>
            </DialogContent>
        </Dialog>
    );
}