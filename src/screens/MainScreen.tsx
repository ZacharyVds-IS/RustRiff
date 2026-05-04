import {Box} from "@mui/material";
import {EffectChain} from "../components/EffectChain.tsx";
import {DefaultAmpControls} from "../components/DefaultAmpControls.tsx";
import {EffectPedal} from "../components/EffectPedal.tsx";
import {useAmpStore} from "../state/AmpConfigStore.tsx";
import {useEffect, useState} from "react";
import {EffectDto} from "../domain";

export function MainScreen() {
    const activeChannel = useAmpStore((state) =>
        state.channels.find((c) => c.id === state.current_channel)
    );
    const currentChannelId = useAmpStore((state) => state.current_channel);

    const [selection, setSelection] = useState<number | "amp">("amp");
    useEffect(() => {
        setSelection("amp");
    }, [currentChannelId]);
    const resolvedSelection: EffectDto | "amp" | undefined =
        selection === "amp"
            ? "amp"
            : activeChannel?.effect_chain.find((e) => e.data.id === selection);

    return (
        <Box
            sx={{
                p: 4,
                display: "flex",
                flexDirection: "column",
                alignItems: "center",
                justifyContent: "start",
                minHeight: "100vh",
                gap: 4
            }}
        >
            {activeChannel &&
                <EffectChain
                    effects={activeChannel.effect_chain}
                    selected={resolvedSelection ?? "amp"}
                    onSelectionChange={(selected) => {
                        setSelection(selected === "amp" ? "amp" : selected.data.id);
                    }}
                />
            }

            {resolvedSelection === "amp" || !resolvedSelection
                ? <DefaultAmpControls/>
                : <EffectPedal effect={resolvedSelection}/>
            }
        </Box>
    );
}