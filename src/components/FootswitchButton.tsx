import {Box} from "@mui/material";

interface FootswitchButtonProps {
    onClick: () => void;
}

export function FootswitchButton({onClick}: FootswitchButtonProps) {
    return (
        <Box
            onClick={onClick}
            sx={{
                width: 'calc(100% + 8px)',
                height: 110,
                flexShrink: 0,
                bgcolor: '#1a1a1a',
                borderRadius: '2px 2px 8px 8px',
                border: '2px solid #000',
                boxShadow: 'inset 0 2px 4px rgba(255,255,255,0.1)',
                display: 'flex',
                justifyContent: 'center',
                alignItems: 'flex-end',
                pb: 1,
                cursor: 'pointer',
                zIndex: 3,
                transition: 'transform 0.05s',
                '&:active': {transform: 'scale(0.98) translateY(2px)'}
            }}
        >
            <Box
                sx={{
                    width: 12,
                    height: 12,
                    borderRadius: '50%',
                    background: 'radial-gradient(circle, #444, #000)',
                    border: '1px solid #333'
                }}
            />
        </Box>
    );
}
