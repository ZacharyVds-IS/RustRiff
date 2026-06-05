import {Alert, Box, CircularProgress, Divider, FormControlLabel, Switch, Typography, useTheme} from "@mui/material";
import {useAudioDevices} from "../hooks/useAudioDevices.ts";
import {useUpdateAudioDevices} from "../hooks/useUpdateAudioDevices.ts";
import {useEffect} from "react";
import {useUIStore} from "../state/UIStore.tsx";
import * as commands from "../domain/commands";
import {DeviceRoutingSection} from "../components/DeviceroutingSection.tsx";
import {LatencySection} from "../components/LatencySection.tsx";
import {MidiSection} from "../components/MidiSection.tsx";
import {useAudioDriver} from "../hooks/useAudioDriver.ts";
import {useAsioChannels} from "../hooks/useAsioChannels.ts";
import {useAudioLatency} from "../hooks/useAudioLatency.ts";

function extractErrorMessage(error: unknown): string | null {
    if (error instanceof Error) {
        return error.message;
    }

    if (typeof error === "string") {
        return error;
    }

    if (typeof error === "object" && error && "message" in error) {
        const maybeMessage = (error as {message?: unknown}).message;
        if (typeof maybeMessage === "string") {
            return maybeMessage;
        }
    }

    return null;
}

function formatDriverSwitchError(driver: string, error: unknown): string {
    const message = extractErrorMessage(error);
    if (!message) {
        return "Failed to switch audio driver";
    }

    const normalized = message.toLowerCase();
    if (
        driver.toLowerCase() === "asio"
        && (normalized.includes("no asio device") || normalized.includes("failed to enumerate asio"))
    ) {
        return "No ASIO device found. Connect and power on an ASIO-capable interface, then try again.";
    }

    return message;
}

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

    const {
        driverOptions,
        selectedDriver,
        setSelectedDriver,
        driverError,
        setDriverError,
        isAsioMode,
    } = useAudioDriver();

    const {
        bufferLatency,
        bufferSizeFrames,
        bufferSizeSaving,
        bufferSizeError,
        bufferSizeOptions,
        roundTripLatency,
        roundTripLoading,
        roundTripError,
        handleBufferSizeChange,
        handleMeasureRoundTripLatency,
        loadBufferLatency,
    } = useAudioLatency();

    const {
        selectedAsioInputChannels,
        selectedAsioOutputChannels,
        asioChannelsError,
        setAsioChannelsError,
        asioInputChannelDropdownOptions,
        asioOutputChannelDropdownOptions,
        loadAsioChannelCapabilities,
        handleAsioInputChannelsChange,
        handleAsioOutputChannelsChange,
    } = useAsioChannels(loadBufferLatency);

    const inputOptions = inputs.map(d => ({label: `${d.name} (${d.sample_rate} Hz)`, value: d.id}));
    const outputOptions = outputs.map(d => ({label: `${d.name} (${d.sample_rate} Hz)`, value: d.id}));

    async function handleInputChange(value: string | number) {
        const id = String(value);
        setSelectedInput(id);
        await updateInputDevice(id);
        await loadBufferLatency();
    }

    async function handleOutputChange(value: string | number) {
        const id = String(value);
        setSelectedOutput(id);
        await updateOutputDevice(id);
        await loadBufferLatency();
    }

    async function handleAsioDeviceChange(value: string | number) {
        const id = String(value);
        setSelectedInput(id);
        setSelectedOutput(id);
        await updateInputDevice(id);
        await loadAsioChannelCapabilities(id);
        await loadBufferLatency();
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
            console.error("Failed to switch audio driver:", err);
            setDriverError(formatDriverSwitchError(driver, err));
        }
    }

    useEffect(() => {
        if (isAsioMode) {
            const inputDeviceExists = inputs.some(d => d.id === selectedInput);

            if ((!selectedInput || !inputDeviceExists) && inputs.length > 0) {
                const initialAsioDevice = inputs[0];
                setSelectedInput(initialAsioDevice.id);
                setSelectedOutput(initialAsioDevice.id);
                void loadAsioChannelCapabilities(initialAsioDevice.id);
            } else if (selectedInput && inputDeviceExists) {
                void loadAsioChannelCapabilities(selectedInput);
            } else {
                void loadAsioChannelCapabilities("");
            }
            return;
        }

        if (!selectedInput && inputs.length > 0) {
            setSelectedInput(inputs[0].id);
        }
        if (!selectedOutput && outputs.length > 0) {
            setSelectedOutput(outputs[0].id);
        }
    }, [isAsioMode, inputs, outputs, selectedInput, selectedOutput, setSelectedInput, setSelectedOutput, loadAsioChannelCapabilities]);

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
                        <MidiSection/>
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
                        handleAsioInputChannelsChange={(value) => handleAsioInputChannelsChange(value, selectedInput)}
                        asioOutputChannelDropdownOptions={asioOutputChannelDropdownOptions}
                        selectedAsioOutputChannels={selectedAsioOutputChannels}
                        handleAsioOutputChannelsChange={(value) => handleAsioOutputChannelsChange(value, selectedInput)}
                    />
                </Box>
            </Box>
        </Box>
    );
}
