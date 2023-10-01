import { VersionAction, versionReducer } from './version'



describe("versionReducer", () => {
    it("adds version when no version", () => {
        const results = versionReducer({ versions: [] }, { type: VersionAction.ADD, value: { version: 1, appliedVersion: false, versionDate: "2023" } })
        expect(results.versions).toEqual([{ version: 1, appliedVersion: false, versionDate: "2023" }])
    })
    it("adds version when existing versions", () => {
        const results = versionReducer({
            versions: [
                { version: 1, appliedVersion: true, versionDate: "2023" }
            ]
        }, { type: VersionAction.ADD, value: { version: 2, appliedVersion: false, versionDate: "2023" } })
        expect(results.versions).toEqual([{ version: 1, appliedVersion: true, versionDate: "2023" }, { version: 2, appliedVersion: false, versionDate: "2023" }])
    })
    it("removes version when it exists", () => {
        const results = versionReducer({
            versions: [
                { version: 1, appliedVersion: true, versionDate: "2023" }
            ]
        }, { type: VersionAction.REMOVE, value: 1 })
        expect(results.versions).toEqual([])
    })
    it("does nothing if version does not exist", () => {
        const results = versionReducer({ versions: [{ version: 1, appliedVersion: true, versionDate: "2023" }] }, { type: VersionAction.REMOVE, value: 2 })
        expect(results.versions).toEqual([{ version: 1, appliedVersion: true, versionDate: "2023" }])
    })
    it("inits", () => {
        const results = versionReducer({
            versions: [
                { version: 1, appliedVersion: true, versionDate: "2023" }
            ]
        }, { type: VersionAction.INIT, value: [{ version: 2, appliedVersion: false, versionDate: "2023" }] })
        expect(results.versions).toEqual([{ version: 2, appliedVersion: false, versionDate: "2023" }])
    })
    it("selects", () => {
        const results = versionReducer({
            versions: [
                { version: 1, appliedVersion: true, versionDate: "2023" },
                { version: 2, appliedVersion: false, versionDate: "2023" }
            ]
        }, { type: VersionAction.SELECT, value: 2 })
        expect(results.selectedVersion).toEqual(2)
        expect(results.versions).toEqual([
            { version: 1, appliedVersion: true, versionDate: "2023" },
            { version: 2, appliedVersion: false, versionDate: "2023" }
        ])
    })
    it("sets applied if already applied", () => {
        const results = versionReducer({
            versions: [
                { version: 1, appliedVersion: true, versionDate: "2023" },
                { version: 2, appliedVersion: false, versionDate: "2023" }
            ]
        }, { type: VersionAction.SET_APPLIED, value: 1 })
        expect(results.versions).toEqual([
            { version: 1, appliedVersion: true, versionDate: "2023" },
            { version: 2, appliedVersion: false, versionDate: "2023" }
        ])
    })
    it("sets applied if something else selected for applied", () => {
        const results = versionReducer({
            versions: [
                { version: 1, appliedVersion: true, versionDate: "2023" },
                { version: 2, appliedVersion: false, versionDate: "2023" }
            ]
        }, { type: VersionAction.SET_APPLIED, value: 2 })
        expect(results.versions).toEqual([
            { version: 1, appliedVersion: false, versionDate: "2023" },
            { version: 2, appliedVersion: true, versionDate: "2023" }
        ])
    })
})