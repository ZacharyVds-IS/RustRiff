import {Button, Dialog, DialogActions, DialogContent, DialogTitle, TextField} from "@mui/material";
import {useEffect, useState} from "react";

interface AddChannelProps {
    open: boolean;
    onClose: () => void;
    onCreate: (channelName: string) => void;
}

export function AddChannelDialog({open, onClose, onCreate}: AddChannelProps) {

    const [channelName, setChannelName] = useState("");


    useEffect(() => {
        if (!open) setChannelName("");
    }, [open]);

    const handleCreate = () => {
        const trimmed = channelName.trim();
        if (!trimmed) return;

        onCreate(trimmed);
    };

    return (
        <Dialog
            open={open}
            onClose={onClose}
            fullWidth
            maxWidth="sm"
        >
            <DialogTitle>New Channel</DialogTitle>

            <DialogContent>

                <TextField
                    autoFocus
                    margin="dense"
                    label="Channel name"
                    fullWidth
                    value={channelName}
                    onChange={(e) => setChannelName(e.target.value)}
                    slotProps={{
                        htmlInput: {maxLength: 30}
                    }}
                    helperText={`${channelName.length}/30`}
                    onKeyDown={(e) => {
                        if (e.key === "Enter") handleCreate();
                    }}
                />

            </DialogContent>

            <DialogActions>
                <Button onClick={onClose}>Cancel</Button>
                <Button
                    variant="contained"
                    onClick={handleCreate}
                    disabled={!channelName.trim()}
                >
                    Create
                </Button>
            </DialogActions>
        </Dialog>
    );
}