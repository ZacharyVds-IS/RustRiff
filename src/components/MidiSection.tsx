import {Box, Button, Typography, useTheme} from "@mui/material";
import RefreshIcon from '@mui/icons-material/Refresh';
import {useNavigate} from "react-router-dom";
import {useMidiDevices} from "../hooks/useMidiDevices.ts";
import {MidiDeviceList} from "./MidiDeviceList.tsx";

export function MidiSection() {
    const theme = useTheme();
    const navigate = useNavigate();

    const {devices, connectedDeviceId, loading, handleConnect, handleDisconnect, refresh} = useMidiDevices();

    return (
        <Box sx={{mt: 2, pt: 2, borderTop: `1px solid ${theme.palette.divider}`}}>
            <Box sx={{display: 'flex', justifyContent: 'space-between', alignItems: 'center', mb: 1.5}}>
                <Typography variant="subtitle2"
                            sx={{fontWeight: "bold", display: 'flex', alignItems: 'center', gap: 1}}>
                    MIDI Configuration
                </Typography>
                <Button
                    size="small"
                    startIcon={<RefreshIcon sx={{fontSize: 14}}/>}
                    onClick={refresh}
                    disabled={loading}
                    sx={{fontSize: '0.75rem', py: 0}}
                >
                    Scan
                </Button>
            </Box>

            <Box
                sx={{
                    borderRadius: 1,
                    border: `1px solid ${theme.palette.divider}`,
                    bgcolor: 'background.paper',
                    maxHeight: 180,
                    overflowY: 'auto',
                    mb: 2
                }}
            >
                <MidiDeviceList
                    devices={devices}
                    connectedDeviceId={connectedDeviceId}
                    loading={loading}
                    onConnect={handleConnect}
                    onDisconnect={handleDisconnect}
                />
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
