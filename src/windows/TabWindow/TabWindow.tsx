import {useEffect, useRef, useState} from "react";
import {AlphaTabApi} from "@coderline/alphatab";
import {Box, Button, Stack, Typography, useTheme} from "@mui/material";
import "./TabWindow.css";

export function TabWindow() {
    const theme = useTheme();
    const tabContainerRef = useRef<HTMLDivElement | null>(null);
    const apiRef = useRef<AlphaTabApi | null>(null);
    const fileInputRef = useRef<HTMLInputElement | null>(null);
    const prevObjectUrlRef = useRef<string | null>(null);
    const [selectedFileName, setSelectedFileName] = useState<string | null>(null);
    const cursorBeatBackground = theme.palette.primary.main;

    useEffect(() => {

        const createApi = (fileUrl?: string) => {
            if (!tabContainerRef.current) return;

            if (apiRef.current) {
                try {
                    apiRef.current.destroy();
                } catch (e) {
                }
                apiRef.current = null;
            }

            if (fileUrl) {
                tabContainerRef.current.setAttribute('data-file', fileUrl);
            } else {
                tabContainerRef.current.removeAttribute('data-file');
            }

            const settings: ConstructorParameters<typeof AlphaTabApi>[1] = {
                core: {
                    fontDirectory: '/font/'
                },
                player: {
                    enablePlayer: true,
                    soundFont: '/soundfont/sonivox.sf2',

                }
            };
            apiRef.current = new AlphaTabApi(tabContainerRef.current, settings);

            // Set the scroll element after API is created (alphaTab creates the viewport in the DOM)
            const viewport = tabContainerRef.current?.querySelector('.at-viewport');
            if (viewport && apiRef.current) {
                (apiRef.current as any).settings.scrollElement = viewport;
            }

            // Hook into player position to render cursor line
            if (apiRef.current) {
                apiRef.current.playerPositionChanged.on(() => {
                    // alphaTab automatically renders the cursor at the current playback position
                });
            }
        };

        createApi();

        return () => {
            if (apiRef.current) {
                try {
                    apiRef.current.destroy();
                } catch (e) {
                }
                apiRef.current = null;
            }

            if (prevObjectUrlRef.current) {
                URL.revokeObjectURL(prevObjectUrlRef.current);
                prevObjectUrlRef.current = null;
            }
        };
    }, []);

    const handlePlayPause = (): void => {
        if (apiRef.current) {
            apiRef.current.playPause();
        }
    };

    const handleChooseFile = () => {
        fileInputRef.current?.click();
    };

    const handleFileChange = async (e: React.ChangeEvent<HTMLInputElement>) => {
        const file = e.target.files && e.target.files[0];
        if (!file || !tabContainerRef.current) return;

        setSelectedFileName(file.name);

        const objectUrl = URL.createObjectURL(file);

        if (prevObjectUrlRef.current) {
            URL.revokeObjectURL(prevObjectUrlRef.current);
        }
        prevObjectUrlRef.current = objectUrl;


        if (apiRef.current && 'open' in apiRef.current && typeof (apiRef.current as any).open === 'function') {
            try {
                const res = (apiRef.current as any).open(objectUrl);
                if (res && typeof res.then === 'function') {
                    await res;
                }
                return;
            } catch (err) {
            }
        }

        if (apiRef.current) {
            try {
                apiRef.current.destroy();
            } catch (e) {
            }
            apiRef.current = null;
        }

        tabContainerRef.current.setAttribute('data-file', objectUrl);
        const settings: ConstructorParameters<typeof AlphaTabApi>[1] = {
            core: {fontDirectory: '/font/'},
            player: {enablePlayer: true, soundFont: '/soundfont/sonivox.sf2'}
        };
        apiRef.current = new AlphaTabApi(tabContainerRef.current, settings);

        // Set the scroll element after API is created (alphaTab creates the viewport in the DOM)
        const viewport = tabContainerRef.current?.querySelector('.at-viewport');
        if (viewport && apiRef.current) {
            (apiRef.current as any).settings.scrollElement = viewport;
        }

        // Hook into player position to render cursor line
        if (apiRef.current) {
            apiRef.current.playerPositionChanged.on(() => {
                // alphaTab automatically renders the cursor at the current playback position
            });
        }
    };

    return (
        <Box sx={{p: 3}}>
            {selectedFileName == null ?
                <Stack spacing={2} sx={{mb: 3}}>
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
                            accept=".gp"
                            style={{display: 'none'}}
                            onChange={handleFileChange}
                        />

                        <Button onClick={handleChooseFile} variant="outlined">Choose .gp file</Button>
                    </Box>
                </Stack>
                :
                <>
                    <Button onClick={handlePlayPause} variant="contained" color="primary" sx={{width: 'fit-content', position: "absolute" }}>
                        Play / Pause
                    </Button>
                    {/* Container element for alphaTab */}
                </>
            }
            <div
                ref={tabContainerRef}
                className="alphatab-host"
                style={{"--cursor-beat-bg": cursorBeatBackground} as React.CSSProperties}
            />
        </Box>
    );
}
