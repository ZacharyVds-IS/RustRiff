import {useEffect, useRef} from "react";
import {AlphaTabApi} from "@coderline/alphatab";
import {Box, Button, Stack} from "@mui/material";

export function TabWindow() {
    const tabContainerRef = useRef<HTMLDivElement | null>(null);
    const apiRef = useRef<AlphaTabApi | null>(null);

    useEffect(() => {
        if (tabContainerRef.current) {
            // Automatically extracts the exact expected options type for your alphaTab version
            const settings: ConstructorParameters<typeof AlphaTabApi>[1] = {
                core: {
                    fontDirectory: '/font/'
                },
                player: {
                    enablePlayer: true,
                    soundFont: '/soundfont/sonivox.sf2'
                }
            };

            apiRef.current = new AlphaTabApi(tabContainerRef.current, settings);
        }

        return () => {
            if (apiRef.current) {
                apiRef.current.destroy();
                apiRef.current = null;
            }
        };
    }, []);

    const handlePlayPause = (): void => {
        if (apiRef.current) {
            apiRef.current.playPause();
        }
    };

    return (
        <Box sx={{p: 3}}>
            <Stack spacing={2} sx={{mb: 3}}>
                <Button onClick={handlePlayPause} variant="contained" color="primary" sx={{width: 'fit-content'}}>
                    Play / Pause
                </Button>
            </Stack>

            {/* Container element for alphaTab - Points to public folder */}
            <div
                ref={tabContainerRef}
                data-file="/sultans-of-swing.gp"
            />
        </Box>
    );
}


