import {useEffect, useState} from "react";
import {
    AlgorithmicLatencyDto,
    ExecutionTimingDto,
    measureAllDspAlgorithmicLatency,
    measureAllDspCpuTimings,
} from "../domain";

export function useDspMetrics(enabled: boolean) {
    const [latency, setLatency] = useState<AlgorithmicLatencyDto[]>([]);
    const [cpuTimings, setCpuTimings] = useState<ExecutionTimingDto[]>([]);

    useEffect(() => {
        if (enabled) {
            const fetchTimings = async () => {
                try {
                    const [latencyResults, cpuTimingResults] = await Promise.all([
                        measureAllDspAlgorithmicLatency(),
                        measureAllDspCpuTimings(),
                    ] as const);
                    setLatency(latencyResults || []);
                    setCpuTimings(cpuTimingResults || []);
                } catch (error) {
                    console.error("Failed to fetch latency metrics:", error);
                }
            };
            fetchTimings();
        } else {
            setLatency([]);
            setCpuTimings([]);
        }
    }, [enabled]);

    function getTimingValue(processorName: string): string {
        const timing = latency.find(t => t.processor_name === processorName);
        return timing ? `${timing.latency_ms.toFixed(3)} ms (${timing.latency_samples} samples)` : "-";
    }

    function getCpuTimeValue(processorName: string): string {
        const timing = cpuTimings.find(t => t.processor_name === processorName);
        return timing ? `${timing.execution_us_per_sample.toFixed(3)} µs/sample` : "-";
    }

    return {latency, cpuTimings, getTimingValue, getCpuTimeValue};
}
