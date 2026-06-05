import {useEffect, useState} from "react";
import * as commands from "../domain/commands";

export function useAudioDriver() {
    const [availableDrivers, setAvailableDrivers] = useState<string[]>(["Default"]);
    const [selectedDriver, setSelectedDriver] = useState<string>("Default");
    const [driverError, setDriverError] = useState<string | null>(null);

    async function loadAudioDrivers() {
        try {
            const [drivers, selected] = await Promise.all([
                commands.getAvailableAudioDrivers(),
                commands.getSelectedAudioDriver(),
            ]);
            setAvailableDrivers(drivers.length > 0 ? drivers : ["Default"]);
            setSelectedDriver(selected || "Default");
            setDriverError(null);
        } catch (err) {
            setDriverError(err instanceof Error ? err.message : "Failed to load audio drivers");
        }
    }

    useEffect(() => {
        void loadAudioDrivers();
    }, []);

    const isAsioMode = selectedDriver.toLowerCase() === "asio";
    const driverOptions = availableDrivers.map((driver) => ({label: driver, value: driver}));

    return {
        availableDrivers,
        selectedDriver,
        setSelectedDriver,
        driverError,
        setDriverError,
        driverOptions,
        isAsioMode,
        loadAudioDrivers,
    };
}
