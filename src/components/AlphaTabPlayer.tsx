import {useEffect, useRef} from "react";
import {AlphaTabApi} from "@coderline/alphatab";
import {Box, Button, useTheme} from "@mui/material";

interface AlphaTabPlayerProps {
    fileUrl: string;
    onClose: () => void;
}

export function AlphaTabPlayer({ fileUrl, onClose }: AlphaTabPlayerProps) {
    const theme = useTheme();
    const tabContainerRef = useRef<HTMLDivElement | null>(null);
    const apiRef = useRef<AlphaTabApi | null>(null);
    const cursorBeatBackground = theme.palette.primary.main;

    useEffect(() => {
        if (!tabContainerRef.current) return;

        // Set the file attribute on the container
        tabContainerRef.current.setAttribute('data-file', fileUrl);

        const settings: ConstructorParameters<typeof AlphaTabApi>[1] = {
            core: {
                fontDirectory: '/font/'
            },
            player: {
                enablePlayer: true,
            }
        };

        apiRef.current = new AlphaTabApi(tabContainerRef.current, settings);

        // Set the scroll element after API is created (alphaTab creates the viewport in the DOM)
        const viewport = tabContainerRef.current?.querySelector('.at-viewport');
        if (viewport && apiRef.current) {
            (apiRef.current as any).settings.scrollElement = viewport;
        }

        // Hook into player position to render cursor line
        apiRef.current.playerPositionChanged.on(() => {
            // alphaTab automatically renders the cursor at the current playback position
        });

        // Cleanup AlphaTab instance when component unmounts or file changes
        return () => {
            if (apiRef.current) {
                try {
                    apiRef.current.destroy();
                } catch (e) {
                    // instances already destroyed or empty
                }
                apiRef.current = null;
            }
        };
    }, [fileUrl]);

    const handlePlayPause = (): void => {
        if (apiRef.current) {
            apiRef.current.playPause();
        }
    };

    return (
        <Box sx={{ width: '100%', display: 'flex', flexDirection: 'column' }}>
            {/* Container element for alphaTab */}
            <Box sx={{ p: 3 }}>
                <div
                    ref={tabContainerRef}
                    className="alphatab-host"
                    style={{"--cursor-beat-bg": cursorBeatBackground, zIndex: 10} as React.CSSProperties}
                />
            </Box>
            {/* Sticky Control Bar */}
            <Box sx={{
                position: 'sticky',
                bottom: 0,
                zIndex: 1100,
                bgcolor: 'background.paper',
                p: 2,
                boxShadow: '0px -2px 4px rgba(0,0,0,0.05)',
                borderTop: '1px solid',
                borderColor: 'divider',
                display: 'flex',
                alignItems: 'center',
                gap: 2
            }}>
                <Button onClick={handlePlayPause} variant="contained" color="primary">
                    Play / Pause
                </Button>

                {/* Close button to return back to the upload screen */}
                <Button onClick={onClose} variant="outlined" color="error" sx={{ ml: 'auto' }}>
                    Close Tab
                </Button>
            </Box>
        </Box>
    );
}