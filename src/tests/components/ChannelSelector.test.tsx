// @vitest-environment jsdom
import React from "react";
import {cleanup, render, screen} from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import {afterEach, beforeEach, describe, expect, it, vi} from "vitest";
import {ChannelSelector} from "../../components/ChannelSelector";

const dropdownMock = vi.hoisted(() => vi.fn());

vi.mock("../../components/selection/DropdownSelector.tsx", () => ({
    DropdownSelector: (props: any) => {
        dropdownMock(props);
        return (
            <div>
                <button onClick={() => props.onSelectionChange("2")}>pick-string</button>
                <button onClick={() => props.onSelectionChange("invalid")}>pick-invalid</button>
                <button onClick={() => props.onSelectionChange(3)}>pick-number</button>
                <button onClick={() => props.onAdd?.()}>add-channel</button>
            </div>
        );
    },
}));

describe("ChannelSelector", () => {
    const channels = [
        {label: "Clean", value: "1"},
        {label: "Lead", value: "2"},
        {label: "Crunch", value: "3"},
    ];

    beforeEach(() => {
        vi.clearAllMocks();
    });

    afterEach(() => {
        cleanup();
    });

    describe("success_path", () => {
        it("passes the current selected channel value to DropdownSelector", () => {
            // Arrange
            const onChannelChange = vi.fn();

            // Act
            render(
                <ChannelSelector
                    channels={channels}
                    currentChannelId={"2"}
                    onChannelChange={onChannelChange}
                    onAdd={vi.fn()}
                />
            );

            // Assert
            const props = dropdownMock.mock.calls[0][0];
            expect(props.selectedValue).toBe("2");
            expect(props.hasBorder).toBe(false);
            expect(props.hasLabel).toBe(false);
            expect(props.options).toEqual(channels);
        });

        it("passes empty selectedValue when current channel id does not exist", () => {
            // Arrange & Act
            render(
                <ChannelSelector
                    channels={channels}
                    currentChannelId={"999"}
                    onChannelChange={vi.fn()}
                    onAdd={vi.fn()}
                />
            );

            // Assert
            const props = dropdownMock.mock.calls[0][0];
            expect(props.selectedValue).toBe("");
        });

        it("fires onChannelChange with the string ID when a value is selected", async () => {
            // Arrange
            const onChannelChange = vi.fn();
            const onAdd = vi.fn();
            const user = userEvent.setup();

            render(
                <ChannelSelector
                    channels={channels}
                    currentChannelId={"1"}
                    onChannelChange={onChannelChange}
                    onAdd={onAdd}
                />
            );

            // Act
            await user.click(screen.getByRole("button", {name: "pick-string"}));

            // Assert
            expect(onChannelChange).toHaveBeenCalledWith("2");
            expect(typeof onChannelChange.mock.calls[0][0]).toBe("string");
        });

        it("fires onChannelChange with numeric values unchanged", async () => {
            // Arrange
            const onChannelChange = vi.fn();
            const onAdd = vi.fn();
            const user = userEvent.setup();

            render(
                <ChannelSelector
                    channels={channels}
                    currentChannelId={"2"}
                    onChannelChange={onChannelChange}
                    onAdd={onAdd}
                />
            );

            // Act
            await user.click(screen.getByRole("button", {name: "pick-number"}));

            // Assert
            expect(onChannelChange).toHaveBeenCalledWith("3");
        });

        it("fires onAdd when the add action is triggered", async () => {
            // Arrange
            const onChannelChange = vi.fn();
            const onAdd = vi.fn();
            const user = userEvent.setup();

            render(
                <ChannelSelector
                    channels={channels}
                    currentChannelId={"2"}
                    onChannelChange={onChannelChange}
                    onAdd={onAdd}
                />
            );

            // Act
            await user.click(screen.getByRole("button", {name: "add-channel"}));

            // Assert
            expect(onAdd).toHaveBeenCalledTimes(1);
        });
    });
});
