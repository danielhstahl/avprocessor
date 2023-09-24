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
    setAppliedVersion: (_: string) => void,
    removeVersion: (_: string) => void,
}
const versionContext: VersionContextState = {
    versions: initVersions,
    setSelectedVersion: (_: string) => { },
    addVersion: (_: string) => { },
    setVersions: (_: Version[]) => { },
    setAppliedVersion: (_: string) => { },
    removeVersion: (_: string) => { },
}

export const VersionContext = React.createContext(versionContext)

interface VersionProviderProps extends PropsWithChildren {
    versions?: Version[]
}
export const VersionProviderComponent = ({ versions = initVersions, children }: VersionProviderProps) => {

    const addVersion = (version: string) =>
        setContext(currentContext => ({ ...currentContext, versions: [...currentContext.versions, { version, appliedVersion: false }] }))

    const setVersions = (versions: Version[]) =>
        setContext(currentContext => ({ ...currentContext, versions }))

    const removeVersion = (version: string) =>
        setContext(currentContext => ({ ...currentContext, versions: currentContext.versions.filter(v => v.version !== version) }))

    const setSelectedVersion = (version: string) =>
        setContext(currentContext => ({ ...currentContext, selectedVersion: version }))

    const setAppliedVersion = (versionApplied: string) =>
        setContext(currentContext => ({
            ...currentContext,
            versions: currentContext.versions.map(({ version }) => ({ version, appliedVersion: version === versionApplied }))
        }))
    const initState = {
        versions: versions,
        addVersion,
        setVersions,
        setSelectedVersion,
        setAppliedVersion,
        removeVersion
    }
    const [context, setContext] = useState(initState)

    return (
        <VersionContext.Provider value={context}>
            {children}
        </VersionContext.Provider>
    )
}
