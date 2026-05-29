import {useCallback, useEffect, useState} from "react";
import {
    Alert,
    Box,
    Button,
    CircularProgress,
    Grid,
    IconButton,
    Paper,
    Stack,
    Table,
    TableBody,
    TableCell,
    TableContainer,
    TableHead,
    TableRow,
    Typography
} from "@mui/material";
import RefreshIcon from '@mui/icons-material/Refresh';
import DeleteIcon from '@mui/icons-material/Delete';
import {listen} from "@tauri-apps/api/event";
import {getAmpConfig, getMidiBindings, MidiTargetParameter, removeMidiBinding} from "../domain";
import {useNavigate} from "react-router-dom";
import ArrowBackIcon from '@mui/icons-material/ArrowBack';

interface EffectEntry {
    kind: string;
    data?: { id?: string; name?: string };
}

interface ActiveBinding {
    channel: number;
    cc_number: number;
    effect_id: string;
    parameter: MidiTargetParameter;
}

export function MidiConfigScreen() {
    const navigate = useNavigate();
    const [bindings, setBindings] = useState<ActiveBinding[]>([]);
    const [activeEffects, setActiveEffects] = useState<{ id: string; name: string; kind: string }[]>([]);
    const [loading, setLoading] = useState<boolean>(false);
    const [error, setError] = useState<string | null>(null);
    const [successMessage, setSuccessMessage] = useState<string | null>(null);

    // Track state of the hardware recognition module
    const [isLearning, setIsLearning] = useState<boolean>(false);

    const [, setMidiChannel] = useState<number>(1);
    const [, setCcNumber] = useState<number>(11);
    const [selectedEffectId, setSelectedEffectId] = useState<string>("");

    const fetchMidiMatrixData = useCallback(async () => {
        setLoading(true);
        setError(null);
        try {
            const ampConfig = await getAmpConfig();
            const incomingEffects: EffectEntry[] = (ampConfig as { effects?: EffectEntry[] })?.effects || [];

            const parsedEffects = incomingEffects.map((eff) => ({
                id: eff.data?.id || "",
                name: eff.data?.name || `${eff.kind || 'DSP'} Module`,
                kind: eff.kind || "Unknown"
            }));
            setActiveEffects(parsedEffects);

            const activeBindings = await getMidiBindings();
            setBindings(activeBindings);

            if (parsedEffects.length > 0 && !selectedEffectId) {
                setSelectedEffectId(parsedEffects[0].id);
            }
        } catch (err) {
            console.error("Failed to sync system matrices:", err);
            setError(typeof err === "string" ? err : "Failed to sync system operational state.");
        } finally {
            setLoading(false);
        }
    }, [selectedEffectId]);

    // Background listener hooks into the Rust backend's raw traffic broadcaster
    useEffect(() => {
        fetchMidiMatrixData();

        const unlistenPromise = listen<[number, number]>("midi-raw-sniff", (event) => {
            if (isLearning) {
                const [payloadChannel, payloadCc] = event.payload;
                setMidiChannel(payloadChannel);
                setCcNumber(payloadCc);
                setIsLearning(false); // Disengage learn mode instantly once captured
                setSuccessMessage(`Recognized Input! Set Port Line to CH ${payloadChannel}, CC Event ID to #${payloadCc}.`);
            }
        });

        return () => {
            unlistenPromise.then((cleanup) => cleanup());
        };
    }, [isLearning, fetchMidiMatrixData]);

    const handleRemoveBinding = async (channel: number, ccNumber: number) => {
        setError(null);
        setSuccessMessage(null);
        try {
            await removeMidiBinding({ channel, ccNumber });
            setSuccessMessage(`Mapping for Channel ${channel}, CC #${ccNumber} removed.`);
            await fetchMidiMatrixData();
        } catch (err) {
            console.error("Failed to delete map entry:", err);
            setError(typeof err === "string" ? err : "Failed to remove mapping.");
        }
    };

    const formatParameterName = (param: string) => {
        return param.replace(/([A-Z])/g, ' $1').trim();
    };

    return (
        <Box sx={{ p: 4, maxWidth: 900, margin: "0 auto" }}>
            {/* Header Area */}
            <Box sx={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', mb: 4 }}>
                <Box>
                    <Typography variant="h4" sx={{ display: 'flex', alignItems: 'center', gap: 1.5, fontWeight: 'bold' }}>
                        MIDI Configuration
                    </Typography>
                </Box>
                <Stack direction="row" spacing={1.5}>
                    {/* Back Button added to actions area */}
                    <Button
                        variant="outlined"
                        color="inherit"
                        startIcon={<ArrowBackIcon />}
                        onClick={() => navigate(-1)}
                    >
                        Back
                    </Button>
                    <Button
                        variant="contained"
                        startIcon={<RefreshIcon />}
                        onClick={fetchMidiMatrixData}
                        disabled={loading}
                    >
                        Refresh Matrix
                    </Button>
                </Stack>
            </Box>

            {error && (
                <Alert severity="error" sx={{ mb: 3 }} onClose={() => setError(null)}>
                    {error}
                </Alert>
            )}

            {successMessage && (
                <Alert severity="success" sx={{ mb: 3 }} onClose={() => setSuccessMessage(null)}>
                    {successMessage}
                </Alert>
            )}

            <Grid container spacing={3}>
                <Grid size={{ xs: 12 }}>
                    <TableContainer component={Paper} variant="outlined">
                        <Box sx={{ p: 2.5, bgcolor: 'action.hover', display: 'flex', alignItems: 'center', gap: 1.5, borderBottom: '1px solid', borderColor: 'divider' }}>
                            <Typography variant="subtitle1" sx={{ fontWeight: 'bold' }}>
                                Active Mappings
                            </Typography>
                        </Box>

                        {loading ? (
                            <Box sx={{ display: 'flex', justifyContent: 'center', p: 6, alignItems: 'center', gap: 2 }}>
                                <CircularProgress size={24} />
                                <Typography color="text.secondary">Reading internal memory pipelines...</Typography>
                            </Box>
                        ) : bindings.length === 0 ? (
                            <Box sx={{ p: 6, textAlign: 'center' }}>
                                <Typography variant="body1" color="text.secondary" sx={{ fontWeight: 500 }}>
                                    No parameters are currently mapped to continuous controllers.
                                </Typography>
                                <Typography variant="body2" color="text.secondary" sx={{ mt: 0.5 }}>
                                    Toggle the verification engine form above to insert diagnostic entries.
                                </Typography>
                            </Box>
                        ) : (
                            <Table sx={{ minWidth: 650 }}>
                                <TableHead sx={{ bgcolor: 'action.initial' }}>
                                    <TableRow>
                                        <TableCell sx={{ fontWeight: 'bold' }}>Port Line</TableCell>
                                        <TableCell sx={{ fontWeight: 'bold' }}>CC Event ID</TableCell>
                                        <TableCell sx={{ fontWeight: 'bold' }}>Target DSP Module</TableCell>
                                        <TableCell sx={{ fontWeight: 'bold' }}>Intersect Parameter</TableCell>
                                        <TableCell sx={{ fontWeight: 'bold' }}>Hardware Runtime Target Reference</TableCell>
                                        <TableCell sx={{ fontWeight: 'bold', width: 60 }} align="center">Actions</TableCell>
                                    </TableRow>
                                </TableHead>
                                <TableBody>
                                    {bindings.map((binding, index) => {
                                        const matchedEffect = activeEffects.find(e => e.id === binding.effect_id);
                                        const displayName = matchedEffect ? matchedEffect.name : "Custom/Decoupled Block";
                                        const displayKind = matchedEffect ? matchedEffect.kind.replace("Dto", "") : "External Reference";

                                        return (
                                            <TableRow
                                                key={`${binding.effect_id}-${binding.cc_number}-${index}`}
                                                sx={{ '&:last-child td, &:last-child th': { border: 0 }, '&:hover': { bgcolor: 'action.hover' } }}
                                            >
                                                <TableCell>
                                                    <Typography variant="body2" sx={{ fontFamily: 'monospace', fontWeight: 'bold' }}>
                                                        CH {binding.channel}
                                                    </Typography>
                                                </TableCell>
                                                <TableCell>
                                                    <Paper variant="outlined" sx={{ display: 'inline-block', px: 1, py: 0.2, bgcolor: 'background.default', fontFamily: 'monospace', fontWeight: 'bold', fontSize: '0.85rem' }}>
                                                        CC #{binding.cc_number}
                                                    </Paper>
                                                </TableCell>
                                                <TableCell>
                                                    <Stack spacing={0.2}>
                                                        <Typography variant="body2" sx={{ fontWeight: 500 }}>
                                                            {displayName}
                                                        </Typography>
                                                        <Typography variant="caption" color="text.secondary">
                                                            Type: {displayKind}
                                                        </Typography>
                                                    </Stack>
                                                </TableCell>
                                                <TableCell>
                                                    <Typography variant="body2" sx={{ fontWeight: 500, color: 'secondary.main' }}>
                                                        {formatParameterName(binding.parameter)}
                                                    </Typography>
                                                </TableCell>
                                                <TableCell>
                                                    <Typography variant="caption" sx={{ fontFamily: 'monospace', color: 'text.secondary' }}>
                                                        {binding.effect_id}
                                                    </Typography>
                                                </TableCell>
                                                <TableCell align="center">
                                                    <IconButton
                                                        color="error"
                                                        size="small"
                                                        onClick={() => handleRemoveBinding(binding.channel, binding.cc_number)}
                                                    >
                                                        <DeleteIcon fontSize="small" />
                                                    </IconButton>
                                                </TableCell>
                                            </TableRow>
                                        );
                                    })}
                                </TableBody>
                            </Table>
                        )}
                    </TableContainer>
                </Grid>
            </Grid>
        </Box>
    );
}