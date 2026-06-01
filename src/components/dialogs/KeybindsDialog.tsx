import {
    Dialog,
    DialogContent,
    DialogTitle,
    Table,
    TableBody,
    TableCell,
    TableContainer,
    TableHead,
    TableRow,
    Typography,
} from "@mui/material";

interface KeybindsDialogProps {
    open: boolean;
    onClose: () => void;
}

export function KeybindsDialog({open, onClose}: KeybindsDialogProps) {
    const mappings = [
        {keys: "1", action: "Select amp"},
        {keys: "2-0", action: "Select effects 1-9"},
        {keys: "Space", action: "Toggle selected amp/effect on or off"},
        {keys: "Arrow Left", action: "Move selected effect left"},
        {keys: "Arrow Right", action: "Move selected effect right"},
    ];

    return (
        <Dialog open={open} onClose={onClose} fullWidth maxWidth="sm">
            <DialogTitle>Keyboard Shortcuts</DialogTitle>
            <DialogContent>
                <Typography variant="body2" sx={{mb: 1.5}}>
                    Use these shortcuts while on the main screen.
                </Typography>
                <TableContainer>
                    <Table size="small" aria-label="keyboard-shortcuts-table">
                        <TableHead>
                            <TableRow>
                                <TableCell sx={{fontWeight: 700, width: "30%"}}>Key(s)</TableCell>
                                <TableCell sx={{fontWeight: 700}}>Action</TableCell>
                            </TableRow>
                        </TableHead>
                        <TableBody>
                            {mappings.map((mapping) => (
                                <TableRow key={mapping.keys}>
                                    <TableCell>{mapping.keys}</TableCell>
                                    <TableCell>{mapping.action}</TableCell>
                                </TableRow>
                            ))}
                        </TableBody>
                    </Table>
                </TableContainer>
            </DialogContent>
        </Dialog>
    );
}


