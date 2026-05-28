import {createHashRouter} from "react-router-dom";
import {SettingsScreen} from "../../screens/SettingsScreen.tsx";
import {MainScreen} from "../../screens/MainScreen.tsx";
import {AppLayout} from "../../screens/AppLayout.tsx";
import {AnalyzerWindow} from "../../windows/AnalyzerWindow";
import {MidiConfigScreen} from "../../screens/MidiConfigScreen.tsx";


export const router = createHashRouter([
    {
        path: "/",
        element: <AppLayout/>,
        children: [
            {
                index: true,
                element: <MainScreen/>,
            },
            {
                path: "settings",
                element: <SettingsScreen/>,
            },
            {
                path:"midi-mappings",
                element:<MidiConfigScreen/>
            }
        ],
    },
    {
        path: "/analyzer",
        element: <AnalyzerWindow/>,
    },
]);