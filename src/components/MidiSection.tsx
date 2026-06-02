import {useEffect, useState} from "react";
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
    Typography,
    useTheme
} from "@mui/material";
import {useNavigate} from "react-router-dom";
import RefreshIcon from '@mui/icons-material/Refresh';

import {connectMidiDevice, disconnectMidiDevice, getMidiInputs, MidiDeviceDto} from "../domain"; // Adjust this import path to match your file structure

export function MidiSection() {
    const theme = useTheme();
    const navigate = useNavigate();

    const [devices, setDevices] = useState<MidiDeviceDto[]>([]);
    const [connectedDeviceId, setConnectedDeviceId] = useState<string | null>(null);
    const [loading, setLoading] = useState<boolean>(false);

    const fetchMidiDevices = async () => {
        setLoading(true);
        try {
            const inputs = await getMidiInputs();
            setDevices(inputs);
            // Optional: If your backend tracking persistence allows it,
            // you might want to fetch or match the currently active port here.
        } catch (err) {
            console.error("Failed to fetch MIDI devices in settings panel:", err);
        } finally {
            setLoading(false);
        }
    };

    useEffect(() => {
        fetchMidiDevices();
    }, []);

    const handleConnect = async (id: string) => {
        try {
            await connectMidiDevice({id});
            setConnectedDeviceId(id);
        } catch (err) {
            console.error("Failed to connect hardware:", err);
        }
    };

    const handleDisconnect = async () => {
        try {
            await disconnectMidiDevice();
            setConnectedDeviceId(null);
        } catch (err) {
            console.error("Failed to disconnect hardware:", err);
        }
    };

    return (
        <Box sx={{mt: 2, pt: 2, borderTop: `1px solid ${theme.palette.divider}`}}>
            {/* Title Block with Quick Refresh */}
            <Box sx={{display: 'flex', justifyContent: 'space-between', alignItems: 'center', mb: 1.5}}>
                <Typography variant="subtitle2"
                            sx={{fontWeight: "bold", display: 'flex', alignItems: 'center', gap: 1}}>
                    MIDI Configuration
                </Typography>
                <Button
                    size="small"
                    startIcon={<RefreshIcon sx={{fontSize: 14}}/>}
                    onClick={fetchMidiDevices}
                    disabled={loading}
                    sx={{fontSize: '0.75rem', py: 0}}
                >
                    Scan
                </Button>
            </Box>

            {/* Contained & Scrollable Device Wrapper */}
            <Box
                sx={{
                    borderRadius: 1,
                    border: `1px solid ${theme.palette.divider}`,
                    bgcolor: 'background.paper',
                    maxHeight: 180, // Restricts height to enforce scrolling
                    overflowY: 'auto', // Turns on vertical scrolling when overflowing
                    mb: 2
                }}
            >
                {loading ? (
                    <Box sx={{display: 'flex', justifyContent: 'center', alignItems: 'center', p: 3, gap: 1.5}}>
                        <CircularProgress size={16}/>
                        <Typography variant="caption" color="text.secondary">Scanning buses...</Typography>
                    </Box>
                ) : devices.length === 0 ? (
                    <Box sx={{p: 2.5, textAlign: 'center'}}>
                        <Typography variant="caption" color="text.secondary">
                            No MIDI devices detected.
                        </Typography>
                    </Box>
                ) : (
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
                                                <Stack direction={"row"} spacing={1}>
                                                    <Chip
                                                        label="Connected"
                                                        color="success"
                                                        size="small"
                                                        sx={{fontSize: '0.7rem', height: 20}}
                                                    />
                                                    <Button
                                                        variant="outlined"
                                                        size="small"
                                                        onClick={handleDisconnect}
                                                        sx={{fontSize: '0.7rem', height: 20, py: 0}}
                                                    >
                                                        Disconnect
                                                    </Button>
                                                </Stack>
                                            ) : (
                                                <Button
                                                    variant="outlined"
                                                    size="small"
                                                    onClick={() => handleConnect(device.id)}
                                                    sx={{fontSize: '0.7rem', height: 20, py: 0}}
                                                >
                                                    Connect
                                                </Button>
                                            )
                                        }
                                    >
                                        <ListItemText
                                            primary={device.name}

                                        />
                                    </ListItem>
                                </Box>
                            );
                        })}
                    </List>
                )}
            </Box>

            <Button
                variant="contained"
                size="small"
                fullWidth
                onClick={() => navigate("/midi-mappings")}
            >
                Configure Advanced Mappings
            </Button>
        </Box>
    );
}