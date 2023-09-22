import React, { useState, PropsWithChildren } from "react"

export type Version = {
    version: string
    appliedVersion: boolean
}
const initVersions: Version[] = []
interface VersionContextState {
    versions: Version[],
    selectedVersion?: string,
    setSelectedVersion: (_: string) => void,
    addVersion: (_: string) => void,
    setVersions: (_: Version[]) => void,
}
const versionContext: VersionContextState = {
    versions: initVersions,
    setSelectedVersion: (_: string) => { },
    addVersion: (_: string) => { },
    setVersions: (_: Version[]) => { },
}

export const VersionContext = React.createContext(versionContext)


export const VersionProviderComponent = ({ children }: PropsWithChildren) => {

    const addVersion = (version: string) =>
        setContext(currentContext => ({ ...currentContext, versions: [...currentContext.versions, { version, appliedVersion: false }] }))

    const setVersions = (versions: Version[]) =>
        setContext(currentContext => ({ ...currentContext, versions }))

    const setSelectedVersion = (version: string) =>
        setContext(currentContext => ({ ...currentContext, selectedVersion: version }))


    const initState = {
        versions: initVersions,
        addVersion,
        setVersions,
        setSelectedVersion
    }
    const [context, setContext] = useState(initState)

    return (
        <VersionContext.Provider value={context}>
            {children}
        </VersionContext.Provider>
    )
}
