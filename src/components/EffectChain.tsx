import {Box, Button, IconButton, Stack, Typography} from "@mui/material";
import {EffectPedalPreview} from "./EffectPedalPreview.tsx";
import {EffectDto} from "../domain";
import {AddCircle, Delete, KeyboardArrowLeft, KeyboardArrowRight} from "@mui/icons-material";
import {ConfirmationDialog} from "./dialogs/ConfirmationDialog.tsx";
import {useState} from "react";
import {AddEffectDialog} from "./dialogs/AddEffectDialog.tsx";
import {useAmpStore} from "../state/AmpConfigStore.tsx";
import {AmpBox} from "./AmpBox.tsx";

export interface EffectChainProps {
    effects: EffectDto[];
    selected: EffectDto | "amp";
    /** "amp" = amp head selected, EffectDto = that effect is selected */
    onSelectionChange: (selected: EffectDto | "amp") => void;
    onReorderOpen: (open: boolean) => void;
}


export function EffectChain({effects, selected, onSelectionChange, onReorderOpen}: EffectChainProps) {
    function isAmpSelected() {
        return selected === "amp";
    }

    let [removeDialogOpen, setRemoveDialogOpen] = useState(false);
    let [addDialogOpen, setAddDialogOpen] = useState(false);
    let [reorderOpen, setReorderOpen] = useState(false);
    const { startEditingChainOrder, cancelEditingChainOrder, applyChangesToChainOrder, moveEffect } = useAmpStore();

    const handleAdd = (newEffect: EffectDto) => {
        useAmpStore.getState().addEffect(newEffect);

        setAddDialogOpen(false);
        console.log("You tried to add an effect it isn't wired yet")
    }

    const handleEffectRemove = () => {
        if (selected != "amp") {
            useAmpStore.getState().removeEffect(selected.data.id);
        }
        onSelectionChange("amp");
        setRemoveDialogOpen(false);
    }

    const handleToggleEffectReorder = () => {
        if (!reorderOpen) {
            startEditingChainOrder();
        } else {
            cancelEditingChainOrder();
        }
        setReorderOpen(!reorderOpen);
        onReorderOpen(!reorderOpen);
    }

    const handleApply = async () => {
        await applyChangesToChainOrder();
        setReorderOpen(false);
        onReorderOpen(false);
    };

    const handleMovePedal = (effectId: number, direction: "left" | "right") => {
        const currentIndex = effects.findIndex(e => e.data.id === effectId);

        const newIndex = direction === "left" ? currentIndex - 1 : currentIndex + 1;
        if (newIndex < 0 || newIndex >= effects.length) return;

        console.log(`It looks like you're trying to move effect ${effectId} from position ${currentIndex} to ${newIndex}`);

        moveEffect(effectId, newIndex);
    }

    function isEffectSelected(effect: EffectDto) {
        return selected !== "amp" && selected.data.id === effect.data.id;
    }

    const selectedBorder = {
        border: '2px solid',
        borderColor: 'primary.main',
        boxShadow: '0 0 0 3px rgba(25,118,210,0.15)',
    };

    return (
        <Box
            component="section"
            sx={{
                width: '100%',
                bgcolor: 'background.paper',
                borderRadius: 4,
                p: 2,
                position: 'relative',
                height: reorderOpen ? 600 : "auto",
            }}
        >
            <Box sx={{display: 'flex', justifyContent: 'flex-end', mb: reorderOpen ? 23.75 : 0.75}}>
                {!reorderOpen &&
                    <Button
                        sx={{
                            bgcolor: 'background.paper',
                            color: 'primary.main',
                            borderRadius: 50,
                            textTransform: 'none',
                            fontSize: '0.875rem',
                            fontWeight: 500,
                            p: 1.2,
                            px: 3,
                            border: '1px solid',
                            borderColor: 'divider',
                            '&:hover': {
                                bgcolor: '#fdfdfd',
                                cursor: 'pointer'
                            }
                        }}
                        onClick={handleToggleEffectReorder}
                    >
                        Edit Order
                    </Button>
                }
            </Box>

            {/* The Horizontal Line */}
            <Box
                sx={{
                    position: 'absolute',
                    left: 0,
                    right: 0,
                    top: reorderOpen ? '40%' : "50%",
                    transform: 'translateY(-50%)',
                    height: '6px',
                    bgcolor: 'secondary.main',
                    zIndex: 1
                }}
            />
            {/* The Chain Stack */}
            <Stack
                direction="row"
                spacing={6}
                sx={{width: '100%', position: 'relative', zIndex: 2, minHeight: 120}}
            >
                <AmpBox onSelectionChange={onSelectionChange} isAmpSelected={isAmpSelected}
                        selectedBorder={selectedBorder}/>

                {effects.map((item) => (
                    <Box
                        key={"effect-" + item.data.id}
                        onClick={() => onSelectionChange(item)}
                        sx={{
                            display: 'flex',
                            flexDirection: 'column',
                            alignItems: 'center',
                            position: 'relative',
                            '&:hover .remove-button': {
                                opacity: 1,
                                transform: 'scale(1)',
                            },
                            gap: 1
                        }}
                    >
                        <IconButton
                            className="remove-button"
                            size="small"
                            onClick={() => setRemoveDialogOpen(true)}
                            sx={{
                                position: 'absolute',
                                top: -15,
                                right: -10,
                                zIndex: 10,
                                opacity: 0,
                                transform: 'scale(0.8)',
                                transition: 'all 0.2s ease-in-out',
                                bgcolor: 'error.main',
                                color: 'white',
                                '&:hover': {bgcolor: 'error.dark'},
                                width: 25,
                                height: 25
                            }}
                        >
                            <Delete/>
                        </IconButton>
                        <ConfirmationDialog
                            open={removeDialogOpen}
                            onClose={() => setRemoveDialogOpen(false)}
                            onConfirm={handleEffectRemove}
                            title={`Remove effect "${item.data.name}"?`}
                            description={"Are you sure you want to remove this effect from the chain? This action cannot be undone."}
                        />
                        <Box sx={{display: 'flex',flexDirection:"column" , alignItems: 'center', height: 75, width:60}}>
                            <Box sx={{display: 'flex', alignItems: 'center', height: 75}}>
                                <Box sx={{
                                    borderRadius: 2,
                                    transition: 'border 0.15s, box-shadow 0.15s',
                                    ...(isEffectSelected(item) && selectedBorder),
                                }}>
                                    <EffectPedalPreview mainColor={item.data.color} isActive={item.data.is_active}/>
                                </Box>
                            </Box>
                            <Typography
                                variant="caption"
                                sx={{
                                    mt: 1,
                                    color: isEffectSelected(item) ? 'primary.main' : 'text.primary',
                                    fontWeight: isEffectSelected(item) ? 700 : 500,
                                    fontSize: '0.75rem',
                                }}
                            >
                                {item.data.name}
                            </Typography>
                            { reorderOpen && isEffectSelected(item)  &&
                            <Box sx={{display:"flex",flexDirection:"row" , alignItems:"center"}}>
                                <IconButton onClick={() => handleMovePedal(item.data.id, "left")}>
                                    <KeyboardArrowLeft/>
                                </IconButton>
                                <IconButton onClick={() => handleMovePedal(item.data.id, "right")}>
                                    <KeyboardArrowRight/>
                                </IconButton>
                            </Box>
                            }
                        </Box>
                    </Box>
                ))}
                <Box key={"add-effect-wrapper"} sx={{
                    display: 'flex',
                    flexDirection: 'column',
                    alignItems: 'center',
                    justifyContent: 'center',
                    height: 70
                }}>
                    <IconButton onClick={() => setAddDialogOpen(true)} sx={{
                        p: 0,
                        bgcolor: 'white',
                        '&:hover': {bgcolor: 'white', transform: 'scale(1.2)'},
                        overflow: 'hidden',
                        borderRadius: '50%',
                        display: 'flex',
                        alignItems: 'center',
                        justifyContent: 'center'
                    }}>
                        <AddCircle fontSize="large" color="primary"/>
                    </IconButton>
                    <AddEffectDialog open={addDialogOpen} onClose={() => setAddDialogOpen(false)} onCreate={handleAdd}/>
                </Box>
            </Stack>
            {reorderOpen &&
                <Stack direction={"row"} sx={{position: "absolute", bottom: 16, right: 16, zIndex: 3, gap: 3}}>
                    <Button onClick={handleToggleEffectReorder} variant="contained"
                            sx={{bgcolor: "secondary.main"}}>Cancel</Button>
                    <Button variant="contained" onClick={handleApply}>Apply Changes</Button>
                </Stack>
            }
        </Box>
    );
}