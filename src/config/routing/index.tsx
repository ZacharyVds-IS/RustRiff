import {createHashRouter} from "react-router-dom";
import {SettingsScreen} from "../../screens/SettingsScreen.tsx";
import {MainScreen} from "../../screens/MainScreen.tsx";
import {AppLayout} from "../../screens/AppLayout.tsx";
import {AnalyzerWindow} from "../../windows/AnalyzerWindow";
import {TabWindow} from "../../windows/TabWindow";
import {TunerScreen} from "../../screens/TunerScreen.tsx";
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
                path: "tuner",
                element: <TunerScreen/>,
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
    {
        path: "/tab",
        element: <TabWindow/>,
    }
]);