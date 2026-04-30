import {useEffect, useState} from "react";
import {ChannelDto, getAllChannels} from "../domain";

export function useChannels() {
    const [channels, setChannels] = useState<ChannelDto[]>([]);
    const [loading, setLoading] = useState(true);
    const [error, setError] = useState<string | null>(null);

    useEffect(() => {
        const fetchChannels = async () => {
            try {
                setLoading(true);
                setError(null);
                console.log("useChannels: Fetching channels...");
                const data = await getAllChannels();
                console.log("useChannels: Channels fetched successfully:", data);
                setChannels(data);
                if (data.length === 0) {
                    console.warn("useChannels: No channels returned from backend");
                }
            } catch (err) {
                const errorMessage = err instanceof Error ? err.message : String(err);
                console.error("useChannels: Failed to fetch channels:", err);
                setError(errorMessage);
                setChannels([]);
            } finally {
                setLoading(false);
            }
        };

        fetchChannels();
    }, []);

    return {
        channels,
        loading,
        error,
    };
}