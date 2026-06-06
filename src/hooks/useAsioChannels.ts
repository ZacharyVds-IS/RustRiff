import {useState} from "react";
import * as commands from "../domain/commands";

export function useAsioChannels(loadBufferLatency: () => Promise<void>) {
    const [asioInputChannelOptions, setAsioInputChannelOptions] = useState<number[]>([]);
    const [asioOutputChannelOptions, setAsioOutputChannelOptions] = useState<number[]>([]);
    const [selectedAsioInputChannels, setSelectedAsioInputChannels] = useState<number | null>(null);
    const [selectedAsioOutputChannels, setSelectedAsioOutputChannels] = useState<number | null>(null);
    const [asioChannelsError, setAsioChannelsError] = useState<string | null>(null);

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

    async function applyAsioChannelConfig(deviceId: string, inputChannels: number, outputChannels: number) {
        if (!deviceId) return;

        setAsioChannelsError(null);
        try {
            await commands.setAsioChannelConfig({
                deviceId,
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

    async function handleAsioInputChannelsChange(value: string | number, deviceId: string) {
        const inputChannels = Number(value);
        const outputChannels =
            selectedAsioOutputChannels ?? asioOutputChannelOptions[0] ?? inputChannels;
        await applyAsioChannelConfig(deviceId, inputChannels, outputChannels);
    }

    async function handleAsioOutputChannelsChange(value: string | number, deviceId: string) {
        const outputChannels = Number(value);
        const inputChannels =
            selectedAsioInputChannels ?? asioInputChannelOptions[0] ?? outputChannels;
        await applyAsioChannelConfig(deviceId, inputChannels, outputChannels);
    }

    const asioInputChannelDropdownOptions = asioInputChannelOptions.map((channels) => ({
        label: `channel ${channels}`,
        value: channels,
    }));
    const asioOutputChannelDropdownOptions = asioOutputChannelOptions.map((channels) => ({
        label: `channel ${channels}`,
        value: channels,
    }));

    return {
        asioInputChannelOptions,
        asioOutputChannelOptions,
        selectedAsioInputChannels,
        selectedAsioOutputChannels,
        asioChannelsError,
        setAsioChannelsError,
        asioInputChannelDropdownOptions,
        asioOutputChannelDropdownOptions,
        loadAsioChannelCapabilities,
        handleAsioInputChannelsChange,
        handleAsioOutputChannelsChange,
    };
}
