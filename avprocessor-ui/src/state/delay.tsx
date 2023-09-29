import React, { useState, PropsWithChildren } from "react"
import { Speaker } from '../state/speaker'


export enum DelayType {
    MS,
    FEET,
    METERS
}



const initDelayType: DelayType = DelayType.FEET

const delayContext = {
    delayType: initDelayType,
    setDelayType: (update: DelayType) => { },
    //convertForService: () => { }
}


export const DelayContext = React.createContext(delayContext)

const SPEED_OF_SOUND_IN_FEET = 1.0 / 1.125328084
const SPEED_OF_SOUND_IN_METERS = 1.0 / 3.43
//exported for testing
export const convertForServicePure = (delayType: DelayType, value: number) => {
    if (delayType === DelayType.FEET) {
        const delay = SPEED_OF_SOUND_IN_FEET * value //ms delay 
    }
    else if (delayType === DelayType.METERS) {
        const delay = SPEED_OF_SOUND_IN_METERS * value
    }
    else {
        return value
    }
}

export const convertFromAllSpeakers = (delayType: DelayType, speakers: Speaker[]) => {
    const offsets = Math.max(speakers.map(({ delay })))
}



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
