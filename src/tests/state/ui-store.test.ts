import {beforeEach, describe, expect, it, vi} from "vitest";
import {useUIStore} from "../../state/UIStore";

function resetUIStore() {
    useUIStore.setState({
        developerMode: false,
        selectedInputId: "",
        selectedOutputId: "",
    });
}

describe("UIStore", () => {
    beforeEach(() => {
        resetUIStore();
    });

    describe("success_path", () => {
        it("starts with expected default values", async () => {
            vi.resetModules();
            const fresh = await import("../../state/UIStore");
            const state = fresh.useUIStore.getState();

            expect(state.developerMode).toBe(false);
            expect(state.selectedInputId).toBe("");
            expect(state.selectedOutputId).toBe("");
        });

        it("setDeveloperMode toggles developer mode", () => {
            // Arrange
            const setDeveloperMode = useUIStore.getState().setDeveloperMode;

            // Act
            setDeveloperMode(true);

            // Assert
            expect(useUIStore.getState().developerMode).toBe(true);
            expect(useUIStore.getState().selectedInputId).toBe("");
            expect(useUIStore.getState().selectedOutputId).toBe("");
        });

        it("setSelectedInputId updates selected input id", () => {
            // Arrange
            const setSelectedInputId = useUIStore.getState().setSelectedInputId;

            // Act
            setSelectedInputId("input-123");

            // Assert
            expect(useUIStore.getState().selectedInputId).toBe("input-123");
            expect(useUIStore.getState().selectedOutputId).toBe("");
        });

        it("setSelectedOutputId updates selected output id", () => {
            // Arrange
            const setSelectedOutputId = useUIStore.getState().setSelectedOutputId;

            // Act
            setSelectedOutputId("output-456");

            // Assert
            expect(useUIStore.getState().selectedOutputId).toBe("output-456");
            expect(useUIStore.getState().selectedInputId).toBe("");
        });
    });

    describe("failure_path", () => {
        it("accepts empty ids without throwing (clear selection path)", () => {
            // Arrange
            const setSelectedInputId = useUIStore.getState().setSelectedInputId;
            const setSelectedOutputId = useUIStore.getState().setSelectedOutputId;

            // Act
            setSelectedInputId("");
            setSelectedOutputId("");

            // Assert
            expect(useUIStore.getState().selectedInputId).toBe("");
            expect(useUIStore.getState().selectedOutputId).toBe("");
        });
    });
});

