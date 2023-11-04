import { createContext, PropsWithChildren, useContext, useReducer } from "react"
import { SPEAKER_OPTIONS } from "./speaker"
export type Filter = {
    speaker: string
    freq: number
    gain: number
    q: number
}

export interface FilterWithIndex extends Filter {
    index: number
}

const initialState: State = {
    filters: []
}

type State = {
    filters: FilterWithIndex[]
}
export enum FilterAction {
    UPDATE,
    ADD,
    REMOVE,
    INIT,
    SET,
}

interface ActionInterface {
    type: FilterAction;
}

interface FilterActionInterface extends ActionInterface {
    value: FilterWithIndex;
}

interface ConfigurationActionInterface extends ActionInterface {
    value: string;
}

interface FiltersActionInterface extends ActionInterface {
    value: Filter[];
}

type Action = FilterActionInterface | ConfigurationActionInterface | FiltersActionInterface;

const FilterContext = createContext({
    state: initialState,
    dispatch: (_: Action) => { }
})

const getDefaultSettings: (speaker: string, i: number) => FilterWithIndex = (speaker: string, index: number) => ({
    speaker,
    freq: index * 50,
    gain: 0,
    q: 0,
    index
})

type SpeakerFilter = {
    filters: FilterWithIndex[],
    storeIndeces: {
        [key: string]: number,
    }
}

const INDEX_START = 1 //could be zero as well, doesnt really matter
//exported for testing
export const setFiltersPure = (filters: Filter[]) => filters.reduce<SpeakerFilter>((agg: SpeakerFilter, v: Filter) => {
    const { filters, storeIndeces } = agg
    const index = storeIndeces[v.speaker] === undefined ? INDEX_START : storeIndeces[v.speaker] + 1
    return {
        filters: [...filters, { ...v, index }],
        storeIndeces: { //this is just to keep track of the index per speaker.  Kind of lame, but best I can think of ATM
            ...storeIndeces,
            [v.speaker]: index
        }
    }
}, { filters: [], storeIndeces: {} }).filters

//exported for testing
export const setFilterBase = (speakerConfiguration: string, current_filters: FilterWithIndex[]) => {
    const baseSpeakers = SPEAKER_OPTIONS.find(s => s.label === speakerConfiguration)
    return baseSpeakers ? baseSpeakers.speakers.reduce<FilterWithIndex[]>((filters, baseSpeaker) => {
        const existingFilters = current_filters.filter(s => s.speaker === baseSpeaker.speaker)
        return existingFilters.length > 0 ? [...filters, ...existingFilters] : filters
    }, []) : undefined
}

export function filterReducer(state: State, action: Action): State {
    switch (action.type) {
        case FilterAction.UPDATE:
            const filterToUpdate = action.value as FilterWithIndex
            return {
                filters: state.filters.map(v => v.speaker === filterToUpdate.speaker && v.index === filterToUpdate.index ? filterToUpdate : v),
            }
        case FilterAction.ADD:
            const speaker = action.value as string
            return {
                filters: [...state.filters, getDefaultSettings(speaker, state.filters.filter(v => v.speaker === speaker).length + INDEX_START)]
            }
        case FilterAction.INIT:
            const speakerConfiguration = action.value as string
            return {
                filters: setFilterBase(speakerConfiguration, state.filters) || state.filters
            }
        case FilterAction.REMOVE:
            const filterToRemove = action.value as FilterWithIndex
            return {
                filters: state.filters.filter(v => !(v.speaker === filterToRemove.speaker && v.index === filterToRemove.index)),
            }
        case FilterAction.SET:
            const filters = action.value as Filter[]
            return {
                filters: setFiltersPure(filters)
            }
        default:
            return state
    }
}

export const FilterProvider = ({ children }: PropsWithChildren) => {
    const [state, dispatch] = useReducer(filterReducer, initialState);
    return (
        <FilterContext.Provider value={{ state, dispatch }}>
            {children}
        </FilterContext.Provider>
    );
};

export const useFilter = () => {
    const context = useContext(FilterContext);
    if (!context) {
        throw new Error("useFilter must be used within a FilterProvider");
    }
    return context;
}
