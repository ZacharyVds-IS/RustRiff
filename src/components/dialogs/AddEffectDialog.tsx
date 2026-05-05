import {Button, Dialog, DialogActions, DialogContent, DialogTitle, Typography} from "@mui/material";

interface AddEffectDialogProps {
    open: boolean;
    onClose: () => void;
    onCreate: () => void;
}

export function AddEffectDialog({open, onClose, onCreate}: AddEffectDialogProps) {

    return (
        <Dialog
            open={open}
            onClose={onClose}
            fullWidth
            maxWidth="sm"
        >
            <DialogTitle>New Effect</DialogTitle>

            <DialogContent>
                <Typography>We gaan hiere is een effectje toevoegen eh.</Typography>
            </DialogContent>

            <DialogActions>
                <Button onClick={onClose}>Cancel</Button>
                <Button
                    variant="contained"
                    onClick={onCreate}
                >
                    Create
                </Button>
            </DialogActions>
        </Dialog>
    );
}