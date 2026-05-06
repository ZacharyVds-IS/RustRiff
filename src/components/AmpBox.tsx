import {EffectDto} from "../domain";
import {Box, Typography} from "@mui/material";

interface AmpBoxProps {
    onSelectionChange: (selected: EffectDto | "amp") => void;
    isAmpSelected: () => boolean;
    selectedBorder: {
        border: string;
        borderColor: string;
        boxShadow: string;
    };
}

export function AmpBox({
                           onSelectionChange,
                           isAmpSelected,
                           selectedBorder
                       }: AmpBoxProps) {

    const selected = isAmpSelected();

    return (
        <Box
            onClick={() => onSelectionChange("amp")}
            sx={{
                display: 'flex',
                flexDirection: 'column',
                alignItems: 'center',
                cursor: 'pointer',
                position: 'relative'
            }}
        >
            <Box
                sx={{
                    display: 'flex',
                    flexDirection: "column",
                    alignItems: 'center',
                    height: 75
                }}
            >
                <Box
                    sx={{
                        width: 60,
                        height: 60,
                        bgcolor: 'background.paper',
                        border: '1px solid',
                        borderColor: 'text.secondary',
                        borderRadius: 2,
                        transition: 'border 0.15s, box-shadow 0.15s',
                        ...(selected && selectedBorder),
                    }}
                />
            </Box>

            <Typography
                variant="caption"
                sx={{
                    mt: 1,
                    color: selected ? 'primary.main' : 'text.primary',
                    fontWeight: selected ? 700 : 500,
                    fontSize: '0.75rem',
                }}
            >
                Amp
            </Typography>
        </Box>
    );
}