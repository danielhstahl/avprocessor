import { useReducer, useContext, createContext, PropsWithChildren } from "react"

export type Version = {
    version: string
    appliedVersion: boolean
}

type State = {
    versions: Version[],
    selectedVersion?: string
}

export enum VersionAction {
    ADD,
    SELECT,
    SET_APPLIED,
    REMOVE,
    INIT
}

interface ActionInterface {
    type: VersionAction;
}

interface VersionActionInterface extends ActionInterface {
    value: string;
}

interface VersionsActionInterface extends ActionInterface {
    value: Version[];
}
type Action = VersionActionInterface | VersionsActionInterface;

const initialState: State = {
    versions: []
}
const VersionContext = createContext({
    state: initialState,
    dispatch: (_: Action) => { }
})

export function versionReducer(state: State, action: Action): State {
    switch (action.type) {
        case VersionAction.ADD:
            const versionToAdd = action.value as string
            return {
                ...state,
                versions: [...state.versions, { version: versionToAdd, appliedVersion: false }]
            }
        case VersionAction.REMOVE:
            const versionToRemove = action.value as string
            return {
                ...state,
                versions: state.versions.filter(v => v.version !== versionToRemove)
            }
        case VersionAction.INIT:
            const versions = action.value as Version[]
            return {
                ...state,
                versions
            }
        case VersionAction.SELECT:
            const selectedVersion = action.value as string
            return {
                ...state,
                selectedVersion
            }
        case VersionAction.SET_APPLIED:
            const appliedVersion = action.value as string
            return {
                ...state,
                versions: state.versions.map(({ version }) => ({ version, appliedVersion: version === appliedVersion }))
            }
        default:
            return state
    }
}

interface VersionProps extends PropsWithChildren {
    versionState?: State
}
export const VersionProvider = ({ versionState = initialState, children }: VersionProps) => {
    const [state, dispatch] = useReducer(versionReducer, versionState);

    return (
        <VersionContext.Provider value={{ state, dispatch }}>
            {children}
        </VersionContext.Provider>
    );
};

export const useVersion = () => {
    const context = useContext(VersionContext);
    if (!context) {
        throw new Error("useVersion must be used within a VersionProvider");
    }
    return context;
}