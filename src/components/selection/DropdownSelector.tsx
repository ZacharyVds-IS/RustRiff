import {
    Divider,
    FormControl,
    InputLabel,
    ListItemIcon,
    ListItemText,
    MenuItem,
    Select,
    SelectChangeEvent,
    Typography
} from "@mui/material";
import AddIcon from '@mui/icons-material/Add';

interface DropdownProps {
    title?: string;
    label: string;
    options: { label: string; value: string | number }[];
    selectedValue: string | number;
    onSelectionChange: (value: string | number) => void;
    onAdd?: () => void;
    hasBorder?: boolean;
    hasLabel?: boolean;
}

export function DropdownSelector({title, label, options, selectedValue, onSelectionChange, onAdd, hasBorder = true, hasLabel = true}: DropdownProps) {
    const handleChange = (event: SelectChangeEvent<string | number>) => {
        const selectedValue = event.target.value;

        if (selectedValue === "__ADD_NEW__") {
            onAdd?.()
        } else {
            onSelectionChange(selectedValue);
        }
    };
    return (
        <>
            {title && (
                <Typography variant="h6" gutterBottom>
                    {title}
                </Typography>
            )}

            <FormControl fullWidth>
                {hasLabel && <InputLabel id="simple-select-label">{label}</InputLabel>}
                <Select
                    labelId="simple-select-label"
                    id="simple-select"
                    value={selectedValue}
                    label={label}
                    onChange={handleChange}
                    sx={{
                        '& .MuiOutlinedInput-notchedOutline': {
                            border: hasBorder ? '1px solid' : 'none',
                        },
                    }}

                >
                    {options.map((option) => (
                        <MenuItem key={option.value} value={option.value}>
                            {option.label}
                        </MenuItem>
                    ))}

                    {onAdd && (
                        [
                            <Divider key="divider"/>,
                            <MenuItem key="add-button" value="__ADD_NEW__">
                                <ListItemIcon>
                                    <AddIcon fontSize="small"/>
                                </ListItemIcon>
                                <ListItemText>Add New Channel</ListItemText>
                            </MenuItem>
                        ]
                    )}

                </Select>
            </FormControl>
        </>
    )
        ;
}