import {Button, Dialog, DialogActions, DialogContent, DialogTitle} from "@mui/material";

interface DeleteChannelConfirmationDialogProps {
    confirmOpen: boolean;
    setConfirmOpen: (open: boolean) => void;
    handleDelete: () => void;
}

export function DeleteChannelConfirmationDialog({confirmOpen, setConfirmOpen, handleDelete}: DeleteChannelConfirmationDialogProps) {
    return (
        <Dialog
            open={confirmOpen}
            onClose={() => setConfirmOpen(false)}
        >
            <DialogTitle>Delete channel?</DialogTitle>

            <DialogContent>
                This action cannot be undone. Are you sure you want to delete this channel?
            </DialogContent>

            <DialogActions>
                <Button onClick={() => setConfirmOpen(false)}>
                    Cancel
                </Button>

                <Button
                    color="error"
                    variant="contained"
                    onClick={() => {
                        handleDelete();
                        setConfirmOpen(false);
                    }}
                >
                    Delete
                </Button>
            </DialogActions>
        </Dialog>

    )
}