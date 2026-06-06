import {
    Box,
    Button,
    Chip,
    CircularProgress,
    Divider,
    List,
    ListItem,
    ListItemText,
    Stack,
    Typography
} from "@mui/material";
import {MidiDeviceDto} from "../domain";

interface MidiDeviceListProps {
    devices: MidiDeviceDto[];
    connectedDeviceId: string | null;
    loading: boolean;
    onConnect: (id: string) => void;
    onDisconnect: () => void;
}

export function MidiDeviceList({devices, connectedDeviceId, loading, onConnect, onDisconnect}: MidiDeviceListProps) {
    if (loading) {
        return (
            <Box sx={{display: 'flex', justifyContent: 'center', alignItems: 'center', p: 3, gap: 1.5}}>
                <CircularProgress size={16}/>
                <Typography variant="caption" color="text.secondary">Scanning buses...</Typography>
            </Box>
        );
    }

    if (devices.length === 0) {
        return (
            <Box sx={{p: 2.5, textAlign: 'center'}}>
                <Typography variant="caption" color="text.secondary">
                    No MIDI devices detected.
                </Typography>
            </Box>
        );
    }

    return (
        <List disablePadding dense>
            {devices.map((device, index) => {
                const isConnected = device.id === connectedDeviceId;
                return (
                    <Box key={device.id}>
                        {index > 0 && <Divider/>}
                        <ListItem
                            sx={{
                                py: 0.75,
                                px: 2,
                                bgcolor: isConnected ? 'action.selected' : 'transparent',
                                '&:hover': {bgcolor: 'action.hover'}
                            }}
                            secondaryAction={
                                isConnected ? (
                                    <Stack direction="row" spacing={1}>
                                        <Chip label="Connected" color="success" size="small"
                                              sx={{fontSize: '0.7rem', height: 20}}/>
                                        <Button variant="outlined" size="small" onClick={onDisconnect}
                                                sx={{fontSize: '0.7rem', height: 20, py: 0}}>
                                            Disconnect
                                        </Button>
                                    </Stack>
                                ) : (
                                    <Button variant="outlined" size="small"
                                            onClick={() => onConnect(device.id)}
                                            sx={{fontSize: '0.7rem', height: 20, py: 0}}>
                                        Connect
                                    </Button>
                                )
                            }
                        >
                            <ListItemText primary={device.name}/>
                        </ListItem>
                    </Box>
                );
            })}
        </List>
    );
}
