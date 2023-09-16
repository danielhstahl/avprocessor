import React, { useState, PropsWithChildren } from "react"

export type Filter = {
    speaker: string
    freq: number
    gain: number
    q: number
}

const initFilters: Filter[] = []
const filterContext = { filters: initFilters, addFilter: (update: Filter) => { }, setFilters: (update: Filter[]) => { } }

export const FilterContext = React.createContext(filterContext)

export const FilterProviderComponent = ({ children }: PropsWithChildren) => {

    const addFilter = (filter: Filter) =>
        setContext(currentContext => ({
            ...currentContext,
            filters: [...currentContext.filters, filter]
        })
        )
    const setFilters = (filters: Filter[]) =>
        setContext(currentContext => ({
            ...currentContext,
            filters
        })
        )
    const initState = {
        filters: initFilters,
        addFilter,
        setFilters
    }
    const [context, setContext] = useState(initState)

    return (
        <FilterContext.Provider value={context}>
            {children}
        </FilterContext.Provider>
    )
}
