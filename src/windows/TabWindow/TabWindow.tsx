import {useEffect, useRef, useState} from "react";
import {Box, Button, Stack, Typography} from "@mui/material";
import "./TabWindow.css";
import {AlphaTabPlayer} from "../../components/AlphaTabPlayer.tsx";

export function TabWindow() {
    const fileInputRef = useRef<HTMLInputElement | null>(null);
    const [fileUrl, setFileUrl] = useState<string | null>(null);

    // Automatically clean up object URLs to prevent memory leaks when fileUrl changes or unmounts
    useEffect(() => {
        return () => {
            if (fileUrl) {
                URL.revokeObjectURL(fileUrl);
            }
        };
    }, [fileUrl]);

    const handleChooseFile = () => {
        fileInputRef.current?.click();
    };

    const handleFileChange = (e: React.ChangeEvent<HTMLInputElement>) => {
        const file = e.target.files && e.target.files[0];
        if (!file) return;

        // Revoke previous URL if a user swaps files without explicitly closing
        if (fileUrl) {
            URL.revokeObjectURL(fileUrl);
        }

        setFileUrl(URL.createObjectURL(file));
    };

    const handleCloseTab = () => {
        setFileUrl(null);
    };

    const acceptedExtensions = ".gp,.gp3,.gp4,.gp5,.gpx,.xml,.cap,.alphaTex";

    return (
        <Box>
            {fileUrl == null ? (
                <Box sx={{ p: 3 }}>
                    <Stack spacing={2} sx={{ mb: 3 }}>
                        <Box sx={{
                            display: 'flex',
                            alignItems: 'center',
                            justifyContent: "center",
                            gap: 2,
                            flexDirection: "column",
                            height: "85vh"
                        }}>
                            <Typography variant={"h4"} color="textSecondary">
                                No tab loaded
                            </Typography>

                            <input
                                ref={fileInputRef}
                                type="file"
                                accept= {acceptedExtensions}
                                style={{ display: 'none' }}
                                onChange={handleFileChange}
                            />

                            <Button onClick={handleChooseFile} variant="outlined">
                                Choose .gp file
                            </Button>
                        </Box>
                    </Stack>
                </Box>
            ) : (
                <AlphaTabPlayer
                    fileUrl={fileUrl}
                    onClose={handleCloseTab}
                />
            )}
        </Box>
    );
}