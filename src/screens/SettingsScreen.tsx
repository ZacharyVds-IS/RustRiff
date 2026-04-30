import {
    Alert,
    Box,
    Button,
    CircularProgress,
    Divider,
    FormControlLabel,
    Stack,
    Switch,
    Typography,
    useTheme
} from "@mui/material";
import {DropdownSelector} from "../components/selection/DropdownSelector.tsx";
import {useAudioDevices} from "../hooks/useAudioDevices.ts";
import {useUpdateAudioDevices} from "../hooks/useUpdateAudioDevices.ts";
import {useEffect, useState} from "react";
import {useUIStore} from "../state/UIStore.tsx";
import * as commands from "../domain/commands.ts";
import * as types from "../domain/types.ts";

export function SettingsScreen() {
    const theme = useTheme();
    const {inputs, outputs, isLoading, error} = useAudioDevices();
    const {updateInputDevice, updateOutputDevice, error: routingError} = useUpdateAudioDevices();

    const selectedInput = useUIStore((state) => state.selectedInputId);
    const setSelectedInput = useUIStore((state) => state.setSelectedInputId);
    const selectedOutput = useUIStore((state) => state.selectedOutputId);
    const setSelectedOutput = useUIStore((state) => state.setSelectedOutputId);
    const developerMode = useUIStore((state) => state.developerMode);
    const setDeveloperMode = useUIStore((state) => state.setDeveloperMode);


    const [inputSampleRate, setInputSampleRate] = useState<number | null>(null);
    const [outputSampleRate, setOutputSampleRate] = useState<number | null>(null);
    const [roundTripLatency, setRoundTripLatency] = useState<number | null>(null);
    const [roundTripLoading, setRoundTripLoading] = useState(false);
    const [roundTripError, setRoundTripError] = useState<string | null>(null);
    const [bufferLatency, setBufferLatency] = useState<types.BufferLatencyDto | null>(null);
    const [bufferSizeFrames, setBufferSizeFrames] = useState<number>(256);
    const [bufferSizeSaving, setBufferSizeSaving] = useState(false);
    const [bufferSizeError, setBufferSizeError] = useState<string | null>(null);

    const BUFFER_SIZE_OPTIONS = [64, 128, 256, 512, 1024, 2048, 4096];

    const inputOptions = inputs.map(d => ({label: `${d.name} (${d.sample_rate} Hz)`, value: d.id}));
    const outputOptions = outputs.map(d => ({label: `${d.name} (${d.sample_rate} Hz)`, value: d.id}));
    const bufferSizeOptions = BUFFER_SIZE_OPTIONS.map((frames) => ({
        label: `${frames} frames`,
        value: frames,
    }));

    async function handleInputChange(value: string | number) {
        const id = String(value);
        const device = inputs.find(d => d.id === id);
        setSelectedInput(id);
        setInputSampleRate(device?.sample_rate ?? null);
        await updateInputDevice(id);
        await loadBufferLatency();
    }

    async function handleOutputChange(value: string | number) {
        const id = String(value);
        const device = outputs.find(d => d.id === id);
        setSelectedOutput(id);
        setOutputSampleRate(device?.sample_rate ?? null);
        await updateOutputDevice(id);
        await loadBufferLatency();
    }

    async function loadBufferLatency() {
        try {
            const latency = await commands.measureBufferLatency();
            setBufferLatency(latency);
        } catch {
            setBufferLatency(null);
        }
    }

    async function loadBufferSizeFrames() {
        try {
            const frames = await commands.getBufferSizeFrames();
            setBufferSizeFrames(frames);
            setBufferSizeError(null);
        } catch (err) {
            setBufferSizeError(err instanceof Error ? err.message : "Failed to load buffer size");
        }
    }

    async function applyBufferSizeFrames() {
        setBufferSizeSaving(true);
        setBufferSizeError(null);
        try {
            await commands.setBufferSizeFrames({frames: bufferSizeFrames});
            await loadBufferLatency();
        } catch (err) {
            setBufferSizeError(err instanceof Error ? err.message : "Failed to apply buffer size");
        } finally {
            setBufferSizeSaving(false);
        }
    }

    function handleBufferSizeChange(value: string | number) {
        setBufferSizeFrames(Number(value));
    }

    async function performRoundTripMeasurement() {
        setRoundTripLoading(true);
        setRoundTripError(null);
        try {
            const result = await commands.measureRoundTripLatency();
            if (result.is_valid) {
                setRoundTripLatency(result.latency_ms);
            } else {
                setRoundTripError(result.error || "Failed to measure round-trip latency");
                setRoundTripLatency(null);
            }
        } catch (err) {
            setRoundTripError(err instanceof Error ? err.message : "Unknown error occurred");
            setRoundTripLatency(null);
        } finally {
            setRoundTripLoading(false);
        }
    }

    function handleMeasureRoundTripLatency() {
        void performRoundTripMeasurement();
    }

    useEffect(() => {
        void loadBufferLatency();
        void loadBufferSizeFrames();
    }, []);

    if (isLoading) return <CircularProgress/>;
    if (error) return <Alert severity="error">{error}</Alert>;

    return (
        <Box sx={{p: 4, display: "flex", flexDirection: "column", height: "100%", gap: 2}}>
            <Typography variant="h6">Settings</Typography>
            {routingError && <Alert severity="error">{routingError}</Alert>}


            <Box
                sx={{
                    display: "flex",
                    flexDirection: "column",
                    flex: 1,
                    minHeight: 0,
                    overflow: "hidden",
                    backgroundColor: theme.palette.background.paper,
                    borderRadius: 2,
                    boxShadow: 2,
                    p: 3,
                }}
            >
                <Box sx={{display: "flex", gap: 3, flex: 1, minHeight: 0, overflow: "hidden"}}>
                    {/* Left: General Settings */}
                    <Box
                        sx={{
                            flex: "0 0 50%",
                            display: "flex",
                            flexDirection: "column",
                            gap: 2,
                            overflowY: "auto",
                            overflowX: "hidden",
                            pr: 2,
                            "&::-webkit-scrollbar": {width: "8px"},
                            "&::-webkit-scrollbar-track": {background: "transparent"},
                            "&::-webkit-scrollbar-thumb": {
                                background: theme.palette.action.disabled,
                                borderRadius: "4px"
                            },
                        }}
                    >
                        <FormControlLabel
                            control={<Switch checked={developerMode}
                                             onChange={(e) => setDeveloperMode(e.target.checked)}/>}
                            label="Developer Mode"
                        />

                        {inputSampleRate && outputSampleRate && inputSampleRate !== outputSampleRate && (
                            <Typography variant="body1">
                                <Box component="span" sx={{color: theme.palette.primary.main, fontWeight: "bold"}}>
                                    Sample rates do not match!
                                </Box>{" "}
                                Output will have a sample rate of:{" "}
                                <Box component="span" sx={{fontWeight: "bold", color: theme.palette.primary.main}}>
                                    {outputSampleRate} Hz
                                </Box>
                            </Typography>
                        )}

                        <Box sx={{mt: 2, pt: 2, borderTop: `1px solid ${theme.palette.divider}`}}>
                            <Typography variant="subtitle2" sx={{fontWeight: "bold", mb: 1}}>Latency</Typography>

                            <Box sx={{borderRadius: 1, mb: 1.5}}>
                                <Box sx={{display: "flex", gap: 1, alignItems: "center"}}>
                                    <Stack sx={{width:"100%"}}>
                                        <Box sx={{flex: 1}}>
                                            <DropdownSelector
                                                title="Prefered Buffer Size"
                                                label="Frames"
                                                options={bufferSizeOptions}
                                                selectedValue={bufferSizeFrames}
                                                onSelectionChange={handleBufferSizeChange}
                                            />
                                            <Typography variant="caption"
                                                        sx={{display: "block", mt: 0.75, color: theme.palette.text.secondary}}>
                                                Lower frames reduce latency but can increase crackles on weaker CPUs.
                                            </Typography>
                                        </Box>
                                        <Button
                                            sx={{mt:1}}
                                            variant="outlined"
                                            size="small"
                                            onClick={applyBufferSizeFrames}
                                            disabled={bufferSizeSaving}
                                        >
                                            {bufferSizeSaving ? "Applying..." : "Apply"}
                                        </Button>
                                    </Stack>
                                </Box>
                                {bufferSizeError && (
                                    <Alert severity="error" sx={{mt: 1}}>{bufferSizeError}</Alert>
                                )}
                            </Box>

                            <Box sx={{p: 1.5, backgroundColor: theme.palette.action.hover, borderRadius: 1, mb: 1.5}}>
                                <Typography variant="body2" sx={{fontWeight: "bold"}}>Estimated Buffer
                                    Latency</Typography>
                                {bufferLatency ? (
                                    <Typography variant="caption" sx={{display: "block", mt: 0.5}}>
                                        {bufferLatency.input_buffer_latency_ms.toFixed(2)} ms (input) +{" "}
                                        {bufferLatency.output_buffer_latency_ms.toFixed(2)} ms (output) ={" "}
                                        <Box component="span"
                                             sx={{fontWeight: "bold", color: theme.palette.primary.main}}>
                                            {bufferLatency.total_buffer_latency_ms.toFixed(2)} ms
                                        </Box>
                                    </Typography>
                                ) : (
                                    <Typography variant="caption"
                                                sx={{display: "block", mt: 0.5, color: theme.palette.text.secondary}}>
                                        Unable to read current buffer latency.
                                    </Typography>
                                )}
                            </Box>

                            <Box sx={{p: 1.5, backgroundColor: theme.palette.action.hover, borderRadius: 1}}>
                                <Typography variant="body2" sx={{fontWeight: "bold", mb: 1}}>Measured Round-Trip
                                    Latency</Typography>
                                <Button
                                    variant="outlined"
                                    size="small"
                                    onClick={handleMeasureRoundTripLatency}
                                    disabled={roundTripLoading}
                                    fullWidth
                                    sx={{mb: 1}}
                                >
                                    {roundTripLoading ? "Measuring..." : "Measure Round-Trip"}
                                </Button>

                                {roundTripError && <Alert severity="error" sx={{mb: 1}}>{roundTripError}</Alert>}

                                {roundTripLatency !== null && (
                                    <Typography variant="caption" sx={{display: "block"}}>
                                        <Box component="span"
                                             sx={{fontWeight: "bold", color: theme.palette.primary.main}}>
                                            {roundTripLatency.toFixed(2)} ms
                                        </Box>{" "}
                                        measured end-to-end.
                                    </Typography>
                                )}

                                {roundTripLatency === null && !roundTripError && (
                                    <Typography variant="caption"
                                                sx={{display: "block", color: theme.palette.text.secondary}}>
                                        Route output back into input (for example: Line Out to Line In), then press
                                        Measure Round-Trip.
                                    </Typography>
                                )}
                            </Box>
                        </Box>
                    </Box>

                    <Divider orientation="vertical"/>

                    {/* Right: Device Settings */}
                    <Box
                        sx={{
                            flex: "0 0 50%",
                            display: "flex",
                            flexDirection: "column",
                            gap: 2,
                            pl: 2,
                            overflowX: "hidden",
                        }}
                    >
                        <DropdownSelector
                            title="Input Device"
                            label="Select input device"
                            options={inputOptions}
                            selectedValue={selectedInput}
                            onSelectionChange={handleInputChange}
                        />

                        <DropdownSelector
                            title="Output Device"
                            label="Select output device"
                            options={outputOptions}
                            selectedValue={selectedOutput}
                            onSelectionChange={handleOutputChange}
                        />
                    </Box>
                </Box>
            </Box>
        </Box>
    );
}

