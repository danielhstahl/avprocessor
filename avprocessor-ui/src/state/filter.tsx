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
    filters: {}
}

type State = {
    filters: SpeakerFilter
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


export type SpeakerFilter = {
    [key: string]: FilterWithIndex[],
}

const INDEX_START = 1 //could be zero as well, doesnt really matter
//exported for testing
export const perSpeakerFilters: (filters: Filter[]) => SpeakerFilter = (filters: Filter[]) => {
    return filters.reduce<SpeakerFilter>((agg, filter) => {
        const currentFilters = agg[filter.speaker] || []
        const index = currentFilters.length + INDEX_START
        const filterWithIndex = { ...filter, index }
        return {
            ...agg,
            [filter.speaker]: [...currentFilters, filterWithIndex]
        }
    }, {})
}

//exported for testing
export const setFilterBase = (
    speakerConfiguration: string,
    current_filters: SpeakerFilter
) => {
    const baseSpeakers = SPEAKER_OPTIONS.find(s => s.label === speakerConfiguration)
    return baseSpeakers ? baseSpeakers.speakers.reduce<SpeakerFilter>((perSpeakerFilters, baseSpeaker) => {
        const existingFilters = current_filters[baseSpeaker.speaker] || []
        return { ...perSpeakerFilters, [baseSpeaker.speaker]: existingFilters }
    }, {}) : undefined
}

export function filterReducer(state: State, action: Action): State {
    switch (action.type) {
        case FilterAction.UPDATE:
            const filterToUpdate = action.value as FilterWithIndex
            const speakerFiltersToUpdate = state.filters[filterToUpdate.speaker]
            return {
                filters: {
                    ...state.filters,
                    [filterToUpdate.speaker]: speakerFiltersToUpdate.map(v => v.index === filterToUpdate.index ? filterToUpdate : v),
                }
            }
        case FilterAction.ADD:
            const speaker = action.value as string
            const speakerFilterToAdd = state.filters[speaker]
            return {
                filters: {
                    ...state.filters,
                    [speaker]: [...speakerFilterToAdd, getDefaultSettings(speaker, speakerFilterToAdd.length + INDEX_START)]
                }
            }
        case FilterAction.INIT:
            const speakerConfiguration = action.value as string
            return {
                filters: setFilterBase(speakerConfiguration, state.filters) || state.filters
            }
        case FilterAction.REMOVE:
            const filterToRemove = action.value as FilterWithIndex
            const speakerFilterToRemove = state.filters[filterToRemove.speaker]
            return speakerFilterToRemove ? {
                filters: {
                    ...state.filters,
                    [filterToRemove.speaker]: speakerFilterToRemove.filter(v => v.index !== filterToRemove.index),
                }
            } : state
        case FilterAction.SET:
            const filters = action.value as Filter[]
            return {
                filters: perSpeakerFilters(filters)
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
