import { useReducer, useContext, createContext, PropsWithChildren } from "react"


export enum DeviceType {
    OktoDac8 = 'oktodac8',
    ToppingDm7 = 'toppingdm7',
    MotuMk5 = 'motumk5',
    HDMI = 'hdmi'
}
export enum DeviceAction {
    UPDATE
}

type State = {
    deviceType: DeviceType
}
type Action = {
    type: DeviceAction,
    value: DeviceType
}
const initialState = { deviceType: DeviceType.ToppingDm7 };

export const deviceReducer = (state: State, action: Action) => {
    switch (action.type) {
        case DeviceAction.UPDATE:
            return { deviceType: action.value };
        default:
            return state;
    }
};

const DeviceContext = createContext({ state: initialState, dispatch: (_: Action) => { } });

export const DeviceProvider = ({ children }: PropsWithChildren) => {
    const [state, dispatch] = useReducer(deviceReducer, initialState);

    return (
        <DeviceContext.Provider value={{ state, dispatch }}>
            {children}
        </DeviceContext.Provider>
    );
};

export const useDevice = () => {
    const context = useContext(DeviceContext);
    if (!context) {
        throw new Error("useDevice must be used within a DeviceProvider");
    }
    return context;
};