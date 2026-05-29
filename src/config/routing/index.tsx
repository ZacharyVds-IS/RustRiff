import {createHashRouter} from "react-router-dom";
import {SettingsScreen} from "../../screens/SettingsScreen.tsx";
import {MainScreen} from "../../screens/MainScreen.tsx";
import {AppLayout} from "../../screens/AppLayout.tsx";
import {AnalyzerWindow} from "../../windows/AnalyzerWindow";
import {TunerScreen} from "../../screens/TunerScreen.tsx";
import {TabWindow} from "../../windows/TabWindow";


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