import {Alert, Box, CircularProgress, Divider, FormControlLabel, Switch, Typography, useTheme} from "@mui/material";
import {useAudioDevices} from "../hooks/useAudioDevices.ts";
import {useUpdateAudioDevices} from "../hooks/useUpdateAudioDevices.ts";
import {useEffect, useRef, useState} from "react";
import {useUIStore} from "../state/UIStore.tsx";
import * as commands from "../domain/commands.ts";
import * as types from "../domain/types.ts";
import {DeviceRoutingSection} from "../components/DeviceroutingSection.tsx";
import {LatencySection} from "../components/LatencySection.tsx";
import {SampleRateWarning} from "../components/SampleRateWarning.tsx";

export function SettingsScreen() {
    const theme = useTheme();
    const {inputs, outputs, isLoading, error, refresh: refreshDevices} = useAudioDevices();
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
    const [availableDrivers, setAvailableDrivers] = useState<string[]>(["Default"]);
    const [selectedDriver, setSelectedDriver] = useState<string>("Default");
    const [driverError, setDriverError] = useState<string | null>(null);
    const [asioInputChannelOptions, setAsioInputChannelOptions] = useState<number[]>([]);
    const [asioOutputChannelOptions, setAsioOutputChannelOptions] = useState<number[]>([]);
    const [selectedAsioInputChannels, setSelectedAsioInputChannels] = useState<number | null>(null);
    const [selectedAsioOutputChannels, setSelectedAsioOutputChannels] = useState<number | null>(null);
    const [asioChannelsError, setAsioChannelsError] = useState<string | null>(null);

    // Prevents auto-applying the buffer size when it is first loaded from the backend on mount
    const isInitialMount = useRef(true);

    const BUFFER_SIZE_OPTIONS = [64, 128, 256, 512, 1024, 2048, 4096];

    const inputOptions = inputs.map(d => ({label: `${d.name} (${d.sample_rate} Hz)`, value: d.id}));
    const outputOptions = outputs.map(d => ({label: `${d.name} (${d.sample_rate} Hz)`, value: d.id}));
    const driverOptions = availableDrivers.map((driver) => ({label: driver, value: driver}));

    const asioInputChannelDropdownOptions = asioInputChannelOptions.map((channels) => ({
        label: `channel ${channels}`,
        value: channels,
    }));
    const asioOutputChannelDropdownOptions = asioOutputChannelOptions.map((channels) => ({
        label: `channel ${channels}`,
        value: channels,
    }));
    const bufferSizeOptions = BUFFER_SIZE_OPTIONS.map((frames) => ({
        label: `${frames} frames`,
        value: frames,
    }));
    const isAsioMode = selectedDriver.toLowerCase() === "asio";

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

    async function handleAsioDeviceChange(value: string | number) {
        const id = String(value);
        const device = inputs.find(d => d.id === id);
        setSelectedInput(id);
        setSelectedOutput(id);
        setInputSampleRate(device?.sample_rate ?? null);
        setOutputSampleRate(device?.sample_rate ?? null);
        await updateInputDevice(id);
        await loadAsioChannelCapabilities(id);
        await loadBufferLatency();
    }

    async function loadAsioChannelCapabilities(deviceId: string) {
        if (!deviceId) {
            setAsioInputChannelOptions([]);
            setAsioOutputChannelOptions([]);
            setSelectedAsioInputChannels(null);
            setSelectedAsioOutputChannels(null);
            setAsioChannelsError(null);
            return;
        }

        try {
            const [inputOptions, outputOptions, selectedInputChannels, selectedOutputChannels] = await Promise.all([
                commands.getInputChannelOptions({deviceId}),
                commands.getOutputChannelOptions({deviceId}),
                commands.getSelectedInputChannelCount(),
                commands.getSelectedOutputChannelCount(),
            ]);

            setAsioInputChannelOptions(inputOptions);
            setAsioOutputChannelOptions(outputOptions);
            setSelectedAsioInputChannels(selectedInputChannels);
            setSelectedAsioOutputChannels(selectedOutputChannels);
            setAsioChannelsError(null);
        } catch (err) {
            setAsioChannelsError(err instanceof Error ? err.message : "Failed to load ASIO channel options");
        }
    }

    async function applyAsioChannelConfig(inputChannels: number, outputChannels: number) {
        if (!selectedInput) {
            return;
        }

        setAsioChannelsError(null);
        try {
            await commands.setAsioChannelConfig({
                deviceId: selectedInput,
                inputChannels,
                outputChannels,
            });
            setSelectedAsioInputChannels(inputChannels);
            setSelectedAsioOutputChannels(outputChannels);
            await loadBufferLatency();
        } catch (err) {
            setAsioChannelsError(err instanceof Error ? err.message : "Failed to set ASIO channel config");
        }
    }

    async function handleAsioInputChannelsChange(value: string | number) {
        const inputChannels = Number(value);
        const outputChannels =
            selectedAsioOutputChannels ?? asioOutputChannelOptions[0] ?? inputChannels;
        await applyAsioChannelConfig(inputChannels, outputChannels);
    }

    async function handleAsioOutputChannelsChange(value: string | number) {
        const outputChannels = Number(value);
        const inputChannels =
            selectedAsioInputChannels ?? asioInputChannelOptions[0] ?? outputChannels;
        await applyAsioChannelConfig(inputChannels, outputChannels);
    }

    async function loadAudioDrivers() {
        try {
            const [drivers, selected] = await Promise.all([
                commands.getAvailableAudioDrivers(),
                commands.getSelectedAudioDriver(),
            ]);
            setAvailableDrivers(drivers.length > 0 ? drivers : ["Default"]);
            setSelectedDriver(selected || "Default");
            setDriverError(null);
        } catch (err) {
            setDriverError(err instanceof Error ? err.message : "Failed to load audio drivers");
        }
    }

    async function handleDriverChange(value: string | number) {
        const driver = String(value);
        setDriverError(null);
        setAsioChannelsError(null);
        try {
            await commands.setAudioDriver({driver});
            setSelectedDriver(driver);
            await refreshDevices();
            await loadBufferLatency();
        } catch (err) {
            setDriverError(err instanceof Error ? err.message : "Failed to switch audio driver");
        }
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
        } finally {
            // Setup flag so subsequent state updates trigger auto-save hook safely
            isInitialMount.current = false;
        }
    }

    function handleBufferSizeChange(value: string | number) {
        setBufferSizeFrames(Number(value));
    }

    // Dynamic Auto-Apply effect tracking buffer selection shifts
    useEffect(() => {
        if (isInitialMount.current) return;

        async function autoApplyBuffer() {
            setBufferSizeSaving(true);
            setBufferSizeError(null);
            try {
                await commands.setBufferSizeFrames({frames: bufferSizeFrames});
                await loadBufferLatency();
            } catch (err) {
                setBufferSizeError(err instanceof Error ? err.message : "Failed to auto-apply buffer size");
            } finally {
                setBufferSizeSaving(false);
            }
        }

        void autoApplyBuffer();
    }, [bufferSizeFrames]);

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
        void loadAudioDrivers();
        void loadBufferLatency();
        void loadBufferSizeFrames();
    }, []);

    useEffect(() => {
        if (isAsioMode) {
            const inputDeviceExists = inputs.some(d => d.id === selectedInput);

            if ((!selectedInput || !inputDeviceExists) && inputs.length > 0) {
                const initialAsioDevice = inputs[0];
                setSelectedInput(initialAsioDevice.id);
                setSelectedOutput(initialAsioDevice.id);
                setInputSampleRate(initialAsioDevice.sample_rate);
                setOutputSampleRate(initialAsioDevice.sample_rate);
                void loadAsioChannelCapabilities(initialAsioDevice.id);
            } else if (selectedInput && inputDeviceExists) {
                void loadAsioChannelCapabilities(selectedInput);
            } else {
                void loadAsioChannelCapabilities("");
            }
            return;
        }

        setAsioInputChannelOptions([]);
        setAsioOutputChannelOptions([]);
        setSelectedAsioInputChannels(null);
        setSelectedAsioOutputChannels(null);
        setAsioChannelsError(null);

        if (!selectedInput && inputs.length > 0) {
            setSelectedInput(inputs[0].id);
            setInputSampleRate(inputs[0].sample_rate);
        }
        if (!selectedOutput && outputs.length > 0) {
            setSelectedOutput(outputs[0].id);
            setOutputSampleRate(outputs[0].sample_rate);
        }
    }, [isAsioMode, inputs, outputs, selectedInput, selectedOutput, setSelectedInput, setSelectedOutput]);

    if (isLoading) return (
        <Box sx={{
            p: 4,
            display: "flex",
            justifyContent: "center",
            alignItems: "center",
            flexDirection: "row",
            width: "100%",
            minHeight: "100vh",
            gap: 2,
        }}>
            <CircularProgress/>
        </Box>);
    if (error) return <Alert severity="error">{error}</Alert>;

    return (
        <Box sx={{p: 4, display: "flex", flexDirection: "column", height: "100%", gap: 2}}>
            <Typography variant="h6">Settings</Typography>
            {routingError && <Alert severity="error">{routingError}</Alert>}
            {driverError && <Alert severity="error">{driverError}</Alert>}
            {asioChannelsError && <Alert severity="error">{asioChannelsError}</Alert>}
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
                <Box sx={{display: "flex", gap: 3, flex: 1, minHeight: 0, minWidth: 0, overflow: "hidden"}}>
                    <Box
                        sx={{
                            flex: 1,
                            minWidth: 0,
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
                        <SampleRateWarning
                            inputSampleRate={inputSampleRate}
                            outputSampleRate={outputSampleRate}
                        />
                        <LatencySection
                            bufferSizeOptions={bufferSizeOptions}
                            bufferSizeFrames={bufferSizeFrames}
                            handleBufferSizeChange={handleBufferSizeChange}
                            bufferSizeSaving={bufferSizeSaving}
                            bufferSizeError={bufferSizeError}
                            bufferLatency={bufferLatency}
                            handleMeasureRoundTripLatency={handleMeasureRoundTripLatency}
                            roundTripLoading={roundTripLoading}
                            roundTripError={roundTripError}
                            roundTripLatency={roundTripLatency}
                        />
                    </Box>
                    <Divider orientation="vertical" flexItem/>
                    <DeviceRoutingSection
                        driverOptions={driverOptions}
                        selectedDriver={selectedDriver}
                        handleDriverChange={handleDriverChange}
                        isAsioMode={isAsioMode}
                        inputOptions={inputOptions}
                        selectedInput={selectedInput}
                        handleAsioDeviceChange={handleAsioDeviceChange}
                        handleInputChange={handleInputChange}
                        outputOptions={outputOptions}
                        selectedOutput={selectedOutput}
                        handleOutputChange={handleOutputChange}
                        asioInputChannelDropdownOptions={asioInputChannelDropdownOptions}
                        selectedAsioInputChannels={selectedAsioInputChannels}
                        handleAsioInputChannelsChange={handleAsioInputChannelsChange}
                        asioOutputChannelDropdownOptions={asioOutputChannelDropdownOptions}
                        selectedAsioOutputChannels={selectedAsioOutputChannels}
                        handleAsioOutputChannelsChange={handleAsioOutputChannelsChange}
                    />
                </Box>
            </Box>
        </Box>
    );
}