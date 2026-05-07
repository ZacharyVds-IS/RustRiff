import {Button, Stack, TextField, Typography} from "@mui/material";
import {DropdownSelector} from "../selection/DropdownSelector.tsx";
import {CABINET_CUSTOM_IR_VALUE, EFFECT_METADATA} from "../../config/effects";
import {type Control, Controller, type FieldErrors, type UseFormRegister, useWatch} from "react-hook-form";
import {type AddEffectFormValues} from "../../config/schemas/addEffectSchema";

const EFFECT_OPTIONS = Object.entries(EFFECT_METADATA).map(([kind, meta]) => ({
    label: meta.label,
    value: kind,
}));

interface AddEffectFormFieldsProps {
    control: Control<AddEffectFormValues>;
    register: UseFormRegister<AddEffectFormValues>;
    errors: FieldErrors<AddEffectFormValues>;
    cabinetIrOptions: { label: string; value: string }[];
    canRemoveSelectedCabinetIr: boolean;
    selectedCabinetIrIsInUse: boolean;
    onRemoveSelectedCabinetIr: () => void;
    cabinetIrActionError?: string;
}

export function AddEffectFormFields({
    control,
    register,
    errors,
    cabinetIrOptions,
    canRemoveSelectedCabinetIr,
    selectedCabinetIrIsInUse,
    onRemoveSelectedCabinetIr,
    cabinetIrActionError,
}: AddEffectFormFieldsProps) {
    const selectedEffect = useWatch({control, name: "selectedEffect"});
    const selectedCabinetIr = useWatch({control, name: "cabinetIrChoice"});
    const isCabinetSelected = selectedEffect === "Cabinet";
    const isCustomCabinetIr = selectedCabinetIr === CABINET_CUSTOM_IR_VALUE;

    return (
        <Stack direction="column" spacing={2} sx={{paddingY: 2}}>
            <Controller
                name="selectedEffect"
                control={control}
                render={({field}) => (
                    <>
                        <DropdownSelector
                            label={"Effect Type"}
                            options={EFFECT_OPTIONS}
                            selectedValue={field.value}
                            onSelectionChange={(value) => field.onChange(String(value))}
                        />
                        {errors.selectedEffect && (
                            <Typography variant="caption" color="error" sx={{mt: -1}}>
                                {errors.selectedEffect.message}
                            </Typography>
                        )}
                    </>
                )}
            />
            <Stack direction="row" spacing={2}>
                <TextField
                    label="Name"
                    {...register("name")}
                    sx={{width: 450}}
                    slotProps={{htmlInput: {maxLength: 15}}}
                    error={Boolean(errors.name)}
                    helperText={errors.name?.message}
                />
                <TextField
                    type="color"
                    label="Color"
                    {...register("color")}
                    sx={{width: 100}}
                    error={Boolean(errors.color)}
                    helperText={errors.color?.message}
                />
            </Stack>

            {isCabinetSelected && (
                <>
                    <Controller
                        name="cabinetIrChoice"
                        control={control}
                        render={({field}) => (
                            <>
                                <DropdownSelector
                                    label={"Cabinet IR"}
                                    options={cabinetIrOptions}
                                    selectedValue={field.value ?? ""}
                                    onSelectionChange={(value) => field.onChange(String(value))}
                                />
                                {errors.cabinetIrChoice && (
                                    <Typography variant="caption" color="error" sx={{mt: -1}}>
                                        {errors.cabinetIrChoice.message}
                                    </Typography>
                                )}
                            </>
                        )}
                    />

                    {isCustomCabinetIr && (
                        <TextField
                            type="file"
                            label="Custom IR File"
                            {...register("customCabinetIrFile")}
                            slotProps={{
                                inputLabel: {shrink: true},
                                htmlInput: {accept: ".wav"},
                            }}
                            error={Boolean(errors.customCabinetIrFile)}
                            helperText={errors.customCabinetIrFile?.message?.toString()}
                        />
                    )}

                    {!isCustomCabinetIr && selectedCabinetIr && (
                        <>
                            <Button
                                variant="outlined"
                                color="error"
                                disabled={!canRemoveSelectedCabinetIr}
                                onClick={onRemoveSelectedCabinetIr}
                            >
                                Remove selected IR profile
                            </Button>
                            {selectedCabinetIrIsInUse && (
                                <Typography variant="caption" color="text.secondary">
                                    This IR is currently used in an effect chain and cannot be removed.
                                </Typography>
                            )}
                        </>
                    )}

                    {cabinetIrActionError && (
                        <Typography variant="caption" color="error">
                            {cabinetIrActionError}
                        </Typography>
                    )}
                </>
            )}
        </Stack>
    );
}

