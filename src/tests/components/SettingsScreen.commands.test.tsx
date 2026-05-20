// @vitest-environment jsdom
import React from "react";
import {cleanup, render, screen, waitFor} from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import {afterEach, beforeEach, describe, expect, it, vi} from "vitest";
import {SettingsScreen} from "../../screens/SettingsScreen";
import * as commands from "../../domain/commands";

const uiState = vi.hoisted(() => ({
    selectedInputId: "in-1",
    selectedOutputId: "out-1",
    developerMode: false,
    setSelectedInputId: vi.fn((id: string) => {
        uiState.selectedInputId = id;
    }),
    setSelectedOutputId: vi.fn((id: string) => {
        uiState.selectedOutputId = id;
    }),
    setDeveloperMode: vi.fn((v: boolean) => {
        uiState.developerMode = v;
    }),
}));

const useUIStoreMock = vi.hoisted(() => vi.fn((selector: (s: typeof uiState) => unknown) => selector(uiState)));
const useAudioDevicesMock = vi.hoisted(() => vi.fn());
const useUpdateAudioDevicesMock = vi.hoisted(() => vi.fn());
const updateInputDeviceMock = vi.hoisted(() => vi.fn().mockResolvedValue(undefined));
const updateOutputDeviceMock = vi.hoisted(() => vi.fn().mockResolvedValue(undefined));

vi.mock("../../state/UIStore.tsx", () => ({
    useUIStore: useUIStoreMock,
}));

vi.mock("../../hooks/useAudioDevices.ts", () => ({
    useAudioDevices: useAudioDevicesMock,
}));

vi.mock("../../hooks/useUpdateAudioDevices.ts", () => ({
    useUpdateAudioDevices: useUpdateAudioDevicesMock,
}));

vi.mock("../../components/selection/DropdownSelector.tsx", () => ({
    DropdownSelector: ({title, label, options, onSelectionChange}: any) => (
        <div>
            <div>{`selected:${String(label)}`}</div>
            <button
                onClick={() => {
                    if (options?.length) {
                        onSelectionChange(options[0].value);
                    }
                }}
            >
                {`${title || label}-select-first`}
            </button>
            <button
                onClick={() => {
                    if (options?.length) {
                        onSelectionChange(options[options.length - 1].value);
                    }
                }}
            >
                {`${title || label}-select-last`}
            </button>
        </div>
    ),
}));

// Mock the child structural subcomponents to verify how props are passed down from SettingsScreen
vi.mock("../../components/LatencySection.tsx", () => ({
    LatencySection: ({
                         bufferSizeOptions,
                         handleBufferSizeChange,
                         bufferSizeSaving,
                         handleMeasureRoundTripLatency,
                         roundTripLoading,
                         roundTripError,
                     }: any) => (
        <div>
            <button
                onClick={() => handleBufferSizeChange(bufferSizeOptions[1]?.value || 512)}
                aria-label="Change Buffer Size"
            >
                Change Buffer Size
            </button>
            {bufferSizeSaving && <div>Saving Buffer Size...</div>}

            <button
                onClick={handleMeasureRoundTripLatency}
                aria-label={roundTripLoading ? "Measuring..." : "Measure Round-Trip"}
            >
                {roundTripLoading ? "Measuring..." : "Measure Round-Trip"}
            </button>
            {roundTripError && <div>{roundTripError}</div>}
        </div>
    ),
}));

vi.mock("../../components/DeviceroutingSection.tsx", () => ({
    DeviceRoutingSection: ({handleInputChange, handleOutputChange}: any) => (
        <div>
            <button onClick={() => handleInputChange("in-2")} aria-label="Input Device-select-last">
                Input Device-select-last
            </button>
            <button onClick={() => handleOutputChange("out-2")} aria-label="Output Device-select-last">
                Output Device-select-last
            </button>
        </div>
    ),
}));

vi.mock("../../components/SampleRateWarning.tsx", () => ({
    SampleRateWarning: () => <div />,
}));

vi.mock("../../domain/commands.ts", () => ({
    measureBufferLatency: vi.fn(),
    getBufferSizeFrames: vi.fn(),
    setBufferSizeFrames: vi.fn(),
    measureRoundTripLatency: vi.fn(),
    getAvailableAudioDrivers: vi.fn().mockResolvedValue(["Default"]),
    getSelectedAudioDriver: vi.fn().mockResolvedValue("Default"),
}));

describe("SettingsScreen command interactions", () => {
    beforeEach(() => {
        vi.clearAllMocks();
        uiState.selectedInputId = "in-1";
        uiState.selectedOutputId = "out-1";
        uiState.developerMode = false;

        useAudioDevicesMock.mockReturnValue({
            inputs: [
                {id: "in-1", name: "Input A", sample_rate: 44100},
                {id: "in-2", name: "Input B", sample_rate: 48000},
            ],
            outputs: [
                {id: "out-1", name: "Output A", sample_rate: 44100},
                {id: "out-2", name: "Output B", sample_rate: 48000},
            ],
            isLoading: false,
            error: null,
            refresh: vi.fn(),
        });

        useUpdateAudioDevicesMock.mockReturnValue({
            updateInputDevice: updateInputDeviceMock,
            updateOutputDevice: updateOutputDeviceMock,
            isSetting: false,
            error: null,
        });

        vi.mocked(commands.measureBufferLatency).mockResolvedValue({
            input_buffer_latency_ms: 1,
            output_buffer_latency_ms: 2,
            total_buffer_latency_ms: 3,
        } as any);

        vi.mocked(commands.getBufferSizeFrames).mockResolvedValue(256);
        vi.mocked(commands.setBufferSizeFrames).mockResolvedValue(undefined);
        vi.mocked(commands.measureRoundTripLatency).mockResolvedValue({
            is_valid: true,
            latency_ms: 7.5,
            error: null,
        } as any);
    });

    afterEach(() => {
        cleanup();
    });

    it("auto-fires setBufferSizeFrames when buffer size value changes", async () => {
        // Arrange
        const user = userEvent.setup();
        render(<SettingsScreen />);

        // Initial setup fetch
        await waitFor(() => {
            expect(commands.getBufferSizeFrames).toHaveBeenCalledTimes(1);
        });

        // Act - changing state triggers the auto-apply useEffect hook
        await user.click(screen.getByRole("button", {name: "Change Buffer Size"}));

        // Assert
        await waitFor(() => {
            expect(commands.setBufferSizeFrames).toHaveBeenCalledWith({frames: 128});
            expect(commands.measureBufferLatency).toHaveBeenCalled();
        });
    });

    it("shows loading indicator while buffer size auto-save is in-flight", async () => {
        // Arrange
        let resolveApply: () => void = () => undefined;
        vi.mocked(commands.setBufferSizeFrames).mockImplementationOnce(
            () => new Promise<void>((resolve) => {
                resolveApply = resolve;
            })
        );
        const user = userEvent.setup();
        render(<SettingsScreen />);

        await waitFor(() => {
            expect(commands.getBufferSizeFrames).toHaveBeenCalledTimes(1);
        });

        // Act
        await user.click(screen.getByRole("button", {name: "Change Buffer Size"}));

        // Assert
        expect(screen.getByText("Saving Buffer Size...")).toBeTruthy();

        resolveApply();
        await waitFor(() => {
            expect(screen.queryByText("Saving Buffer Size...")).toBeNull();
        });
    });

    it("fires measureRoundTripLatency when Measure Round-Trip is pressed", async () => {
        // Arrange
        const user = userEvent.setup();
        render(<SettingsScreen />);

        // Act
        await user.click(screen.getByRole("button", {name: "Measure Round-Trip"}));

        // Assert
        await waitFor(() => {
            expect(commands.measureRoundTripLatency).toHaveBeenCalledTimes(1);
        });
    });

    it("shows Measuring... while round-trip measurement is in-flight", async () => {
        // Arrange
        let resolveMeasure: (v: any) => void = () => undefined;
        vi.mocked(commands.measureRoundTripLatency).mockImplementationOnce(
            () => new Promise((resolve) => {
                resolveMeasure = resolve;
            }) as any
        );
        const user = userEvent.setup();
        render(<SettingsScreen />);

        // Act
        await user.click(screen.getByRole("button", {name: "Measure Round-Trip"}));

        // Assert
        expect(screen.getByRole("button", {name: "Measuring..."})).toBeTruthy();

        resolveMeasure({is_valid: true, latency_ms: 8, error: null});
        await waitFor(() => {
            expect(screen.getByRole("button", {name: "Measure Round-Trip"})).toBeTruthy();
        });
    });

    it("updates input and output routing handlers when dropdown actions are triggered", async () => {
        // Arrange
        const user = userEvent.setup();
        render(<SettingsScreen />);

        // Act
        await user.click(screen.getByRole("button", {name: "Input Device-select-last"}));
        await user.click(screen.getByRole("button", {name: "Output Device-select-last"}));

        // Assert
        await waitFor(() => {
            expect(uiState.setSelectedInputId).toHaveBeenCalledWith("in-2");
            expect(updateInputDeviceMock).toHaveBeenCalledWith("in-2");
            expect(uiState.setSelectedOutputId).toHaveBeenCalledWith("out-2");
            expect(updateOutputDeviceMock).toHaveBeenCalledWith("out-2");
        });
    });


    it("shows round-trip invalid response error message", async () => {
        // Arrange
        vi.mocked(commands.measureRoundTripLatency).mockResolvedValueOnce({
            is_valid: false,
            latency_ms: 0,
            error: "loopback missing",
        } as any);
        const user = userEvent.setup();
        render(<SettingsScreen />);

        // Act
        await user.click(screen.getByRole("button", {name: "Measure Round-Trip"}));

        // Assert
        await waitFor(() => {
            expect(screen.getByText("loopback missing")).toBeTruthy();
        });
    });
});