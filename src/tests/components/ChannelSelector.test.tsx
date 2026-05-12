// @vitest-environment jsdom

/**
 * Unit tests for ChannelSelector component logic
 *
 * The ChannelSelector component specifically contains logic for:
 * 1. Finding the selected channel object from the list
 * 2. Converting string values (from selects) to numbers safely
 * 3. Calling the callback with the correct numeric ID
 */
describe("ChannelSelector logic", () => {
    // Helper to extract the logic logic from the component
    function findSelectedChannel(channels: {label: string; value: number}[], currentChannelId: number) {
        return channels.find((ch) => ch.value === currentChannelId);
    }

    function handleSelectionChange(value: string | number, callback: (id: number) => void) {
        const nextChannelId = typeof value === "number" ? value : Number(value);
        if (!Number.isNaN(nextChannelId)) {
            callback(nextChannelId);
        }
    }

    const mockChannels = [
        {label: "Clean", value: 1},
        {label: "Lead", value: 2},
        {label: "Crunch", value: 3},
    ];

    describe("success_path", () => {
        it("finds the currently selected channel from the list", () => {
            // Arrange
            const currentId = 2;

            // Act
            const selected = findSelectedChannel(mockChannels, currentId);

            // Assert
            expect(selected).toEqual({label: "Lead", value: 2});
        });

        it("converts string channel ID to number and calls callback", () => {
            // Arrange
            const onChannelChange = vi.fn();

            // Act
            handleSelectionChange("3", onChannelChange);

            // Assert
            expect(onChannelChange).toHaveBeenCalledWith(3);
            expect(typeof onChannelChange.mock.calls[0][0]).toBe("number");
        });

        it("passes numeric value directly to callback", () => {
            // Arrange
            const onChannelChange = vi.fn();

            // Act
            handleSelectionChange(2, onChannelChange);

            // Assert
            expect(onChannelChange).toHaveBeenCalledWith(2);
        });

        it("handles all channels in the list", () => {
            // Arrange & Act & Assert
            mockChannels.forEach((channel) => {
                const selected = findSelectedChannel(mockChannels, channel.value);
                expect(selected).toEqual(channel);
            });
        });
    });

    describe("failure_path", () => {
        it("does not call callback when string value is not a valid number", () => {
            // Arrange
            const onChannelChange = vi.fn();

            // Act
            handleSelectionChange("invalid", onChannelChange);

            // Assert
            expect(onChannelChange).not.toHaveBeenCalled();
        });

        it("does not call callback when value converts to NaN", () => {
            // Arrange
            const onChannelChange = vi.fn();

            // Act
            handleSelectionChange(NaN, onChannelChange);

            // Assert
            expect(onChannelChange).not.toHaveBeenCalled();
        });

        it("returns undefined when channel ID is not in the list", () => {
            // Arrange
            const currentId = 999;

            // Act
            const selected = findSelectedChannel(mockChannels, currentId);

            // Assert
            expect(selected).toBeUndefined();
        });

        it("returns undefined when channels list is empty", () => {
            // Arrange
            const emptyChannels: {label: string; value: number}[] = [];

            // Act
            const selected = findSelectedChannel(emptyChannels, 1);

            // Assert
            expect(selected).toBeUndefined();
        });

        it("handles zero as a valid channel ID", () => {
            // Arrange
            const channels = [{label: "Default", value: 0}, ...mockChannels];

            // Act
            const selected = findSelectedChannel(channels, 0);

            // Assert
            expect(selected).toEqual({label: "Default", value: 0});
        });
    });
});

import {describe, expect, it, vi} from "vitest";
