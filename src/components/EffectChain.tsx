import {Box, IconButton, Stack, Tooltip, Typography} from "@mui/material";
import {EffectPedalPreview} from "./EffectPedalPreview.tsx";
import {CabinetPreview} from "./CabinetPreview.tsx";
import {EffectDto} from "../domain";
import {AddCircle, Delete, Keyboard, KeyboardArrowLeft, KeyboardArrowRight} from "@mui/icons-material";
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
    onOpenKeybinds?: () => void;
}

export function EffectChain({effects, selected, onSelectionChange, onOpenKeybinds}: EffectChainProps) {
    function isAmpSelected() {
        return selected === "amp";
    }

    const [removeDialogOpen, setRemoveDialogOpen] = useState(false);
    const [addDialogOpen, setAddDialogOpen] = useState(false);
    const {applyChangesToChainOrder, moveEffect} = useAmpStore();

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

    const handleMovePedal = async (currentIndex: number, direction: "left" | "right") => {
        const newIndex = direction === "left" ? currentIndex - 1 : currentIndex + 1;
        if (newIndex < 0 || newIndex >= effects.length) return;

        // Move the effect and instantly commit changes to the backend/store
        moveEffect(currentIndex, newIndex);
        await applyChangesToChainOrder();
    }

    const onDragEnd = async (result: DropResult) => {
        if (!result.destination) return;

        const sourceIndex = result.source.index;
        const newIndex = result.destination.index;

        if (sourceIndex === newIndex) return;

        // Move the effect and instantly commit changes to the backend/store
        moveEffect(sourceIndex, newIndex);
        await applyChangesToChainOrder();
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
                height: 300,
            }}
        >
            <Box sx={{position: "absolute", top: 8, right: 8, zIndex: 3}}>
                {onOpenKeybinds && (
                <Tooltip title="Show keyboard shortcuts">
                    <IconButton
                        size="small"
                        color="primary"
                        onClick={onOpenKeybinds}
                        aria-label="Open keyboard shortcuts"
                    >
                        <Keyboard fontSize="small"/>
                    </IconButton>
                </Tooltip>
                )}
            </Box>
            {/*Scrollable Wrapper*/}
            <Box
                sx={{
                    height: "90%",
                    width: '100%',
                    overflowX: 'auto',
                    position: 'relative',
                    pb: 2,
                    '&::-webkit-scrollbar': {height: '8px'},
                    '&::-webkit-scrollbar-thumb': {
                        bgcolor: 'rgba(0,0,0,0.1)',
                        borderRadius: '4px',
                    },
                    mt: 2,
                    pt: 4
                }}
            >
                <Box sx={{my:4 ,position: 'relative', width: 'max-content', minWidth: '100%'}}>
                    {/* The Horizontal Line */}
                    <Box
                        sx={{
                            position: 'absolute',
                            left: 0,
                            right: 0,
                            top: "35%",
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
                                        px: 2,
                                        py:1
                                    }}
                                >
                                    <AmpBox onSelectionChange={onSelectionChange} isAmpSelected={isAmpSelected}
                                            selectedBorder={selectedBorder}/>

                                    {effects.map((item, index) => (
                                        <Draggable
                                            key={`effect-${item.kind}-${item.data.id}`}
                                            draggableId={`effect-${item.kind}-${item.data.id}`}
                                            index={index}
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
                                                        cursor: 'grab'
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
                                                        {isEffectSelected(item) &&
                                                            <Box sx={{
                                                                display: "flex",
                                                                flexDirection: "row",
                                                                alignItems: "center",
                                                                mt: 0.5
                                                            }}>
                                                                <IconButton
                                                                    onClick={(e) => {
                                                                        e.stopPropagation(); // Stop selection trigger
                                                                        handleMovePedal(index, "left");
                                                                    }}>
                                                                    <KeyboardArrowLeft/>
                                                                </IconButton>
                                                                <IconButton
                                                                    onClick={(e) => {
                                                                        e.stopPropagation(); // Stop selection trigger
                                                                        handleMovePedal(index, "right");
                                                                    }}>
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
                                </Stack>
                            )}
                        </Droppable>
                    </DragDropContext>
                </Box>
            </Box>
        </Box>
    );
}