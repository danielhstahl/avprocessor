import { Speaker } from '../state/speaker'
import { Filter } from '../state/filter'

export const getConfiguration = (version: string) => fetch(`/config/${version}`, {
    method: "GET",
}).then(r => r.json())

export type ConfigPayload = { speakers: Speaker[], filters: Filter[] }

export const saveConfig = (body: ConfigPayload) => fetch(`/config`, {
    method: "PUT",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(body)
}).then(r => r.text())

export const applyConfig = (version: string) => fetch(`/config/apply/${version}`, {
    method: "POST",
    headers: { "Content-Type": "application/json" }
}).then(r => r.text())

export const deleteConfig = (version: string) => fetch(`/config/${version}`, {
    method: "DELETE",
}).then(r => r.text())