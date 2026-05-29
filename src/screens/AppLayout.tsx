import {AppBar, Box, Button, IconButton, Toolbar, Typography} from "@mui/material";
import {Link, Outlet, useNavigate} from "react-router-dom";
import {ChannelSelector} from "../components/ChannelSelector.tsx";
import {useAmpStore} from "../state/AmpConfigStore.tsx";
import {useState} from "react";
import {AddChannelDialog} from "../components/dialogs/AddChannelDialog.tsx";
import DeleteIcon from '@mui/icons-material/Delete';
import {ConfirmationDialog} from "../components/dialogs/ConfirmationDialog.tsx";
import {openAnalyzerWindow} from "../windows/AnalyzerWindow";

export function AppLayout() {
    const navigate = useNavigate();
    const ampStore = useAmpStore();
    const channels = ampStore.channels;
    const currentChannelId = ampStore.current_channel;
    const currentChannel = ampStore.channels.find(c => c.id === currentChannelId) || {id: "", name: "No Channel"};

    console.log("AppLayout - channels:", channels, "currentChannelName:", currentChannel.name);

    const channelOptions = channels.map((channel) => ({label: channel.name, value: channel.id}));

    const handleChannelChange = async (id: string) => {
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
                <Toolbar variant="dense" sx={{justifyContent: 'space-between', gap: 2, minWidth: 0}}>
                    <Typography variant="h6" sx={{fontWeight: 'bold', textDecoration: "none", color: "initial"}}
                                component={Link} to={"/"}>
                        Rust Riff
                    </Typography>
                    <Box
                        sx={{
                            display: 'flex',
                            alignItems: 'center',
                            justifyContent: 'flex-end',
                            gap: 1,
                            flex: 1,
                            minWidth: 0,
                            overflowX: 'auto',
                            whiteSpace: 'nowrap'
                        }}
                    >
                        {channels.length > 0 ? (
                            <>
                                <Box sx={{width: 200, minWidth: 200, flexShrink: 0}}>
                                    <ChannelSelector
                                        channels={channelOptions}
                                        currentChannelId={currentChannelId}
                                        onChannelChange={handleChannelChange}
                                        onAdd={() => setDialogOpen(true)}
                                    />
                                </Box>
                                <AddChannelDialog open={dialogOpen} onClose={() => setDialogOpen(false)}
                                                  onCreate={handleAddChannel}/>
                                {channels.length > 1 &&
                                    <IconButton onClick={() => setConfirmOpen(true)}><DeleteIcon/></IconButton>}
                                <ConfirmationDialog
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
                        <Button
                            color="inherit"
                            onClick={async () => {
                                try {
                                    await openAnalyzerWindow();
                                } catch (error) {
                                    console.error("Failed to open Analyzer window", error);
                                }
                            }}
                        >
                            Analyzer
                        </Button>
                        <Button color="inherit" onClick={() => navigate("/tuner")}>Tuner</Button>
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