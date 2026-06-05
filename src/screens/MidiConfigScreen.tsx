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
import {useNavigate} from "react-router-dom";
import ArrowBackIcon from '@mui/icons-material/ArrowBack';
import {formatParameterName, useMidiBindings} from "../hooks/useMidiBindings.ts";
import {useMidiLearning} from "../hooks/useMidiLearning.ts";

export function MidiConfigScreen() {
    const navigate = useNavigate();
    const {
        bindings,
        activeEffects,
        loading,
        error,
        successMessage,
        setError,
        setSuccessMessage,
        handleRemoveBinding,
        refresh: fetchMidiMatrixData,
    } = useMidiBindings();

    useMidiLearning();

    return (
        <Box sx={{ p: 4, maxWidth: 900, margin: "0 auto" }}>
            <Box sx={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', mb: 4 }}>
                <Box>
                    <Typography variant="h4" sx={{ display: 'flex', alignItems: 'center', gap: 1.5, fontWeight: 'bold' }}>
                        MIDI Configuration
                    </Typography>
                </Box>
                <Stack direction="row" spacing={1.5}>
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
