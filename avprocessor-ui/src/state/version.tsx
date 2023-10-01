import { useReducer, useContext, createContext, PropsWithChildren } from "react"

export type Version = {
    version: number
    appliedVersion: boolean,
    versionDate: string
}

type State = {
    versions: Version[],
    selectedVersion?: number
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
    value: Version;
}

interface VersionSelectActionInterface extends ActionInterface {
    value: number;
}
interface VersionsActionInterface extends ActionInterface {
    value: Version[];
}
type Action = VersionActionInterface | VersionsActionInterface | VersionSelectActionInterface;

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
            const versionToAdd = action.value as Version
            return {
                ...state,
                versions: [...state.versions, versionToAdd]
            }
        case VersionAction.REMOVE:
            const versionToRemove = action.value as number
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
            const selectedVersion = action.value as number
            return {
                ...state,
                selectedVersion
            }
        case VersionAction.SET_APPLIED:
            const appliedVersion = action.value as number
            return {
                ...state,
                versions: state.versions.map((version) => ({ ...version, appliedVersion: version.version === appliedVersion }))
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