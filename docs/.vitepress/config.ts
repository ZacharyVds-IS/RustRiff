import {defineConfig} from 'vitepress'
import {katex} from "@mdit/plugin-katex";

// https://vitepress.dev/reference/site-config
export default defineConfig({
    base: '/RustRiff/',
    title: "RustRiff documentation",
    description: "Documentation combining both frontend and backend docs for RustRiff",
    themeConfig: {
        nav: [
            {text: 'Home', link: '/'},
            {text: 'Guide', link: '/project_docs/overview'},
            {text: 'Frontend Docs', link: '/frontend/index.html'},
            {text: 'Backend Docs', link: '/backend/doc/rustriff_lib/index.html'}
        ],

        sidebar: [
            {
                text: 'Project Docs',
                items: [
                    {text: 'Overview', link: '/project_docs/overview'},
                    {text: 'Project Structure', link: '/project_docs/project-structure'},
                ]
            },
            {
                text: 'Amp Design',
                items: [
                    {text: 'Relation to real amp', link: '/project_docs/amp_design/structure'},
                    {text: 'Audio Drivers', link: '/project_docs/amp_design/driver_support.md'},
                    {text: 'Hotkeys', link: '/project_docs/amp_design/keybinds.md'},
                    {text: 'MIDI support', link: '/project_docs/amp_design/midi-support.md'},
                    {text: 'Resampling', link: '/project_docs/amp_design/resampling.md'},
                    {text: 'Visual analysis', link: '/project_docs/amp_design/spectrum-analyzer.md'},
                    {text: 'Tablatures', link: '/project_docs/amp_design/tabs.md'},
                    {text: 'Tuner', link: '/project_docs/amp_design/tuner.md'},
                    {text: 'Latency', link: '/project_docs/amp_design/latency'},
                    {text: 'Gain', link: '/project_docs/amp_design/gain'},
                    {text: 'Master Volume', link: '/project_docs/amp_design/master-volume'},
                    {text: 'Tone Stack', link: '/project_docs/amp_design/tone-stack'}
                ]
            },
            {
                text: 'Persistency',
                items: [
                    {text: 'Storing data', link: '/project_docs/persistency/storing-data'}
                ]
            },
            {
                text: 'Effects',
                items: [
                    {text: 'Effect chain', link: '/project_docs/effects/chain'},
                    {text: 'Distortion', link: '/project_docs/effects/distortion'},
                    {text: 'Cabinet Simulation (IR)', link: '/project_docs/effects/cabinet_simulation_ir'},
                    {text: 'Recording an impulse response', link: '/project_docs/effects/custom_ir_recording'},
                    {text: 'Wah', link: '/project_docs/effects/wah'},
                ]
            },
            {
                text: 'Testing',
                items: [
                    {text: 'General Concepts', link: '/project_docs/testing/Testing.md'},
                    {text: 'Frontend Testing', link: '/project_docs/testing/frontend-testing.md'},
                    {text: 'Backend Testing', link: '/project_docs/testing/backend-testing.md'},
                ]
            },
            {
                text: 'API References',
                items: [
                    {text: 'Frontend API (TypeDoc)', link: '/frontend/index.html'},
                    {text: 'Backend API (Rustdoc)', link: '/backend/doc/rustriff_lib/index.html'}
                ]
            },
            {
                text: 'Research',
                items: [
                    {text: 'Architecture Description (arc42)', link: '/arc42/arc42_project_descriptions.md'},
                    {text: 'Neural Networking in DSP', link: '/arc42/neural-network-research.md'}
                ]
            }
        ],

        socialLinks: [
            {icon: 'github', link: 'https://github.com/ZacharyVds-IS/Guitar-Amplifier'}
        ]
    },
    markdown: {
        config: (md) => {
            md.use(katex)
        }
    }
})
