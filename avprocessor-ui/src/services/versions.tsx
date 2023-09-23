export const getVersions = () => fetch("/versions", {
    method: "GET",
}).then(r => r.json()).catch(() => [])

