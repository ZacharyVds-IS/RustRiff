import {Button, Dialog, DialogActions, DialogContent, DialogContentText, DialogTitle} from "@mui/material";

interface DeleteConfirmationDialogProps {
    open: boolean;
    onClose: () => void;
    onConfirm: () => void;
    title?: string;
    description?: string;
    confirmLabel?: string;
}

export function DeleteConfirmationDialog({
                                             open,
                                             onClose,
                                             onConfirm,
                                             title = "Confirm Delete",
                                             description = "This action cannot be undone. Are you sure you want to proceed?",
                                             confirmLabel = "Delete"
                                         }: DeleteConfirmationDialogProps) {
    return (
        <Dialog
            open={open}
            onClose={onClose}
            aria-labelledby="delete-dialog-title"
        >
            <DialogTitle id="delete-dialog-title">
                {title}
            </DialogTitle>

            <DialogContent>
                <DialogContentText>
                    {description}
                </DialogContentText>
            </DialogContent>

            <DialogActions sx={{ px: 3, pb: 2 }}>
                <Button onClick={onClose} color="inherit">
                    Cancel
                </Button>

                <Button
                    color="error"
                    variant="contained"
                    autoFocus
                    onClick={() => {
                        onConfirm();
                        onClose();
                    }}
                >
                    {confirmLabel}
                </Button>
            </DialogActions>
        </Dialog>
    );
}