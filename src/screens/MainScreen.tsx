import {Box, Button} from "@mui/material";
import {EffectChain} from "../components/EffectChain.tsx";
import {EffectControls} from "../components/EffectControls.tsx";
import {measureAllDspTimings} from "../domain/commands";

async function handleMeasureAllTimings() {
    try {
        const timings = await measureAllDspTimings();
        console.log("DSP Chain Execution Timings:");
        timings.forEach((timing) => {
            console.log(`  ${timing.processor_name}: ${timing.execution_us_per_sample.toFixed(6)} us/sample`);
        });
    } catch (error) {
        console.error("Failed to measure DSP timings:", error);
    }
}

export function MainScreen() {
    return (
        <Box sx={{ p: 4, display: "flex", flexDirection: "column", gap: 2 }}>
            <EffectChain/>
            <EffectControls/>
            <Button
                onClick={handleMeasureAllTimings}
                variant="contained"
            >
                Measure All DSP Timings
            </Button>
        </Box>
    );
}
