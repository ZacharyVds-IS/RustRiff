// @vitest-environment jsdom
import React from "react";
import {cleanup, render, screen} from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import {afterEach, beforeEach, describe, expect, it, vi} from "vitest";
import {EffectChain} from "../../components/EffectChain";
import {EffectDto} from "../../domain";

const storeState = vi.hoisted(() => ({
    startEditingChainOrder: vi.fn(),
    cancelEditingChainOrder: vi.fn(),
    applyChangesToChainOrder: vi.fn().mockResolvedValue(undefined),
    moveEffect: vi.fn(),
    addEffect: vi.fn(),
    removeEffect: vi.fn(),
}));

const useAmpStoreMock = vi.hoisted(() => {
    const fn: any = vi.fn((selector?: (state: any) => any) =>
        typeof selector === "function" ? selector(storeState) : storeState
    );
    fn.getState = () => storeState;
    return fn;
});

vi.mock("../../state/AmpConfigStore.tsx", () => ({
    useAmpStore: useAmpStoreMock,
}));

vi.mock("@hello-pangea/dnd", () => ({
    DragDropContext: ({children, onDragEnd}: any) => (
        <div>
            {children}
            <button onClick={() => onDragEnd({source: {index: 0}, destination: {index: 1}})}>drag-valid</button>
            <button onClick={() => onDragEnd({source: {index: 0}, destination: null})}>drag-missing-destination</button>
        </div>
    ),
    Droppable: ({children}: any) => children({droppableProps: {}, innerRef: () => undefined, placeholder: null}),
    Draggable: ({children}: any) => children(
        {innerRef: () => undefined, draggableProps: {style: {}}, dragHandleProps: {}},
        {isDragging: false}
    ),
}));

vi.mock("../../components/AmpBox.tsx", () => ({
    AmpBox: ({onSelectionChange, isAmpSelected}: any) => (
        <div>
            <span>{isAmpSelected() ? "amp-selected" : "amp-not-selected"}</span>
            <button onClick={() => onSelectionChange("amp")}>amp-item</button>
        </div>
    ),
}));

vi.mock("../../components/EffectPedalPreview.tsx", () => ({
    EffectPedalPreview: () => <div>pedal-preview</div>,
}));

vi.mock("../../components/CabinetPreview.tsx", () => ({
    CabinetPreview: () => <div>cab-preview</div>,
}));

vi.mock("../../components/dialogs/AddEffectDialog.tsx", () => ({
    AddEffectDialog: ({open, onCreate}: any) =>
        open ? (
            <button
                onClick={() =>
                    onCreate({
                        kind: "Delay",
                        data: {id: 77, name: "Mock Delay", is_active: true, delay_time: 200, level: 0.4, color: "#333"},
                    })
                }
            >
                create-effect
            </button>
        ) : null,
}));

vi.mock("../../components/dialogs/ConfirmationDialog.tsx", () => ({
    ConfirmationDialog: ({open, onConfirm, onClose}: any) =>
        open ? <button onClick={() => { onConfirm(); onClose(); }}>confirm-remove</button> : null,
}));

function createMockEffect(id: number, name: string): EffectDto {
    return {
        kind: "Delay",
        data: {id, name, is_active: true, delay_time: 240, level: 0.5, color: "#3498db"},
    } as EffectDto;
}

describe("EffectChain", () => {
    const effects = [createMockEffect(1, "Delay One"), createMockEffect(2, "Delay Two")];

    beforeEach(() => {
        vi.clearAllMocks();
    });

    afterEach(() => {
        cleanup();
    });

    describe("success_path", () => {
        it("uses amp selection identity to compute AmpBox selected state", () => {
            // Arrange & Act
            const {rerender} = render(
                <EffectChain
                    effects={effects}
                    selected={"amp"}
                    onSelectionChange={vi.fn()}
                    onReorderOpen={vi.fn()}
                />
            );

            // Assert
            expect(screen.getByText("amp-selected")).toBeTruthy();

            rerender(
                <EffectChain
                    effects={effects}
                    selected={effects[0]}
                    onSelectionChange={vi.fn()}
                    onReorderOpen={vi.fn()}
                />
            );
            expect(screen.getByText("amp-not-selected")).toBeTruthy();
        });

        it("fires onSelectionChange with effect and index when an effect is clicked", async () => {
            // Arrange
            const onSelectionChange = vi.fn();
            const user = userEvent.setup();
            render(
                <EffectChain
                    effects={effects}
                    selected={"amp"}
                    onSelectionChange={onSelectionChange}
                    onReorderOpen={vi.fn()}
                />
            );

            // Act
            await user.click(screen.getByText("Delay Two"));

            // Assert
            expect(onSelectionChange).toHaveBeenCalledWith(effects[1], 1);
        });

        it("fires reorder-start actions when Edit Order is pressed", async () => {
            // Arrange
            const user = userEvent.setup();
            const onSelectionChange = vi.fn();
            const onReorderOpen = vi.fn();

            render(
                <EffectChain
                    effects={effects}
                    selected={"amp"}
                    onSelectionChange={onSelectionChange}
                    onReorderOpen={onReorderOpen}
                />
            );

            // Act
            await user.click(screen.getByRole("button", {name: "Edit Order"}));

            // Assert
            expect(storeState.startEditingChainOrder).toHaveBeenCalledTimes(1);
            expect(onReorderOpen).toHaveBeenCalledWith(true);
            expect(screen.getByRole("button", {name: "Apply Changes"})).toBeTruthy();
        });

        it("fires reorder-cancel actions when Cancel is pressed", async () => {
            // Arrange
            const user = userEvent.setup();
            const onReorderOpen = vi.fn();

            render(
                <EffectChain
                    effects={effects}
                    selected={"amp"}
                    onSelectionChange={vi.fn()}
                    onReorderOpen={onReorderOpen}
                />
            );
            await user.click(screen.getByRole("button", {name: "Edit Order"}));

            // Act
            await user.click(screen.getByRole("button", {name: "Cancel"}));

            // Assert
            expect(storeState.cancelEditingChainOrder).toHaveBeenCalledTimes(1);
            expect(onReorderOpen).toHaveBeenLastCalledWith(false);
        });

        it("fires apply order command when Apply Changes is pressed", async () => {
            // Arrange
            const user = userEvent.setup();
            const onReorderOpen = vi.fn();

            render(
                <EffectChain
                    effects={effects}
                    selected={"amp"}
                    onSelectionChange={vi.fn()}
                    onReorderOpen={onReorderOpen}
                />
            );
            await user.click(screen.getByRole("button", {name: "Edit Order"}));

            // Act
            await user.click(screen.getByRole("button", {name: "Apply Changes"}));

            // Assert
            expect(storeState.applyChangesToChainOrder).toHaveBeenCalledTimes(1);
            expect(onReorderOpen).toHaveBeenLastCalledWith(false);
        });

        it("fires moveEffect when left arrow is pressed on selected effect", async () => {
            // Arrange
            const user = userEvent.setup();
            const {container} = render(
                <EffectChain
                    effects={effects}
                    selected={effects[1]}
                    onSelectionChange={vi.fn()}
                    onReorderOpen={vi.fn()}
                />
            );
            await user.click(screen.getByRole("button", {name: "Edit Order"}));
            const left = container.querySelector('[data-testid="KeyboardArrowLeftIcon"]')?.closest("button");
            expect(left).not.toBeNull();

            // Act
            await user.click(left as HTMLButtonElement);

            // Assert
            expect(storeState.moveEffect).toHaveBeenCalledWith(1, 0);
        });

        it("does not move left when selected effect is already first", async () => {
            // Arrange
            const user = userEvent.setup();
            const {container} = render(
                <EffectChain
                    effects={effects}
                    selected={effects[0]}
                    onSelectionChange={vi.fn()}
                    onReorderOpen={vi.fn()}
                />
            );
            await user.click(screen.getByRole("button", {name: "Edit Order"}));
            const left = container.querySelector('[data-testid="KeyboardArrowLeftIcon"]')?.closest("button");
            expect(left).not.toBeNull();

            // Act
            await user.click(left as HTMLButtonElement);

            // Assert
            expect(storeState.moveEffect).not.toHaveBeenCalled();
        });

        it("calls moveEffect on drag end when destination exists", async () => {
            // Arrange
            const user = userEvent.setup();
            render(
                <EffectChain
                    effects={effects}
                    selected={"amp"}
                    onSelectionChange={vi.fn()}
                    onReorderOpen={vi.fn()}
                />
            );

            // Act
            await user.click(screen.getByRole("button", {name: "drag-valid"}));

            // Assert
            expect(storeState.moveEffect).toHaveBeenCalledWith(0, 1);
        });

        it("does not call moveEffect on drag end when destination is missing", async () => {
            // Arrange
            const user = userEvent.setup();
            render(
                <EffectChain
                    effects={effects}
                    selected={"amp"}
                    onSelectionChange={vi.fn()}
                    onReorderOpen={vi.fn()}
                />
            );

            // Act
            await user.click(screen.getByRole("button", {name: "drag-missing-destination"}));

            // Assert
            expect(storeState.moveEffect).not.toHaveBeenCalled();
        });

        it("fires removeEffect and re-selects amp when remove is confirmed", async () => {
            // Arrange
            const user = userEvent.setup();
            const onSelectionChange = vi.fn();
            const {container} = render(
                <EffectChain
                    effects={effects}
                    selected={effects[0]}
                    onSelectionChange={onSelectionChange}
                    onReorderOpen={vi.fn()}
                />
            );
            const remove = container.querySelector('[data-testid="DeleteIcon"]')?.closest("button");
            expect(remove).not.toBeNull();

            // Act
            await user.click(remove as HTMLButtonElement);
            await user.click(screen.getAllByRole("button", {name: "confirm-remove"})[0]);

            // Assert
            expect(storeState.removeEffect).toHaveBeenCalledWith(1);
            expect(onSelectionChange).toHaveBeenCalledWith("amp");
        });

        it("fires addEffect via store when add flow is completed", async () => {
            // Arrange
            const user = userEvent.setup();
            const {container} = render(
                <EffectChain
                    effects={effects}
                    selected={"amp"}
                    onSelectionChange={vi.fn()}
                    onReorderOpen={vi.fn()}
                />
            );
            const addIcon = container.querySelector('[data-testid="AddCircleIcon"]');
            const addButton = addIcon?.closest("button");
            expect(addButton).not.toBeNull();

            // Act
            await user.click(addButton as HTMLButtonElement);
            await user.click(screen.getByRole("button", {name: "create-effect"}));

            // Assert
            expect(storeState.addEffect).toHaveBeenCalledTimes(1);
            expect(storeState.addEffect).toHaveBeenCalledWith(
                expect.objectContaining({kind: "Delay"})
            );
        });
    });
});


