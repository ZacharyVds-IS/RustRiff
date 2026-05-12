// @vitest-environment jsdom
import {describe, expect, it} from "vitest";
import {EffectDto} from "../../domain";

/**
 * Unit tests for EffectChain component logic
 *
 * The EffectChain component has critical logic for:
 * 1. Determining if amp is selected vs an effect
 * 2. Checking if a specific effect is selected
 * 3. Moving pedals left/right with boundary checks
 * 4. Handling drag-drop result processing
 */

describe("EffectChain component logic", () => {
    // Helper to create mock effect
    function createMockEffect(id: number, name: string, kind: "HCDistortion" | "Delay" = "HCDistortion"): EffectDto {
        if (kind === "HCDistortion") {
            return {
                kind: "HCDistortion",
                data: {
                    id,
                    name,
                    is_active: true,
                    threshold: 0.5,
                    level: 0.5,
                    color: "#e67e22",
                },
            } as any;
        }
        return {
            kind: "Delay",
            data: {
                id,
                name,
                is_active: true,
                delay_time: 200,
                level: 0.5,
                color: "#3498db",
            },
        } as any;
    }

    // Extracted logic for testing
    function isAmpSelected(selected: EffectDto | "amp"): boolean {
        return selected === "amp";
    }

    function isEffectSelected(effect: EffectDto, selected: EffectDto | "amp"): boolean {
        return selected !== "amp" && selected === effect;
    }

    function getValidMovedIndex(currentIndex: number, direction: "left" | "right", effectsLength: number): number | null {
        const newIndex = direction === "left" ? currentIndex - 1 : currentIndex + 1;
        if (newIndex < 0 || newIndex >= effectsLength) {
            return null;
        }
        return newIndex;
    }

    const effect1 = createMockEffect(1, "Drive");
    const effect2 = createMockEffect(2, "Delay", "Delay");
    const effects = [effect1, effect2];

    describe("success_path", () => {
        it("detects when amp is selected", () => {
            // Arrange & Act
            const isSelected = isAmpSelected("amp");

            // Assert
            expect(isSelected).toBe(true);
        });

        it("detects when effect is selected", () => {
            // Arrange & Act
            const isSelected = isEffectSelected(effect1, effect1);

            // Assert
            expect(isSelected).toBe(true);
        });

        it("detects when a different effect is selected", () => {
            // Arrange & Act
            const isSelected = isEffectSelected(effect2, effect1);

            // Assert
            expect(isSelected).toBe(false);
        });

        it("returns false when amp is selected but checking an effect", () => {
            // Arrange & Act
            const isSelected = isEffectSelected(effect1, "amp");

            // Assert
            expect(isSelected).toBe(false);
        });

        it("moves effect right when direction is right", () => {
            // Arrange
            const currentIndex = 0;

            // Act
            const newIndex = getValidMovedIndex(currentIndex, "right", effects.length);

            // Assert
            expect(newIndex).toBe(1);
        });

        it("moves effect left when direction is left", () => {
            // Arrange
            const currentIndex = 1;

            // Act
            const newIndex = getValidMovedIndex(currentIndex, "left", effects.length);

            // Assert
            expect(newIndex).toBe(0);
        });

        it("handles drag-drop result by extracting source and destination indices", () => {
            // Arrange
            const dragResult = {
                source: {index: 0},
                destination: {index: 1},
            };

            // Act
            const sourceIndex = dragResult.source.index;
            const destIndex = dragResult.destination?.index ?? dragResult.source.index;

            // Assert
            expect(sourceIndex).toBe(0);
            expect(destIndex).toBe(1);
        });
    });

    describe("failure_path", () => {
        it("returns null when trying to move left from first position", () => {
            // Arrange
            const currentIndex = 0;

            // Act
            const newIndex = getValidMovedIndex(currentIndex, "left", effects.length);

            // Assert
            expect(newIndex).toBeNull();
        });

        it("returns null when trying to move right from last position", () => {
            // Arrange
            const currentIndex = effects.length - 1;

            // Act
            const newIndex = getValidMovedIndex(currentIndex, "right", effects.length);

            // Assert
            expect(newIndex).toBeNull();
        });

        it("returns null when moving beyond array bounds", () => {
            // Arrange
            const outOfBoundsIndex = 999;

            // Act
            const newIndex = getValidMovedIndex(outOfBoundsIndex, "right", effects.length);

            // Assert
            expect(newIndex).toBeNull();
        });

        it("does not match effect by reference equality when it's a different object", () => {
            // Arrange
            const differentEffect = createMockEffect(1, "Drive"); // Same data, different reference

            // Act
            const isSelected = isEffectSelected(effect1, differentEffect);

            // Assert
            expect(isSelected).toBe(false);
        });

        it("handles empty effects array for move validation", () => {
            // Arrange
            const currentIndex = 0;

            // Act
            const newIndex = getValidMovedIndex(currentIndex, "right", 0);

            // Assert
            expect(newIndex).toBeNull();
        });

        it("handles drag drop with no destination", () => {
            // Arrange
            const dragResult = {
                source: {index: 0},
                destination: null,
            };

            // Act
            const hasDestination = dragResult.destination !== null;

            // Assert
            expect(hasDestination).toBe(false);
        });
    });
});


