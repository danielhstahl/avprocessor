import { Version } from "../state/version"

export function getVersions(): Promise<Version[]> {
    return fetch("/versions", {
        method: "GET",
    }).then(r => r.json()).catch(() => [])
}

