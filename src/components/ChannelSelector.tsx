import {DropdownSelector} from "./selection/DropdownSelector.tsx";

interface ChannelSelectorProps {
    channels: { label: string; value: string }[];
    currentChannelId: string;
    onChannelChange: (id: string) => void;
    onAdd: () => void;
}

export function ChannelSelector({channels, currentChannelId, onChannelChange, onAdd}: ChannelSelectorProps) {
    const selectedChannel = channels.find(ch => ch.value === currentChannelId);

    const handleSelectionChange = (value: string | number) => {
        const nextChannelId = typeof value === "string" ? value : String(value);
        onChannelChange(nextChannelId);
    };

    return (
        <DropdownSelector
            label="Channels"
            options={channels}
            selectedValue={selectedChannel ? selectedChannel.value : ""}
            onSelectionChange={handleSelectionChange}
            onAdd={onAdd}
            hasBorder={false}
            hasLabel={false}

        />
    );

}
