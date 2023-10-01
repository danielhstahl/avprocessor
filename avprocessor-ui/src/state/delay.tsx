import { useReducer, useContext, createContext, PropsWithChildren } from "react"


export enum DelayType {
    MS = 'ms',
    FEET = 'feet',
    METERS = 'meters'
}
export enum DelayAction {
    UPDATE
}



type State = {
    delayType: DelayType
}
type Action = {
    type: DelayAction,
    value: DelayType
}
const initialState = { delayType: DelayType.FEET };

export const delayReducer = (state: State, action: Action) => {
    switch (action.type) {
        case DelayAction.UPDATE:
            return { delayType: action.value };
        default:
            return state;
    }
};

const DelayContext = createContext({ state: initialState, dispatch: (_: Action) => { } });

export const DelayProvider = ({ children }: PropsWithChildren) => {
    const [state, dispatch] = useReducer(delayReducer, initialState);

    return (
        <DelayContext.Provider value={{ state, dispatch }}>
            {children}
        </DelayContext.Provider>
    );
};

export const useDelay = () => {
    const context = useContext(DelayContext);
    if (!context) {
        throw new Error("useDelay must be used within a DelayProvider");
    }
    return context;
};