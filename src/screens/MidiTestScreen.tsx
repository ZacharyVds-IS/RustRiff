import {useEffect, useState} from "react";
import {
    Alert,
    Box,
    Button,
    Card,
    CardContent,
    Chip,
    CircularProgress,
    Divider,
    FormControl,
    FormControlLabel,
    Grid,
    InputLabel,
    List,
    ListItem,
    ListItemText,
    MenuItem,
    Paper,
    Radio,
    RadioGroup,
    Select,
    Stack,
    TextField,
    Typography
} from "@mui/material";
import PianoIcon from '@mui/icons-material/Piano';
import LinkOffIcon from '@mui/icons-material/LinkOff';
import CableIcon from '@mui/icons-material/Cable';
import RefreshIcon from '@mui/icons-material/Refresh';
import SettingsInputComponentIcon from '@mui/icons-material/SettingsInputComponent';
import CheckCircleIcon from '@mui/icons-material/CheckCircle';

import {
    connectMidiDevice,
    disconnectMidiDevice,
    getAmpConfig,
    getMidiInputs,
    MidiDeviceDto,
    registerMidiBinding
} from "../domain";

type MidiTargetParameter =
    | "ToggleBypass"
    | "WahPedalPosition"
    | "DelayTime"
    | "DelayLevel"
    | "DistortionLevel"
    | "DistortionThreshold";

export function MidiTestScreen() {
    const [devices, setDevices] = useState<MidiDeviceDto[]>([]);
    const [activeEffects, setActiveEffects] = useState<{ id: string; name: string; kind: string }[]>([]);
    const [connectedDeviceId, setConnectedDeviceId] = useState<string | null>(null);
    const [loading, setLoading] = useState<boolean>(false);
    const [error, setError] = useState<string | null>(null);
    const [successMessage, setSuccessMessage] = useState<string | null>(null);

    // Dynamic selection mode: 'live' uses active DSP blocks; 'custom' allows raw entries
    const [targetMode, setTargetMode] = useState<"live" | "custom">("live");

    // Form binding configuration states
    const [selectedEffectId, setSelectedEffectId] = useState<string>("");
    const [customEffectId, setCustomEffectId] = useState<string>("");
    const [targetParam, setTargetParam] = useState<MidiTargetParameter>("ToggleBypass");
    const [ccNumber, setCcNumber] = useState<number>(11);
    const [midiChannel, setMidiChannel] = useState<number>(1); // Changed to 1-indexed for the UI state

    const fetchMidiDevices = async () => {
        setLoading(true);
        setError(null);
        try {
            const inputs = await getMidiInputs();
            setDevices(inputs);

            const ampConfig = await getAmpConfig();
            const incomingEffects: any[] = (ampConfig as any)?.effects || [];

            const parsedEffects = incomingEffects.map((eff: any) => ({
                id: eff.data?.id || "",
                name: eff.data?.name || `${eff.kind || 'DSP'} Module`,
                kind: eff.kind || "Unknown"
            }));
            setActiveEffects(parsedEffects);

            if (parsedEffects.length > 0) {
                if (!selectedEffectId) {
                    setSelectedEffectId(parsedEffects[0].id);
                    autoAssignDefaultParam(parsedEffects[0].kind);
                }
            } else {
                setTargetMode("custom");
            }
        } catch (err) {
            console.error("Failed to fetch MIDI configurations:", err);
            setError(typeof err === "string" ? err : "Failed to load hardware configurations.");
        } finally {
            setLoading(false);
        }
    };

    useEffect(() => {
        fetchMidiDevices();
    }, []);

    const autoAssignDefaultParam = (kind: string) => {
        if (kind.includes("Wah")) setTargetParam("WahPedalPosition");
        else if (kind.includes("Delay")) setTargetParam("DelayLevel");
        else if (kind.includes("Distortion")) setTargetParam("DistortionLevel");
        else setTargetParam("ToggleBypass");
    };

    const handleConnect = async (id: string) => {
        setError(null);
        try {
            // Tauri commands expect flattened fields matching the struct properties
            await connectMidiDevice({ id });
            setConnectedDeviceId(id);
        } catch (err) {
            console.error("Failed to connect MIDI hardware:", err);
            setError(typeof err === "string" ? err : "Could not establish hardware connection.");
        }
    };

    const handleDisconnect = async () => {
        setError(null);
        try {
            await disconnectMidiDevice();
            setConnectedDeviceId(null);
        } catch (err) {
            console.error("Failed to close MIDI connection:", err);
            setError(typeof err === "string" ? err : "Failed to safely disconnect hardware.");
        }
    };

    const handleSaveBinding = async () => {
        setError(null);
        setSuccessMessage(null);

        const finalEffectId = targetMode === "live" ? selectedEffectId : customEffectId.trim();

        if (!finalEffectId) {
            setError("Effect Identifier cannot be empty. Please input or choose a target UUID.");
            return;
        }

        try {
            // Match the Rust command signature: 'pub async fn register_midi_binding(..., mapping: MidiMappingDto)'
            // Tauri maps command arguments to top-level keys in your JS payload object.
            const payload = {
                mapping: {
                    cc_number: ccNumber,
                    channel: midiChannel, // Safely matches your Rust binding's expectation
                    effect_id: finalEffectId,
                    parameter: targetParam
                }
            };

            await registerMidiBinding(payload as any);

            const labelName = targetMode === "live"
                ? (activeEffects.find(e => e.id === finalEffectId)?.name || "DSP Block")
                : `Custom Block UUID (${finalEffectId.substring(0, 8)}...)`;

            setSuccessMessage(`Successfully registered binding matrix: CC #${ccNumber} mapped to ${labelName}!`);
        } catch (err) {
            console.error("Failed to register map binding:", err);
            setError(typeof err === "string" ? err : "Failed to write parameter mapping to service.");
        }
    };

    const activeDeviceName = devices.find(d => d.id === connectedDeviceId)?.name || "None";

    return (
        <Box sx={{ p: 4, maxWidth: 800, margin: "0 auto" }}>
            {/* Header Area */}
            <Box sx={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', mb: 3 }}>
                <Box>
                    <Typography variant="h4" sx={{ display: 'flex', alignItems: 'center', gap: 1.5, fontWeight: 'bold' }}>
                        <PianoIcon fontSize="large" color="primary" /> MIDI Hardware Mapping Matrix
                    </Typography>
                    <Typography variant="body2" color="text.secondary">
                        Assign standard continuous controllers (CC) or custom expression sweeps directly to virtual engine addresses.
                    </Typography>
                </Box>
                <Button
                    variant="outlined"
                    startIcon={<RefreshIcon />}
                    onClick={fetchMidiDevices}
                    disabled={loading}
                >
                    Refresh List
                </Button>
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
                {/* Connection Status Panel */}
                <Grid size={{ xs: 12 }}>
                    <Card variant="outlined" sx={{ bgcolor: 'background.paper' }}>
                        <CardContent sx={{ display: 'flex', alignItems: 'center', justifycontent: 'space-between', p: 2, '&:last-child': { pb: 2 } }}>
                            <Box sx={{ display: 'flex', alignItems: 'center', gap: 2, width: '100%', justifyContent: 'space-between' }}>
                                <Box sx={{ display: 'flex', alignItems: 'center', gap: 2 }}>
                                    <CableIcon color={connectedDeviceId ? "success" : "disabled"} sx={{ fontSize: 32 }} />
                                    <Box>
                                        <Typography variant="subtitle2" color="text.secondary" sx={{ fontSize: '0.75rem', fontWeight: 'bold', textTransform: 'uppercase', letterSpacing: '0.5px' }}>
                                            Hardware Link Pipeline
                                        </Typography>
                                        <Typography variant="h6" sx={{ fontWeight: 500 }}>
                                            {connectedDeviceId ? `Incoming Port Active: ${activeDeviceName}` : "Hardware Separated (Virtual Assignment Allowed)"}
                                        </Typography>
                                    </Box>
                                </Box>
                                {connectedDeviceId && (
                                    <Chip
                                        label="Disconnect Port"
                                        color="error"
                                        variant="outlined"
                                        onClick={handleDisconnect}
                                        icon={<LinkOffIcon />}
                                        clickable
                                    />
                                )}
                            </Box>
                        </CardContent>
                    </Card>
                </Grid>

                {/* MIDI Mapping Matrix Configurator Panel */}
                <Grid size={{ xs: 12 }}>
                    <Card variant="outlined">
                        <Box sx={{ p: 2.5, bgcolor: 'action.hover', borderBottom: '1px solid', borderColor: 'divider' }}>
                            <Typography variant="subtitle1" sx={{ fontWeight: 'bold', display: 'flex', alignItems: 'center', gap: 1 }}>
                                <SettingsInputComponentIcon color="primary" /> Define Custom Midi Control Link
                            </Typography>
                        </Box>
                        <CardContent sx={{ p: 3 }}>
                            <Stack spacing={3}>
                                {/* Strategy Selector */}
                                <FormControl component="fieldset">
                                    <Typography variant="body2" color="text.secondary" sx={{ mb: 0.5, fontWeight: 'bold' }}>
                                        Target Select Mode:
                                    </Typography>
                                    <RadioGroup
                                        row
                                        value={targetMode}
                                        onChange={(e) => setTargetMode(e.target.value as "live" | "custom")}
                                    >
                                        <FormControlLabel
                                            value="live"
                                            control={<Radio size="small" />}
                                            label="Choose from Live Patch Stack"
                                            disabled={activeEffects.length === 0}
                                        />
                                        <FormControlLabel
                                            value="custom"
                                            control={<Radio size="small" />}
                                            label="Target Custom Effect via Raw UUID"
                                        />
                                    </RadioGroup>
                                </FormControl>

                                <Grid container spacing={2}>
                                    <Grid size={{ xs: 12, sm: 6 }}>
                                        {targetMode === "live" ? (
                                            <FormControl fullWidth size="small">
                                                <InputLabel id="effect-select-label">Target Live DSP Block</InputLabel>
                                                <Select
                                                    labelId="effect-select-label"
                                                    value={selectedEffectId}
                                                    label="Target Live DSP Block"
                                                    onChange={(e) => {
                                                        const effectId = e.target.value;
                                                        setSelectedEffectId(effectId);
                                                        const matched = activeEffects.find(x => x.id === effectId);
                                                        if (matched) autoAssignDefaultParam(matched.kind);
                                                    }}
                                                >
                                                    {activeEffects.map((eff) => (
                                                        <MenuItem key={eff.id} value={eff.id}>
                                                            {eff.name} ({eff.kind.replace("Dto", "")})
                                                        </MenuItem>
                                                    ))}
                                                </Select>
                                            </FormControl>
                                        ) : (
                                            <TextField
                                                label="Target Effect Block UUID"
                                                placeholder="e.g., 936da01f-9abd-4d9d-80c7-02af85c822a8"
                                                size="small"
                                                fullWidth
                                                value={customEffectId}
                                                onChange={(e) => setCustomEffectId(e.target.value)}
                                            />
                                        )}
                                    </Grid>

                                    <Grid size={{ xs: 12, sm: 6 }}>
                                        <FormControl fullWidth size="small">
                                            <InputLabel id="param-select-label">Target Parameter</InputLabel>
                                            <Select
                                                labelId="param-select-label"
                                                value={targetParam}
                                                label="Target Parameter"
                                                onChange={(e) => setTargetParam(e.target.value as MidiTargetParameter)}
                                            >
                                                <MenuItem value="ToggleBypass">Bypass Toggle Switch (On/Off)</MenuItem>
                                                <MenuItem value="WahPedalPosition">Wah Position Filter (Sweep)</MenuItem>
                                                <MenuItem value="DelayTime">Delay Feedback Time (ms)</MenuItem>
                                                <MenuItem value="DelayLevel">Delay Wet Level</MenuItem>
                                                <MenuItem value="DistortionLevel">Distortion Clip Gain</MenuItem>
                                                <MenuItem value="DistortionThreshold">Distortion Ceiling Threshold</MenuItem>
                                            </Select>
                                        </FormControl>
                                    </Grid>
                                </Grid>

                                <Grid container spacing={2}>
                                    <Grid size={{ xs: 6, sm: 3 }}>
                                        <TextField
                                            label="Control Change (CC)"
                                            type="number"
                                            size="small"
                                            fullWidth
                                            slotProps={{ htmlInput: { min: 0, max: 127 } }}
                                            value={ccNumber}
                                            onChange={(e) => {
                                                const val = parseInt(e.target.value);
                                                setCcNumber(isNaN(val) ? 0 : Math.max(0, Math.min(127, val)));
                                            }}
                                            helperText="Exp=11, Switch=13"
                                        />
                                    </Grid>

                                    <Grid size={{ xs: 6, sm: 3 }}>
                                        <TextField
                                            label="MIDI Channel"
                                            type="number"
                                            size="small"
                                            fullWidth
                                            slotProps={{ htmlInput: { min: 1, max: 16 } }}
                                            value={midiChannel}
                                            onChange={(e) => {
                                                const val = parseInt(e.target.value);
                                                setMidiChannel(isNaN(val) ? 1 : Math.max(1, Math.min(16, val)));
                                            }}
                                            helperText="Range (1-16)"
                                        />
                                    </Grid>

                                    <Grid size={{ xs: 12, sm: 6 }} sx={{ display: 'flex', alignItems: 'flex-start', pt: 0.5 }}>
                                        <Button
                                            variant="contained"
                                            color="secondary"
                                            fullWidth
                                            startIcon={<CheckCircleIcon />}
                                            onClick={handleSaveBinding}
                                        >
                                            Save Control Binding
                                        </Button>
                                    </Grid>
                                </Grid>
                            </Stack>
                        </CardContent>
                    </Card>
                </Grid>

                {/* Available Hardware Ports List */}
                <Grid size={{ xs: 12 }}>
                    <Typography variant="h6" sx={{ mb: 1, fontWeight: 'bold' }}>
                        Detected Hardware MIDI Ingestion Ports
                    </Typography>
                    <Paper variant="outlined">
                        {loading ? (
                            <Box sx={{ display: 'flex', justifyContent: 'center', p: 4, alignItems: 'center', gap: 2 }}>
                                <CircularProgress size={24} />
                                <Typography color="text.secondary">Polling hardware interface drivers...</Typography>
                            </Box>
                        ) : devices.length === 0 ? (
                            <Box sx={{ p: 4, textAlign: 'center' }}>
                                <Typography variant="body1" color="text.secondary">
                                    No hardware controllers or pedals detected. Unbound custom mappings can still be managed above.
                                </Typography>
                            </Box>
                        ) : (
                            <List disablePadding>
                                {devices.map((device, index) => {
                                    const isCurrent = device.id === connectedDeviceId;
                                    return (
                                        <Box key={device.id}>
                                            {index > 0 && <Divider />}
                                            <ListItem
                                                sx={{
                                                    py: 1.5,
                                                    px: 3,
                                                    bgcolor: isCurrent ? 'action.selected' : 'transparent',
                                                    '&:hover': { bgcolor: 'action.hover' }
                                                }}
                                                secondaryAction={
                                                    isCurrent ? (
                                                        <Button variant="contained" color="error" size="small" onClick={handleDisconnect}>
                                                            Disconnect
                                                        </Button>
                                                    ) : (
                                                        <Button variant="contained" color="primary" size="small" onClick={() => handleConnect(device.id)}>
                                                            Connect
                                                        </Button>
                                                    )
                                                }
                                            >
                                                <ListItemText
                                                    primary={device.name}
                                                    secondary={`System assigned index: ${device.id}`}
                                                />
                                            </ListItem>
                                        </Box>
                                    );
                                })}
                            </List>
                        )}
                    </Paper>
                </Grid>
            </Grid>
        </Box>
    );
}