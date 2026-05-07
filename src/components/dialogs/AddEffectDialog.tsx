import {Button, Dialog, DialogActions, DialogContent, DialogTitle} from "@mui/material";
import {type EffectDto, getAllIrProfiles, type IrProfileDto, removeIrProfile, uploadIrProfile} from "../../domain";
import {useEffect, useState} from "react";
import {
    CABINET_CUSTOM_IR_VALUE,
    DEFAULT_CABINET_IR_FILE,
    EFFECT_FACTORIES,
    type EffectKind,
    resolveDefaultCabinetIrFile,
} from "../../config/effects";
import {useForm} from "react-hook-form";
import {zodResolver} from "@hookform/resolvers/zod";
import {
    type AddEffectFormValues,
    addEffectSchema,
    DEFAULT_ADD_EFFECT_FORM_VALUES,
    isEffectKind,
} from "../../config/schemas/addEffectSchema";
import {AddEffectFormFields} from "../forms/AddEffectFormFields.tsx";

interface AddEffectDialogProps {
    open: boolean;
    onClose: () => void;
    onCreate: (effect: EffectDto) => void;
}

const MAX_IR_UPLOAD_BYTES = 8 * 1024 * 1024;

export function AddEffectDialog({open, onClose, onCreate}: AddEffectDialogProps) {
    const [cabinetIrProfiles, setCabinetIrProfiles] = useState<IrProfileDto[]>([]);
    const [cabinetIrActionError, setCabinetIrActionError] = useState("");

    const {
        control,
        register,
        handleSubmit,
        reset,
        setValue,
        watch,
        formState: {errors, isValid},
    } = useForm<AddEffectFormValues>({
        resolver: zodResolver(addEffectSchema),
        mode: "onChange",
        defaultValues: DEFAULT_ADD_EFFECT_FORM_VALUES,
    });

    const selectedCabinetIrChoice = watch("cabinetIrChoice");
    const selectedCabinetProfile = cabinetIrProfiles.find(
        (profile) => profile.file_name === selectedCabinetIrChoice,
    );
    const cabinetIrOptions = [
        ...cabinetIrProfiles.map((profile) => ({label: profile.label, value: profile.file_name})),
        {label: "Custom IR file", value: CABINET_CUSTOM_IR_VALUE},
    ];

    useEffect(() => {
        if (!open) {
            reset(DEFAULT_ADD_EFFECT_FORM_VALUES);
            setCabinetIrActionError("");
        }
    }, [open, reset]);

    useEffect(() => {
        if (!open) {
            return;
        }

        void refreshIrProfiles();
    }, [open]);

    const refreshIrProfiles = async () => {
        try {
            const profiles = await getAllIrProfiles();
            setCabinetIrProfiles(profiles);
            setCabinetIrActionError("");
        } catch (error) {
            console.error("Failed to load cabinet IR profiles:", error);
            setCabinetIrProfiles([]);
            setCabinetIrActionError(getUserFriendlyIrError(error, "load"));
        }
    };

    const handleDialogClose = () => {
        reset(DEFAULT_ADD_EFFECT_FORM_VALUES);
        setCabinetIrActionError("");
        onClose();
    };

    const handleRemoveSelectedCabinetIr = async () => {
        if (!selectedCabinetProfile) {
            return;
        }

        try {
            await removeIrProfile({fileName: selectedCabinetProfile.file_name});
            setValue("cabinetIrChoice", "");
            await refreshIrProfiles();
        } catch (error) {
            console.error("Failed to remove IR profile:", error);
            setCabinetIrActionError(getUserFriendlyIrError(error, "remove"));
        }
    };

    const handleCreate = async (values: AddEffectFormValues) => {
        if (!isEffectKind(values.selectedEffect)) {
            return;
        }

        const effectKind: EffectKind = values.selectedEffect;

        let selectedCabinetIrFile = DEFAULT_CABINET_IR_FILE;

        if (effectKind === "Cabinet") {
            if (values.cabinetIrChoice === CABINET_CUSTOM_IR_VALUE) {
                const pickedFile = values.customCabinetIrFile?.item(0);
                if (!pickedFile) {
                    setCabinetIrActionError("Please choose a custom .wav IR file");
                    return;
                }

                if (pickedFile.size > MAX_IR_UPLOAD_BYTES) {
                    setCabinetIrActionError(
                        `IR file is too large. Maximum allowed size is ${Math.floor(MAX_IR_UPLOAD_BYTES / (1024 * 1024))} MB.`,
                    );
                    return;
                }

                try {
                    const fileBytes = new Uint8Array(await pickedFile.arrayBuffer());
                    selectedCabinetIrFile = await uploadIrProfile({
                        fileName: pickedFile.name,
                        fileBytes: Array.from(fileBytes),
                    });
                    await refreshIrProfiles();
                } catch (error) {
                    console.error("Failed to upload IR profile:", error);
                    setCabinetIrActionError(getUserFriendlyIrError(error, "upload"));
                    return;
                }
            } else if (values.cabinetIrChoice) {
                selectedCabinetIrFile = values.cabinetIrChoice;
            } else {
                selectedCabinetIrFile = await resolveDefaultCabinetIrFile();
            }
        }

        const defaultData = EFFECT_FACTORIES[effectKind]({
            name: values.name.trim(),
            color: values.color,
            cabinetIrFilePath: selectedCabinetIrFile,
        });

        const fullDto: EffectDto = {
            kind: effectKind,
            data: defaultData
        } as EffectDto;

        onCreate(fullDto);
        handleDialogClose();
    };

    return (
        <Dialog
            open={open}
            onClose={handleDialogClose}
            fullWidth
            maxWidth="sm"
        >
            <DialogTitle>New Effect</DialogTitle>

            <DialogContent>
                <AddEffectFormFields
                    control={control}
                    register={register}
                    errors={errors}
                    cabinetIrOptions={cabinetIrOptions}
                    canRemoveSelectedCabinetIr={Boolean(
                        selectedCabinetProfile?.is_custom && !selectedCabinetProfile.is_in_use,
                    )}
                    selectedCabinetIrIsInUse={Boolean(selectedCabinetProfile?.is_in_use)}
                    onRemoveSelectedCabinetIr={handleRemoveSelectedCabinetIr}
                    cabinetIrActionError={cabinetIrActionError}
                />
            </DialogContent>

            <DialogActions>
                <Button onClick={handleDialogClose}>Cancel</Button>
                <Button
                    variant="contained"
                    disabled={!isValid}
                    onClick={handleSubmit(handleCreate)}
                >
                    Create
                </Button>
            </DialogActions>
        </Dialog>
    );
}

function getUserFriendlyIrError(error: unknown, operation: "load" | "upload" | "remove"): string {
    const rawMessage = extractRawErrorMessage(error);
    const lower = rawMessage.toLowerCase();

    if (lower.includes("only .wav")) {
        return "Only .wav IR files are supported.";
    }

    if (lower.includes("unsupported wav format") || lower.includes("unexpected fmt chunk size")) {
        return "This WAV encoding is not supported. Re-export as PCM 16/24-bit or Float32 WAV.";
    }

    if (lower.includes("no impulse detected") || lower.includes("first sample is silence")) {
        return "This file does not look like a valid impulse response (the start is effectively silent).";
    }

    if (lower.includes("already exists")) {
        return "An IR with this file name already exists. Rename the file or remove the existing one first.";
    }

    if (lower.includes("currently used by an effect chain")) {
        return "This IR is currently used in your effect chain, so it cannot be removed.";
    }

    if (lower.includes("default ir profiles cannot be removed")) {
        return "Default IR profiles cannot be removed.";
    }

    if (lower.includes("failed to lock")) {
        return "The app is busy processing audio state. Please try again.";
    }

    switch (operation) {
        case "load":
            return "Could not load IR profiles. Please try again.";
        case "upload":
            return "Could not upload this IR file. Please check the file and try again.";
        case "remove":
            return "Could not remove this IR profile right now. Please try again.";
    }
}

function extractRawErrorMessage(error: unknown): string {
    if (typeof error === "string") {
        return error;
    }

    if (error && typeof error === "object") {
        if ("message" in error && typeof (error as {message?: unknown}).message === "string") {
            return (error as {message: string}).message;
        }

        if ("error" in error && typeof (error as {error?: unknown}).error === "string") {
            return (error as {error: string}).error;
        }
    }

    return "Unknown IR error";
}

