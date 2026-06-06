import {Box} from "@mui/material";
import {EffectChain} from "../components/EffectChain.tsx";
import {DefaultAmpControls} from "../components/DefaultAmpControls.tsx";
import {EffectPedal} from "../components/EffectPedal.tsx";
import {WahPedal} from "../components/WahPedal.tsx";
import {CabinetEffect} from "../components/CabinetEffect.tsx";
import {KeybindsDialog} from "../components/dialogs/KeybindsDialog.tsx";
import {useEffectSelection} from "../hooks/useEffectSelection.ts";
import {EffectDto} from "../domain";

export function MainScreen() {
    const {
        setSelection,
        resolvedSelection,
        effectChain,
        isKeybindsOpen,
        setIsKeybindsOpen,
        activeChannel,
    } = useEffectSelection();

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
                    effects={effectChain}
                    selected={resolvedSelection ?? "amp"}
                    onOpenKeybinds={() => setIsKeybindsOpen(true)}
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
                    : resolvedSelection.kind === "Wah"
                    ? <WahPedal effect={resolvedSelection}/>
                    : <EffectPedal effect={resolvedSelection}/>
            }
            <KeybindsDialog open={isKeybindsOpen} onClose={() => setIsKeybindsOpen(false)}/>
        </Box>
    );
}
