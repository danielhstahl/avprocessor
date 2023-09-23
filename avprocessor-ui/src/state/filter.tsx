import React, { useState, PropsWithChildren } from "react"
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

const initFilters: FilterWithIndex[] = []
const filterContext = {
    filters: initFilters,
    addFilter: (update: string) => { },
    setFilters: (update: Filter[]) => { },
    setFilterBase: (update: string) => { },
    updateFilter: (update: FilterWithIndex) => { },
    removeFilter: (update: FilterWithIndex) => { }
}
const getDefaultSettings: (speaker: string, i: number) => FilterWithIndex = (speaker: string, index: number) => {
    return {
        speaker,
        freq: index * 50,
        gain: 0,
        q: 0,
        index
    }
}

export const FilterContext = React.createContext(filterContext)
type SpeakerFilter = {
    filters: FilterWithIndex[],
    storeIndeces: {
        [key: string]: number,
    }
}

//exported for testing
export const setFiltersPure = (filters: Filter[]) => filters.reduce<SpeakerFilter>((agg: SpeakerFilter, v: Filter) => {
    const { filters, storeIndeces } = agg
    const index = storeIndeces[v.speaker] ? storeIndeces[v.speaker] + 1 : 1
    return {
        filters: [...filters, { ...v, index }],
        storeIndeces: { //this is just to keep track of the index per speaker.  Kind of lame, but best I can think of ATM
            ...storeIndeces,
            [v.speaker]: index
        }
    }
}, { filters: [], storeIndeces: {} }).filters



export const FilterProviderComponent = ({ children }: PropsWithChildren) => {

    const addFilter = (speaker: string) =>
        setContext(currentContext => ({
            ...currentContext,
            filters: [...currentContext.filters, getDefaultSettings(speaker, currentContext.filters.filter(v => v.speaker === speaker).length)]
        })
        )
    const setFilters = (filters: Filter[]) =>
        setContext(currentContext => ({
            ...currentContext,
            filters: setFiltersPure(filters)
        })
        )

    const setFilterBase = (speakerConfiguration: string) => setContext((currentContext) => {
        const baseSpeakers = SPEAKER_OPTIONS.find(s => s.label === speakerConfiguration)
        return baseSpeakers ? {
            ...currentContext,
            filters: baseSpeakers.speakers.reduce<FilterWithIndex[]>((filters, baseSpeaker) => {
                const existingFilters = currentContext.filters.filter(s => s.speaker === baseSpeaker.speaker)
                return existingFilters.length > 0 ? [...filters, ...existingFilters] : [...filters, getDefaultSettings(baseSpeaker.speaker, 0)]
            }, [])
        } : currentContext
    })

    const updateFilter = (filter: FilterWithIndex) => setContext((currentContext) => ({
        ...currentContext,
        filters: currentContext.filters.map(v => v.speaker === filter.speaker && v.index === filter.index ? filter : v),
    }))

    const removeFilter = (filter: FilterWithIndex) => setContext((currentContext) => ({
        ...currentContext,
        filters: currentContext.filters.filter(v => !(v.speaker === filter.speaker && v.index === filter.index)),
    }))

    const initState = {
        filters: initFilters,
        addFilter,
        setFilters,
        setFilterBase,
        updateFilter,
        removeFilter
    }
    const [context, setContext] = useState(initState)



    return (
        <FilterContext.Provider value={context}>
            {children}
        </FilterContext.Provider>
    )
}
