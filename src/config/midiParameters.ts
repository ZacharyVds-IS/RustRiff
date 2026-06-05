import {MidiTargetParameter} from "../domain";

export interface ParameterOption {
    value: MidiTargetParameter;
    label: string;
    description: string;
    icon: string;
}

export const PARAMETER_OPTIONS: Record<string, ParameterOption[]> = {
    Wah: [
        {value: "ToggleBypass", label: "On / Off", description: "Toggle the wah effect on and off", icon: "⏻"},
        {value: "WahPedalPosition", label: "Sweep", description: "Control the wah filter sweep position (0–127)", icon: "〰"},
    ],
    Delay: [
        {value: "ToggleBypass", label: "On / Off", description: "Toggle the delay effect on and off", icon: "⏻"},
        {value: "DelayLevel", label: "Wet Level", description: "Control the delay wet/dry mix", icon: "≋"},
        {value: "DelayTime", label: "Delay Time", description: "Control the delay time in milliseconds", icon: "⧗"},
    ],
    HCDistortion: [
        {value: "ToggleBypass", label: "On / Off", description: "Toggle the distortion effect on and off", icon: "⏻"},
        {value: "DistortionLevel", label: "Clip Gain", description: "Control the distortion output level", icon: "▲"},
        {value: "DistortionThreshold", label: "Threshold", description: "Control the clipping threshold", icon: "⌇"},
    ],
    SCDistortion: [
        {value: "ToggleBypass", label: "On / Off", description: "Toggle the distortion effect on and off", icon: "⏻"},
        {value: "DistortionLevel", label: "Clip Gain", description: "Control the distortion output level", icon: "▲"},
        {value: "DistortionThreshold", label: "Threshold", description: "Control the clipping threshold", icon: "⌇"},
    ],
};

export const FALLBACK_PARAMETERS: ParameterOption[] = [
    {value: "ToggleBypass", label: "On / Off", description: "Toggle the effect on and off", icon: "⏻"},
];
