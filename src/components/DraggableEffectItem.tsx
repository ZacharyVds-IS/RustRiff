import {Box, IconButton, Typography} from "@mui/material";
import {Delete, KeyboardArrowLeft, KeyboardArrowRight} from "@mui/icons-material";
import {CabinetPreview} from "./CabinetPreview.tsx";
import {WahPedalPreview} from "./WahPedalPreview.tsx";
import {EffectPedalPreview} from "./EffectPedalPreview.tsx";
import {EffectDto} from "../domain";
import {Draggable} from "@hello-pangea/dnd";

interface DraggableEffectItemProps {
    item: EffectDto;
    index: number;
    isSelected: boolean;
    selectedBorder: Record<string, unknown>;
    onSelect: (item: EffectDto, index: number) => void;
    onRemoveClick: () => void;
    onMoveLeft: () => void;
    onMoveRight: () => void;
}

export function DraggableEffectItem({
                                        item, index, isSelected, selectedBorder,
                                        onSelect, onRemoveClick, onMoveLeft, onMoveRight
                                    }: DraggableEffectItemProps) {
    return (
        <Draggable
            key={`effect-${item.kind}-${item.data.id}`}
            draggableId={`effect-${item.kind}-${item.data.id}`}
            index={index}
        >
            {(provided, snapshot) => (
                <Box
                    onClick={() => onSelect(item, index)}
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
                        onClick={onRemoveClick}
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
                                ...(isSelected && selectedBorder),
                            }}>
                                {item.kind === "Cabinet"
                                    ? <CabinetPreview mainColor={item.data.color}
                                                      isActive={item.data.is_active}/>
                                    : item.kind === "Wah"
                                        ? <WahPedalPreview mainColor={item.data.color}
                                                           isActive={item.data.is_active}
                                                           pedalPosition={item.data.pedal_position}/>
                                        : <EffectPedalPreview mainColor={item.data.color}
                                                             isActive={item.data.is_active}/>
                                }
                            </Box>
                        </Box>
                        <Typography
                            variant="caption"
                            sx={{
                                mt: 1,
                                color: isSelected ? 'primary.main' : 'text.primary',
                                fontWeight: isSelected ? 700 : 500,
                                fontSize: '0.75rem',
                            }}
                        >
                            {item.data.name}
                        </Typography>
                        {isSelected &&
                            <Box sx={{
                                display: "flex",
                                flexDirection: "row",
                                alignItems: "center",
                                mt: 0.5
                            }}>
                                <IconButton onClick={(e) => { e.stopPropagation(); onMoveLeft(); }}>
                                    <KeyboardArrowLeft/>
                                </IconButton>
                                <IconButton onClick={(e) => { e.stopPropagation(); onMoveRight(); }}>
                                    <KeyboardArrowRight/>
                                </IconButton>
                            </Box>
                        }
                    </Box>
                </Box>
            )}
        </Draggable>
    );
}
