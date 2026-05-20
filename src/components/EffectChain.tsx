import {Box, Button, IconButton, Stack, Typography} from "@mui/material";
import {EffectPedalPreview} from "./EffectPedalPreview.tsx";
import {CabinetPreview} from "./CabinetPreview.tsx";
import {EffectDto} from "../domain";
import {AddCircle, Delete, KeyboardArrowLeft, KeyboardArrowRight} from "@mui/icons-material";
import {ConfirmationDialog} from "./dialogs/ConfirmationDialog.tsx";
import {useState} from "react";
import {AddEffectDialog} from "./dialogs/AddEffectDialog.tsx";
import {useAmpStore} from "../state/AmpConfigStore.tsx";
import {AmpBox} from "./AmpBox.tsx";
import {DragDropContext, Draggable, Droppable, DropResult} from "@hello-pangea/dnd";

export interface EffectChainProps {
    effects: EffectDto[];
    selected: EffectDto | "amp";
    /** "amp" = amp head selected, EffectDto = that effect is selected */
    onSelectionChange: (selected: EffectDto | "amp", selectedIndex?: number) => void;
    onReorderOpen: (open: boolean) => void;
}


export function EffectChain({effects, selected, onSelectionChange, onReorderOpen}: EffectChainProps) {
    function isAmpSelected() {
        return selected === "amp";
    }

    const [removeDialogOpen, setRemoveDialogOpen] = useState(false);
    const [addDialogOpen, setAddDialogOpen] = useState(false);
    const [reorderOpen, setReorderOpen] = useState(false);
    const {startEditingChainOrder, cancelEditingChainOrder, applyChangesToChainOrder, moveEffect} = useAmpStore();

    const handleAdd = (newEffect: EffectDto) => {
        useAmpStore.getState().addEffect(newEffect);

        setAddDialogOpen(false);
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

    const handleMovePedal = (currentIndex: number, direction: "left" | "right") => {
        const newIndex = direction === "left" ? currentIndex - 1 : currentIndex + 1;
        if (newIndex < 0 || newIndex >= effects.length) return;

        moveEffect(currentIndex, newIndex);
    }

    const onDragEnd = (result: DropResult) => {
        if (!result.destination) return;

        const sourceIndex = result.source.index;
        const newIndex = result.destination.index;

        moveEffect(sourceIndex, newIndex);
    };

    function isEffectSelected(effect: EffectDto) {
        return selected !== "amp" && selected === effect;
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
                height: reorderOpen ? 600 : 300,
            }}
        >
            <Box sx={{display: 'flex', justifyContent: 'flex-end'}}>
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
            {/*Scrollable Wrapper*/}
            <Box
                sx={{
                    height: "80%",
                    width: '100%',
                    overflowX: 'auto',
                    position: 'relative',
                    pb: 2,
                    '&::-webkit-scrollbar': {height: '8px'},
                    '&::-webkit-scrollbar-thumb': {
                        bgcolor: 'rgba(0,0,0,0.1)',
                        borderRadius: '4px',
                    },
                    mt: reorderOpen ? 10 : 0.75,
                    pt: reorderOpen ? 17 : 8.25
                }}
            >
                <Box sx={{position: 'relative', width: 'max-content', minWidth: '100%'}}>
                    {/* The Horizontal Line */}
                    <Box
                        sx={{
                            position: 'absolute',
                            left: 0,
                            right: 0,
                            top: "30%",
                            transform: 'translateY(-50%)',
                            height: '6px',
                            bgcolor: 'secondary.main',
                            zIndex: 1,
                        }}
                    />
                    {/* The Chain Stack */}
                    <DragDropContext onDragEnd={onDragEnd}>
                        <Droppable droppableId="pedal-board" direction="horizontal">
                            {(provided) => (
                                <Stack
                                    {...provided.droppableProps}
                                    ref={provided.innerRef}
                                    direction="row"
                                    spacing={6}
                                    sx={{
                                        width: 'max-content',
                                        minWidth: '100%',
                                        position: 'relative',
                                        zIndex: 2,
                                        minHeight: 120,
                                        px: 2
                                    }}
                                >
                                    <AmpBox onSelectionChange={onSelectionChange} isAmpSelected={isAmpSelected}
                                            selectedBorder={selectedBorder}/>

                                    {effects.map((item, index) => (
                                        <Draggable
                                            key={`effect-${item.kind}-${item.data.id}`}
                                            draggableId={`effect-${item.kind}-${item.data.id}`}
                                            index={index}
                                            isDragDisabled={!reorderOpen}
                                        >
                                            {(provided, snapshot) => (
                                                <Box
                                                    onClick={() => onSelectionChange(item, index)}
                                                    ref={provided.innerRef}
                                                    {...provided.draggableProps}
                                                    {...provided.dragHandleProps}
                                                    sx={{
                                                        display: 'flex',
                                                        flexDirection: 'column',
                                                        alignItems: 'center',
                                                        position: 'relative',
                                                        '&:hover .remove-button': {
                                                            opacity: 1,
                                                            transform: 'scale(1)',
                                                        },
                                                        gap: 1,
                                                        ...provided.draggableProps.style,
                                                        opacity: snapshot.isDragging ? 0.8 : 1,
                                                        cursor: reorderOpen ? 'grab' : 'pointer'
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
                                                    <Box sx={{
                                                        display: 'flex',
                                                        flexDirection: "column",
                                                        alignItems: 'center',
                                                        height: 75,
                                                        width: 60
                                                    }}>
                                                        <Box sx={{display: 'flex', alignItems: 'center', height: 75}}>
                                                            <Box sx={{
                                                                borderRadius: 2,
                                                                transition: 'border 0.15s, box-shadow 0.15s',
                                                                ...(isEffectSelected(item) && selectedBorder),
                                                            }}>
                                                                {item.kind === "Cabinet"
                                                                    ? <CabinetPreview mainColor={item.data.color}
                                                                                      isActive={item.data.is_active}/>
                                                                    :
                                                                    <EffectPedalPreview mainColor={item.data.color}
                                                                                        isActive={item.data.is_active}/>
                                                                }
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
                                                        {reorderOpen && isEffectSelected(item) &&
                                                            <Box sx={{
                                                                display: "flex",
                                                                flexDirection: "row",
                                                                alignItems: "center"
                                                            }}>
                                                                <IconButton
                                                                    onClick={() => handleMovePedal(index, "left")}>
                                                                    <KeyboardArrowLeft/>
                                                                </IconButton>
                                                                <IconButton
                                                                    onClick={() => handleMovePedal(index, "right")}>
                                                                    <KeyboardArrowRight/>
                                                                </IconButton>
                                                            </Box>
                                                        }
                                                    </Box>
                                                </Box>
                                            )}
                                        </Draggable>
                                    ))}
                                    {provided.placeholder}

                                    {!reorderOpen &&
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
                                            <AddEffectDialog open={addDialogOpen}
                                                             onClose={() => setAddDialogOpen(false)}
                                                             onCreate={handleAdd}/>
                                        </Box>
                                    }
                                </Stack>
                            )}
                        </Droppable>
                    </DragDropContext>
                </Box>
            </Box>
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