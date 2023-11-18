import { Speaker } from '../state/speaker'
import { Filter } from '../state/filter'
import { DelayType } from '../state/delay'
import { Version } from '../state/version'
import { DeviceType } from '../state/device'

export type ConfigPayload = {
    speakers: Speaker[],
    filters: Filter[],
    selectedDistance: DelayType,
    device: DeviceType
}

export function getConfiguration(version: number): Promise<ConfigPayload> {
    return fetch(`/config/${version}`, {
        method: "GET",
    }).then(r => r.json())
}

export function saveConfig(body: ConfigPayload): Promise<Version> {
    return fetch(`/config`, {
        method: "PUT",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify(body)
    }).then(r => r.json())
}

export const applyConfig = (version: number) => fetch(`/config/apply/${version}`, {
    method: "POST",
    headers: { "Content-Type": "application/json" }
})

export const deleteConfig = (version: number) => fetch(`/config/${version}`, {
    method: "DELETE",
})