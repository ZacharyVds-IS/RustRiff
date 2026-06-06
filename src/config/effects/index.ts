import {
    CabinetDto,
    DelayDto,
    type EffectDto,
    getDefaultIrFile,
    HcDistortionDto,
    ScDistortionDto,
    WahDto
} from "../../domain";

export type EffectKind = EffectDto["kind"];

type EffectFactoryMap = {
    [K in EffectKind]: (params: { name: string; color: string; cabinetIrFilePath?: string }) => Extract<EffectDto, {
        kind: K
    }>['data'];
};

/**
 * Synchronous frontend fallback for the default cabinet IR.
 *
 * This must stay aligned with the backend default so the UI remains functional
 * in tests, non-Tauri contexts, and early startup before IPC is available.
 */
export const DEFAULT_CABINET_IR_FILE = "Vox-ac30.wav";

let defaultCabinetIrFilePromise: Promise<string> | null = null;

/**
 * Lazily resolves the backend-configured default cabinet IR filename.
 *
 * The result is cached after the first request. If Tauri IPC is unavailable or
 * the command fails, the synchronous fallback constant is returned instead.
 */
export async function resolveDefaultCabinetIrFile(): Promise<string> {
    if (!defaultCabinetIrFilePromise) {
        defaultCabinetIrFilePromise = getDefaultIrFile().catch((error) => {
            console.warn("Falling back to frontend default cabinet IR file:", error);
            return DEFAULT_CABINET_IR_FILE;
        });
    }

    return defaultCabinetIrFilePromise;
}

export const EFFECT_METADATA: Record<EffectKind, { label: string }> = {
    HCDistortion: {label: "Hard-Clipping Distortion"},
    SCDistortion: {label: "Soft-Clipping Distortion"},
    Cabinet: {label: "Cabinet Simulation"},
    Delay: {label: "Delay"},
    Wah: {label: "Wah"},
};

/**
 * Default short names for each effect type.
 * Used when auto-filling the name field in the Add Effect dialog.
 */
export const EFFECT_SHORT_NAMES: Record<EffectKind, string> = {
    HCDistortion: "Hard-Clipping",
    SCDistortion: "Soft-Clipping",
    Cabinet: "Cabinet",
    Delay: "Delay",
    Wah: "Wah",
};

export const CABINET_CUSTOM_IR_VALUE = "__CUSTOM_FILE__";


export const EFFECT_FACTORIES: EffectFactoryMap = {
    HCDistortion: ({name, color}): HcDistortionDto => ({
        id: "0",
        name,
        color,
        is_active: false,
        threshold: 1,
        level: 0,
    }),
    SCDistortion: ({name, color}): ScDistortionDto => ({
        id: "0",
        name,
        color,
        is_active: false,
        threshold: 1,
        smoothing: 5,
        level: 0,
    }),
    Cabinet: ({name, color, cabinetIrFilePath}): CabinetDto => ({
        id: "0",
        name,
        color,
        is_active: false,
        ir_file_path: cabinetIrFilePath ?? DEFAULT_CABINET_IR_FILE,
    }),
    Delay: ({name, color}): DelayDto => ({
        id: "0",
        name,
        color,
        is_active: false,
        delay_time: 20,
        level: 0.95,
    }),
    Wah: ({name, color}): WahDto => ({
        id: "0",
        name,
        color,
        is_active: false,
        // Aligned with the Rust backend schema changes
        pedal_position: 0.0,
    }),
};