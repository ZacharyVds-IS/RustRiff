import {z} from "zod";
import {CABINET_CUSTOM_IR_VALUE, EFFECT_FACTORIES, type EffectKind} from "../effects";

export const DEFAULT_ADD_EFFECT_FORM_VALUES = {
    selectedEffect: "",
    name: "",
    color: "#ff4400",
    cabinetIrChoice: "",
    customCabinetIrFile: undefined as FileList | undefined,
};

export const addEffectSchema = z.object({
    selectedEffect: z
        .string()
        .min(1, "Please select an effect type")
        .refine((value) => value in EFFECT_FACTORIES, "Invalid effect type"),
    name: z
        .string()
        .trim()
        .min(1, "Name is required")
        .max(15, "Name must be 15 characters or fewer"),
    color: z
        .string()
        .refine(
            (value) => !value || /^#[0-9A-Fa-f]{6}$/.test(value),
            "Color must be a valid hex value"
        )
        .default("#ff4400")
        .transform((value) => value || "#ff4400"),
    cabinetIrChoice: z.string().optional(),
    customCabinetIrFile: z.custom<FileList | undefined>((value) => {
        return value === undefined || value instanceof FileList;
    }).optional(),
}).superRefine((values, context) => {
    if (values.selectedEffect !== "Cabinet") {
        return;
    }

    if (!values.cabinetIrChoice) {
        context.addIssue({
            code: z.ZodIssueCode.custom,
            path: ["cabinetIrChoice"],
            message: "Please select a cabinet IR option",
        });
        return;
    }

    if (values.cabinetIrChoice === CABINET_CUSTOM_IR_VALUE) {
        if (!values.customCabinetIrFile || values.customCabinetIrFile.length === 0) {
            context.addIssue({
                code: z.ZodIssueCode.custom,
                path: ["customCabinetIrFile"],
                message: "Please choose an IR file",
            });
            return;
        }

        const fileName = values.customCabinetIrFile[0]?.name ?? "";
        if (!fileName.toLowerCase().endsWith(".wav")) {
            context.addIssue({
                code: z.ZodIssueCode.custom,
                path: ["customCabinetIrFile"],
                message: "Only .wav IR files are supported",
            });
        }
    }
});

export type AddEffectFormValues = z.infer<typeof addEffectSchema>;

export function isEffectKind(value: string): value is EffectKind {
    return value in EFFECT_FACTORIES;
}

