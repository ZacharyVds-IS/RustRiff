import {useEffect, useRef, useState} from "react";
import {AlphaTabApi} from "@coderline/alphatab";
import {Box, Button, Stack} from "@mui/material";

export function TabWindow() {
    const tabContainerRef = useRef<HTMLDivElement | null>(null);
    const apiRef = useRef<AlphaTabApi | null>(null);
    const fileInputRef = useRef<HTMLInputElement | null>(null);
    const prevObjectUrlRef = useRef<string | null>(null);
    const [selectedFileName, setSelectedFileName] = useState<string | null>(null);

    useEffect(() => {
        // create api with an optional file URL. If fileUrl is provided it will be set as
        // the container's `data-file` (alphaTab will load it on init).
        const createApi = (fileUrl?: string) => {
            if (!tabContainerRef.current) return;

            // destroy any previous instance
            if (apiRef.current) {
                try {
                    apiRef.current.destroy();
                } catch (e) {
                    // ignore destroy errors
                }
                apiRef.current = null;
            }

            if (fileUrl) {
                tabContainerRef.current.setAttribute('data-file', fileUrl);
            } else {
                // no default sample: ensure no data-file is set
                tabContainerRef.current.removeAttribute('data-file');
            }

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
        };

        // init AlphaTab API without a default sample file
        createApi();

        return () => {
            if (apiRef.current) {
                try {
                    apiRef.current.destroy();
                } catch (e) {
                    // ignore
                }
                apiRef.current = null;
            }

            // revoke any object URLs we created
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

        // create object URL for the selected file
        const objectUrl = URL.createObjectURL(file);

        // revoke previous URL if any
        if (prevObjectUrlRef.current) {
            URL.revokeObjectURL(prevObjectUrlRef.current);
        }
        prevObjectUrlRef.current = objectUrl;

        // If the AlphaTab API exposes an `open` method we try to use it. Otherwise we
        // re-create the API instance with the container's data-file pointing to the object URL.
        if (apiRef.current && 'open' in apiRef.current && typeof (apiRef.current as any).open === 'function') {
            try {
                // `open` may return a promise in some versions
                const res = (apiRef.current as any).open(objectUrl);
                if (res && typeof res.then === 'function') {
                    await res;
                }
                return;
            } catch (err) {
                // fallback to re-creating the API below
            }
        }

        // fallback: destroy and recreate the API with the object URL set on the container
        if (apiRef.current) {
            try {
                apiRef.current.destroy();
            } catch (e) {
                // ignore
            }
            apiRef.current = null;
        }

        // set data-file and re-init
        tabContainerRef.current.setAttribute('data-file', objectUrl);
        const settings: ConstructorParameters<typeof AlphaTabApi>[1] = {
            core: { fontDirectory: '/font/' },
            player: { enablePlayer: true, soundFont: '/soundfont/sonivox.sf2' }
        };
        apiRef.current = new AlphaTabApi(tabContainerRef.current, settings);
    };

    return (
        <Box sx={{p: 3}}>
            <Stack spacing={2} sx={{mb: 3}}>
                <Box sx={{display: 'flex', alignItems: 'center', gap: 2}}>
                    <Button onClick={handlePlayPause} variant="contained" color="primary" sx={{width: 'fit-content'}}>
                        Play / Pause
                    </Button>

                    <input
                        ref={fileInputRef}
                        type="file"
                        accept=".gp"
                        style={{display: 'none'}}
                        onChange={handleFileChange}
                    />

                    <Button onClick={handleChooseFile} variant="outlined">Choose .gp file</Button>

                    {selectedFileName && <Box component="span">Loaded: {selectedFileName}</Box>}
                </Box>
            </Stack>

            {/* Container element for alphaTab */}
            <div
                ref={tabContainerRef}
            />
        </Box>
    );
}


