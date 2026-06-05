import {Button, Dialog, DialogActions, DialogContent, DialogTitle} from "@mui/material";
import {type EffectDto, uploadIrProfile} from "../../domain";
import {useEffect} from "react";
import {
    CABINET_CUSTOM_IR_VALUE,
    DEFAULT_CABINET_IR_FILE,
    EFFECT_FACTORIES,
    EFFECT_SHORT_NAMES,
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
import {useIrProfiles} from "../../hooks/useIrProfiles.ts";

interface AddEffectDialogProps {
    open: boolean;
    onClose: () => void;
    onCreate: (effect: EffectDto) => void;
}

const MAX_IR_UPLOAD_BYTES = 8 * 1024 * 1024;

export function AddEffectDialog({open, onClose, onCreate}: AddEffectDialogProps) {
    const {
        cabinetIrProfiles,
        cabinetIrActionError,
        setCabinetIrActionError,
        refreshIrProfiles,
        handleRemoveSelectedCabinetIr,
    } = useIrProfiles();

    const {
        control,
        register,
        handleSubmit,
        reset,
        setValue,
        watch,
        trigger,
        formState: {errors, isValid},
    } = useForm<AddEffectFormValues>({
        resolver: zodResolver(addEffectSchema),
        mode: "onChange",
        defaultValues: DEFAULT_ADD_EFFECT_FORM_VALUES,
    });

    const selectedCabinetIrChoice = watch("cabinetIrChoice");
    const selectedEffect = watch("selectedEffect");
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
        } else {
            void trigger("color");
        }
    }, [open, reset, trigger, setCabinetIrActionError]);

    useEffect(() => {
        if (!open) return;
        void refreshIrProfiles();
    }, [open, refreshIrProfiles]);

    useEffect(() => {
        if (selectedEffect && isEffectKind(selectedEffect)) {
            const defaultName = EFFECT_SHORT_NAMES[selectedEffect];
            setValue("name", defaultName, {shouldValidate: true});
        }
    }, [selectedEffect, setValue]);

    const handleDialogClose = () => {
        reset(DEFAULT_ADD_EFFECT_FORM_VALUES);
        setCabinetIrActionError("");
        onClose();
    };

    const handleCreate = async (values: AddEffectFormValues) => {
        if (!isEffectKind(values.selectedEffect)) return;

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
                    setCabinetIrActionError(`Could not upload this IR file. Please check the file and try again.`);
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
        <Dialog open={open} onClose={handleDialogClose} fullWidth maxWidth="sm">
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
                    onRemoveSelectedCabinetIr={() => handleRemoveSelectedCabinetIr(selectedCabinetProfile)}
                    cabinetIrActionError={cabinetIrActionError}
                />
            </DialogContent>
            <DialogActions>
                <Button onClick={handleDialogClose}>Cancel</Button>
                <Button variant="contained" disabled={!isValid} onClick={handleSubmit(handleCreate)}>
                    Create
                </Button>
            </DialogActions>
        </Dialog>
    );
}
