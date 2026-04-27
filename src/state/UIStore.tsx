import {create} from "zustand/react";

interface UIState {
    showLatencyImpacts: boolean;
    setShowLatencyImpacts: (show: boolean) => void;
}

export const useUIStore = create<UIState>((set) => ({
    showLatencyImpacts: false,
    setShowLatencyImpacts: (show: boolean) => {
        set({ showLatencyImpacts: show });
    },
}));

