import {Box} from "@mui/material";
import {EffectChain} from "../components/EffectChain.tsx";
import {DefaultAmpControls} from "../components/DefaultAmpControls.tsx";

export function MainScreen() {
    return (
        <Box
            sx={{
                p: 4,
                display: "flex",
                flexDirection: "column",
                alignItems: "center", // Centering logic moved here
                justifyContent: "start",
                minHeight: "100vh",
                gap: 4
            }}
        >
            <EffectChain />
            <DefaultAmpControls />
            {/*Currently hidden since this will become functional in a future feature but component is ready for use*/}
            {/*<EffectPedal mainColor="#f46616" name="Distortion" />*/}
        </Box>
    );
}