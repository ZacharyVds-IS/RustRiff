import {Box} from "@mui/material";
import {DropdownSelector} from "./selection/DropdownSelector.tsx";

interface Option {
    label: string;
    value: string | number;
}

interface DeviceRoutingSectionProps {
    driverOptions: Option[];
    selectedDriver: string;
    handleDriverChange: (value: string | number) => void;
    isAsioMode: boolean;
    inputOptions: Option[];
    selectedInput: string;
    handleAsioDeviceChange: (value: string | number) => void;
    handleInputChange: (value: string | number) => void;
    outputOptions: Option[];
    selectedOutput: string;
    handleOutputChange: (value: string | number) => void;
    asioInputChannelDropdownOptions: Option[];
    selectedAsioInputChannels: number | null;
    handleAsioInputChannelsChange: (value: string | number) => void;
    asioOutputChannelDropdownOptions: Option[];
    selectedAsioOutputChannels: number | null;
    handleAsioOutputChannelsChange: (value: string | number) => void;
}

export function DeviceRoutingSection({
                                         driverOptions,
                                         selectedDriver,
                                         handleDriverChange,
                                         isAsioMode,
                                         inputOptions,
                                         selectedInput,
                                         handleAsioDeviceChange,
                                         handleInputChange,
                                         outputOptions,
                                         selectedOutput,
                                         handleOutputChange,
                                         asioInputChannelDropdownOptions,
                                         selectedAsioInputChannels,
                                         handleAsioInputChannelsChange,
                                         asioOutputChannelDropdownOptions,
                                         selectedAsioOutputChannels,
                                         handleAsioOutputChannelsChange,
                                     }: DeviceRoutingSectionProps) {

    const isChannelsOperational = asioInputChannelDropdownOptions.length > 0 && asioOutputChannelDropdownOptions.length > 0;

    return (
        <Box sx={{ flex: 1, minWidth: 0, display: "flex", flexDirection: "column", gap: 2, pl: 2, overflowY: "auto", overflowX: "hidden" }}>
            <DropdownSelector
                title="Audio Driver"
                label="Select audio driver"
                options={driverOptions}
                selectedValue={selectedDriver}
                onSelectionChange={handleDriverChange}
            />

            <DropdownSelector
                title={isAsioMode ? "ASIO Device" : "Input Device"}
                label={isAsioMode ? "Select ASIO device" : "Select input device"}
                options={inputOptions}
                selectedValue={selectedInput}
                onSelectionChange={isAsioMode ? handleAsioDeviceChange : handleInputChange}
            />

            {!isAsioMode && (
                <DropdownSelector
                    title="Output Device"
                    label="Select output device"
                    options={outputOptions}
                    selectedValue={selectedOutput}
                    onSelectionChange={handleOutputChange}
                />
            )}

            {isAsioMode && selectedInput && (
                <>
                    <DropdownSelector
                        title="ASIO Input Channel"
                        label={isChannelsOperational ? "Input channel" : "No device channels loaded"}
                        options={asioInputChannelDropdownOptions}
                        selectedValue={selectedAsioInputChannels ?? ""}
                        onSelectionChange={handleAsioInputChannelsChange}
                        disabled={!isChannelsOperational}
                    />

                    <DropdownSelector
                        title="ASIO Output Channel"
                        label={isChannelsOperational ? "Output channel" : "No device channels loaded"}
                        options={asioOutputChannelDropdownOptions}
                        selectedValue={selectedAsioOutputChannels ?? ""}
                        onSelectionChange={handleAsioOutputChannelsChange}
                        disabled={!isChannelsOperational}
                    />
                </>
            )}
        </Box>
    );
}