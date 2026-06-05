import {useCallback, useState} from "react";
import {getAllIrProfiles, IrProfileDto, removeIrProfile} from "../domain";

function getUserFriendlyIrError(error: unknown, operation: "load" | "upload" | "remove"): string {
    const rawMessage = extractRawErrorMessage(error);
    const lower = rawMessage.toLowerCase();

    if (lower.includes("only .wav")) return "Only .wav IR files are supported.";

    if (lower.includes("unsupported wav format") || lower.includes("unexpected fmt chunk size"))
        return "This WAV encoding is not supported. Re-export as PCM 16/24-bit or Float32 WAV.";

    if (lower.includes("no impulse detected") || lower.includes("first sample is silence"))
        return "This file does not look like a valid impulse response (the start is effectively silent).";

    if (lower.includes("already exists"))
        return "An IR with this file name already exists. Rename the file or remove the existing one first.";

    if (lower.includes("currently used by an effect chain"))
        return "This IR is currently used in your effect chain, so it cannot be removed.";

    if (lower.includes("default ir profiles cannot be removed"))
        return "Default IR profiles cannot be removed.";

    if (lower.includes("failed to lock"))
        return "The app is busy processing audio state. Please try again.";

    switch (operation) {
        case "load": return "Could not load IR profiles. Please try again.";
        case "upload": return "Could not upload this IR file. Please check the file and try again.";
        case "remove": return "Could not remove this IR profile right now. Please try again.";
    }
}

function extractRawErrorMessage(error: unknown): string {
    if (typeof error === "string") return error;

    if (error && typeof error === "object") {
        if ("message" in error && typeof (error as {message?: unknown}).message === "string")
            return (error as {message: string}).message;
        if ("error" in error && typeof (error as {error?: unknown}).error === "string")
            return (error as {error: string}).error;
    }

    return "Unknown IR error";
}

export function useIrProfiles() {
    const [cabinetIrProfiles, setCabinetIrProfiles] = useState<IrProfileDto[]>([]);
    const [cabinetIrActionError, setCabinetIrActionError] = useState("");

    const refreshIrProfiles = useCallback(async () => {
        try {
            const profiles = await getAllIrProfiles();
            setCabinetIrProfiles(profiles);
            setCabinetIrActionError("");
        } catch (error) {
            console.error("Failed to load cabinet IR profiles:", error);
            setCabinetIrProfiles([]);
            setCabinetIrActionError(getUserFriendlyIrError(error, "load"));
        }
    }, []);

    const handleRemoveSelectedCabinetIr = async (profile: IrProfileDto | undefined) => {
        if (!profile) return;

        try {
            await removeIrProfile({fileName: profile.file_name});
            await refreshIrProfiles();
        } catch (error) {
            console.error("Failed to remove IR profile:", error);
            setCabinetIrActionError(getUserFriendlyIrError(error, "remove"));
        }
    };

    return {
        cabinetIrProfiles,
        cabinetIrActionError,
        setCabinetIrActionError,
        refreshIrProfiles,
        handleRemoveSelectedCabinetIr,
    };
}
