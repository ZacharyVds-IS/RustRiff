import {useEffect, useRef} from "react";
import {AlphaTabApi, CoreSettings, PlayerSettings} from "@coderline/alphatab";
import {Box, Button, Stack, Typography} from "@mui/material";

export function TabWindow() {
    const tabContainerRef = useRef<HTMLDivElement | null>(null);

    const apiRef = useRef<AlphaTabApi | null>(null);

    useEffect(() => {
        if (tabContainerRef.current) {
            const settings: { player: PlayerSettings, core: CoreSettings } = {
                player: {
                    soundFont: 'https://cdn.jsdelivr.net/npm/@alphatab/alphatab@latest/dist/soundfont/sonivox.sf2'
                } as Partial<PlayerSettings> as PlayerSettings,
                core: {
                    fontDirectory: 'https://cdn.jsdelivr.net/npm/@alphatab/alphatab@latest/dist/font/'
                } as Partial<CoreSettings> as CoreSettings,
            };
            apiRef.current = new AlphaTabApi(tabContainerRef.current, settings);
        }

        // Cleanup: safely destroy the instance when the component unmounts
        return () => {
            if (apiRef.current) {
                apiRef.current.destroy();
                apiRef.current = null;
            }
        };
    }, []);

    // Example handler for a type-safe button interaction
    const handlePlayPause = (): void => {
        if (apiRef.current) {
            apiRef.current.playPause();
        }
    };

    return (
        <Box>
            <Stack>
                <Typography variant={"h2"}>
                    Test Tab
                </Typography>
                <Button onClick={handlePlayPause} variant={"contained"} color={"primary"}>
                    Play / Pause
                </Button>
            </Stack>

            {/* Container element for alphaTab */}
            <div
                ref={tabContainerRef}
                data-file="Dire Straits - Sultans of Swing.gp"
            />
        </Box>
    );
}


