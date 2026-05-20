import {Alert, Box, Button, CircularProgress, Stack, Typography, useTheme} from "@mui/material";
import {DropdownSelector} from "./selection/DropdownSelector.tsx";
import {BufferLatencyDto} from "../domain";

interface LatencySectionProps {
    bufferSizeOptions: { label: string; value: number }[];
    bufferSizeFrames: number;
    handleBufferSizeChange: (value: string | number) => void;
    bufferSizeSaving: boolean;
    bufferSizeError: string | null;
    bufferLatency: BufferLatencyDto | null;
    handleMeasureRoundTripLatency: () => void;
    roundTripLoading: boolean;
    roundTripError: string | null;
    roundTripLatency: number | null;
}

export function LatencySection({
                                   bufferSizeOptions,
                                   bufferSizeFrames,
                                   handleBufferSizeChange,
                                   bufferSizeSaving,
                                   bufferSizeError,
                                   bufferLatency,
                                   handleMeasureRoundTripLatency,
                                   roundTripLoading,
                                   roundTripError,
                                   roundTripLatency,
                               }: LatencySectionProps) {
    const theme = useTheme();

    return (
        <Box sx={{ mt: 2, pt: 2, borderTop: `1px solid ${theme.palette.divider}` }}>
            <Typography variant="subtitle2" sx={{ fontWeight: "bold", mb: 1 }}>Latency</Typography>

            <Box sx={{ borderRadius: 1, mb: 1.5 }}>
                <Box sx={{ display: "flex", gap: 1, alignItems: "center" }}>
                    <Stack sx={{ width: "100%" }}>
                        <Box sx={{ flex: 1, position: "relative" }}>
                            <DropdownSelector
                                title="Preferred Buffer Size"
                                label="Frames"
                                options={bufferSizeOptions}
                                selectedValue={bufferSizeFrames}
                                onSelectionChange={handleBufferSizeChange}
                            />
                            <Box sx={{ display: "flex", alignItems: "center", gap: 1, mt: 0.75 }}>
                                <Typography variant="caption" sx={{ color: theme.palette.text.secondary, flex: 1 }}>
                                    Lower frames reduce latency but can increase crackles on weaker CPUs.
                                </Typography>
                                {bufferSizeSaving && (
                                    <Stack>
                                        <CircularProgress size={12} thickness={5} />
                                        <Typography variant="caption" sx={{ color: theme.palette.primary.main, fontWeight: "medium" }}>
                                            Applying...
                                        </Typography>
                                    </Stack>
                                )}
                            </Box>
                        </Box>
                    </Stack>
                </Box>
                {bufferSizeError && (
                    <Alert severity="error" sx={{ mt: 1 }}>{bufferSizeError}</Alert>
                )}
            </Box>

            <Box sx={{ p: 1.5, backgroundColor: theme.palette.action.hover, borderRadius: 1, mb: 1.5 }}>
                <Typography variant="body2" sx={{ fontWeight: "bold" }}>Estimated Buffer Latency</Typography>
                {bufferLatency ? (
                    <Typography variant="caption" sx={{ display: "block", mt: 0.5 }}>
                        {bufferLatency.input_buffer_latency_ms.toFixed(2)} ms (input) +{" "}
                        {bufferLatency.output_buffer_latency_ms.toFixed(2)} ms (output) ={" "}
                        <Box component="span" sx={{ fontWeight: "bold", color: theme.palette.primary.main }}>
                            {bufferLatency.total_buffer_latency_ms.toFixed(2)} ms
                        </Box>
                    </Typography>
                ) : (
                    <Typography variant="caption" sx={{ display: "block", mt: 0.5, color: theme.palette.text.secondary }}>
                        Unable to read current buffer latency.
                    </Typography>
                )}
            </Box>

            <Box sx={{ p: 1.5, backgroundColor: theme.palette.action.hover, borderRadius: 1 }}>
                <Typography variant="body2" sx={{ fontWeight: "bold", mb: 1 }}>Measured Round-Trip Latency</Typography>
                <Button
                    variant="outlined"
                    size="small"
                    onClick={handleMeasureRoundTripLatency}
                    disabled={roundTripLoading}
                    fullWidth
                    sx={{ mb: 1 }}
                >
                    {roundTripLoading ? "Measuring..." : "Measure Round-Trip"}
                </Button>

                {roundTripError && <Alert severity="error" sx={{ mb: 1 }}>{roundTripError}</Alert>}

                {roundTripLatency !== null && (
                    <Typography variant="caption" sx={{ display: "block" }}>
                        <Box component="span" sx={{ fontWeight: "bold", color: theme.palette.primary.main }}>
                            {roundTripLatency.toFixed(2)} ms
                        </Box>{" "}
                        measured end-to-end.
                    </Typography>
                )}

                {roundTripLatency === null && !roundTripError && (
                    <Typography variant="caption" sx={{ display: "block", color: theme.palette.text.secondary }}>
                        Route output back into input (for example: Line Out to Line In), then press Measure Round-Trip.
                    </Typography>
                )}
            </Box>
        </Box>
    );
}