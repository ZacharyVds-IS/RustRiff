import {Box, IconButton, Stack, Tooltip} from "@mui/material";
import {EffectDto} from "../domain";
import {AddCircle, Keyboard} from "@mui/icons-material";
import {ConfirmationDialog} from "./dialogs/ConfirmationDialog.tsx";
import {useState} from "react";
import {AddEffectDialog} from "./dialogs/AddEffectDialog.tsx";
import {useAmpStore} from "../state/AmpConfigStore.tsx";
import {AmpBox} from "./AmpBox.tsx";
import {DragDropContext, Droppable} from "@hello-pangea/dnd";
import {DraggableEffectItem} from "./DraggableEffectItem.tsx";
import {useEffectMoves} from "../hooks/useEffectMoves.ts";

export interface EffectChainProps {
    effects: EffectDto[];
    selected: EffectDto | "amp";
    onSelectionChange: (selected: EffectDto | "amp", selectedIndex?: number) => void;
    onOpenKeybinds?: () => void;
}

export function EffectChain({effects, selected, onSelectionChange, onOpenKeybinds}: EffectChainProps) {
    const [removeDialogOpen, setRemoveDialogOpen] = useState(false);
    const [addDialogOpen, setAddDialogOpen] = useState(false);
    const {handleMovePedal, onDragEnd} = useEffectMoves(effects.length);

    const handleAdd = (newEffect: EffectDto) => {
        useAmpStore.getState().addEffect(newEffect);
        setAddDialogOpen(false);
    };

    const handleEffectRemove = () => {
        if (selected !== "amp") {
            useAmpStore.getState().removeEffect(selected.data.id);
        }
        onSelectionChange("amp");
        setRemoveDialogOpen(false);
    };

    function isAmpSelected() {
        return selected === "amp";
    }

    function isEffectSelected(effect: EffectDto) {
        return selected !== "amp" && selected.data.id === effect.data.id && selected.kind === effect.kind;
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
                        <IconButton size="large" color="primary" onClick={onOpenKeybinds} aria-label="Open keyboard shortcuts">
                            <Keyboard fontSize="medium"/>
                        </IconButton>
                    </Tooltip>
                )}
            </Box>
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
                <Box sx={{my: 4, position: 'relative', width: 'max-content', minWidth: '100%'}}>
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
                                        py: 1
                                    }}
                                >
                                    <AmpBox onSelectionChange={onSelectionChange} isAmpSelected={isAmpSelected}
                                            selectedBorder={selectedBorder}/>

                                    {effects.map((item, index) => (
                                        <DraggableEffectItem
                                            key={`effect-${item.kind}-${item.data.id}`}
                                            item={item}
                                            index={index}
                                            isSelected={isEffectSelected(item)}
                                            selectedBorder={selectedBorder}
                                            onSelect={onSelectionChange}
                                            onRemoveClick={() => setRemoveDialogOpen(true)}
                                            onMoveLeft={() => handleMovePedal(index, "left")}
                                            onMoveRight={() => handleMovePedal(index, "right")}
                                        />
                                    ))}
                                    {provided.placeholder}

                                    <Box sx={{
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

            <ConfirmationDialog
                open={removeDialogOpen}
                onClose={() => setRemoveDialogOpen(false)}
                onConfirm={handleEffectRemove}
                title={selected !== "amp" ? `Remove effect "${selected.data.name}"?` : "Remove effect?"}
                description={"Are you sure you want to remove this effect from the chain? This action cannot be undone."}
            />
        </Box>
    );
}
