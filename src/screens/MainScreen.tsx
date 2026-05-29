import {Box} from "@mui/material";
import {EffectChain} from "../components/EffectChain.tsx";
import {DefaultAmpControls} from "../components/DefaultAmpControls.tsx";
import {EffectPedal} from "../components/EffectPedal.tsx";
import {CabinetEffect} from "../components/CabinetEffect.tsx";
import {useAmpStore} from "../state/AmpConfigStore.tsx";
import {useEffect, useState} from "react";
import {EffectDto} from "../domain";

type EffectSelection =
    | "amp"
    | {
        kind: EffectDto["kind"];
        effectId: string;
    };

export function MainScreen() {
    const activeChannel = useAmpStore((state) =>
        state.channels.find((c) => c.id === state.current_channel)
    );
    const currentChannelId = useAmpStore((state) => state.current_channel);
    const [selection, setSelection] = useState<EffectSelection>("amp");
    useEffect(() => {
        setSelection("amp");
    }, [currentChannelId]);

    const resolvedSelection: EffectDto | "amp" | undefined =
        selection === "amp"
            ? "amp"
            : activeChannel?.effect_chain.find(
                (effect) =>
                    effect.kind === selection.kind &&
                    effect.data.id === selection.effectId,
            );

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
                    onSelectionChange={(selected: EffectDto | "amp") => {
                        if (selected === "amp") {
                            setSelection("amp");
                            return;
                        }

                        setSelection({
                            kind: selected.kind,
                            effectId: selected.data.id,
                        });
                    }}
                />
            }
            {
                    (resolvedSelection === "amp" || !resolvedSelection)
                    ? <DefaultAmpControls/>
                    : resolvedSelection.kind === "Cabinet"
                    ? <CabinetEffect effect={resolvedSelection}/>
                    : <EffectPedal effect={resolvedSelection}/>
            }
        </Box>
    );
}