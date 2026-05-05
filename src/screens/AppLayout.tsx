import {AppBar, Box, Button, IconButton, Toolbar, Typography} from "@mui/material";
import {Outlet, useNavigate} from "react-router-dom";
import {ChannelSelector} from "../components/ChannelSelector.tsx";
import {useAmpStore} from "../state/AmpConfigStore.tsx";
import {useState} from "react";
import {AddChannelDialog} from "../components/dialogs/AddChannelDialog.tsx";
import DeleteIcon from '@mui/icons-material/Delete';
import {DeleteConfirmationDialog} from "../components/dialogs/DeleteConfirmationDialog.tsx";

export function AppLayout() {
    const navigate = useNavigate();
    const ampStore = useAmpStore();
    const channels = ampStore.channels;
    const currentChannelId = ampStore.current_channel;
    const currentChannel = ampStore.channels.find(c => c.id === currentChannelId) || {id: -1, name: "No Channel"};

    console.log("AppLayout - channels:", channels, "currentChannelName:", currentChannel.name);

    const channelOptions = channels.map((channel) => ({label: channel.name, value: channel.id}));

    const handleChannelChange = async (id: number) => {
        console.log("Changing channel to id:", id);
        await ampStore.setChannelById(id);
    };

    const [dialogOpen, setDialogOpen] = useState(false);
    const [confirmOpen, setConfirmOpen] = useState(false);


    const handleAddChannel = async (name: string) => {
        await ampStore.addChannel(name);
        setDialogOpen(false);
    };

    const handleDeleteChannel = async () => {
        ampStore.removeChannel(currentChannelId);
    }


    return (
        <Box sx={{display: 'flex', flexDirection: 'column', height: '100vh'}}>
            <AppBar
                position="static"
                sx={{
                    height: '50px',
                    justifyContent: 'center',
                    bgcolor: 'background.paper',
                    color: 'text.primary',
                    borderBottom: '1px solid',
                    borderColor: 'divider'
                }}
            >
                <Toolbar variant="dense" sx={{justifyContent: 'space-between'}}>
                    <Typography variant="h6" sx={{fontWeight: 'bold'}}>
                        Rust Riff
                    </Typography>
                    <Box sx={{display: 'flex', direction: "row", alignItems: 'center', gap: 2, width: "25%"}}>
                        {channels.length > 0 ? (
                            <>
                                <ChannelSelector
                                    channels={channelOptions}
                                    currentChannelId={currentChannelId >= 0 ? currentChannelId : 0}
                                    onChannelChange={handleChannelChange}
                                    onAdd={() => setDialogOpen(true)}
                                />
                                <AddChannelDialog open={dialogOpen} onClose={() => setDialogOpen(false)}
                                                  onCreate={handleAddChannel}/>
                                {currentChannelId != 0 &&
                                    <IconButton onClick={() => setConfirmOpen(true)}><DeleteIcon/></IconButton>}
                                <DeleteConfirmationDialog
                                    open={confirmOpen}
                                    onClose={() => setConfirmOpen(false)}
                                    onConfirm={handleDeleteChannel}
                                    title={`Delete channel "${currentChannel.name}"?`}
                                    description={"Are you sure you want to remove this channel? This action cannot be undone."}
                                />
                            </>

                        ) : (
                            <Typography variant="body2" sx={{color: 'text.secondary'}}>
                                No channels
                            </Typography>
                        )}
                        <Button color="inherit" onClick={() => navigate("/")}>Home</Button>
                        <Button color="inherit" onClick={() => navigate("/settings")}>Settings</Button>
                    </Box>
                </Toolbar>
            </AppBar>

            <Box sx={{flexGrow: 1, overflow: 'auto', bgcolor: 'background.default'}}>
                <Outlet/>
            </Box>
        </Box>
    );
}