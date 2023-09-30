import { VersionAction, versionReducer } from './version'



describe("versionReducer", () => {
    it("adds version when no version", () => {
        const results = versionReducer({ versions: [] }, { type: VersionAction.ADD, value: "hello" })
        expect(results.versions).toEqual([{ version: "hello", appliedVersion: false }])
    })
    it("adds version when existing versions", () => {
        const results = versionReducer({ versions: [{ version: "bye", appliedVersion: true }] }, { type: VersionAction.ADD, value: "hello" })
        expect(results.versions).toEqual([{ version: "bye", appliedVersion: true }, { version: "hello", appliedVersion: false }])
    })
    it("removes version when it exists", () => {
        const results = versionReducer({ versions: [{ version: "bye", appliedVersion: true }] }, { type: VersionAction.REMOVE, value: "bye" })
        expect(results.versions).toEqual([])
    })
    it("does nothing if version does not exist", () => {
        const results = versionReducer({ versions: [{ version: "bye", appliedVersion: true }] }, { type: VersionAction.REMOVE, value: "hello" })
        expect(results.versions).toEqual([{ version: "bye", appliedVersion: true }])
    })
    it("inits", () => {
        const results = versionReducer({
            versions: [
                { version: "bye", appliedVersion: true }
            ]
        }, { type: VersionAction.INIT, value: [{ version: "hello", appliedVersion: false }] })
        expect(results.versions).toEqual([{ version: "hello", appliedVersion: false }])
    })
    it("selects", () => {
        const results = versionReducer({
            versions: [
                { version: "bye", appliedVersion: true },
                { version: "hello", appliedVersion: false }
            ]
        }, { type: VersionAction.SELECT, value: "bye" })
        expect(results.selectedVersion).toEqual("bye")
        expect(results.versions).toEqual([
            { version: "bye", appliedVersion: true },
            { version: "hello", appliedVersion: false }
        ])
    })
    it("sets applied if already applied", () => {
        const results = versionReducer({
            versions: [
                { version: "bye", appliedVersion: true },
                { version: "hello", appliedVersion: false }
            ]
        }, { type: VersionAction.SET_APPLIED, value: "bye" })
        expect(results.versions).toEqual([
            { version: "bye", appliedVersion: true },
            { version: "hello", appliedVersion: false }
        ])
    })
    it("sets applied if something else selected for applied", () => {
        const results = versionReducer({
            versions: [
                { version: "bye", appliedVersion: true },
                { version: "hello", appliedVersion: false }
            ]
        }, { type: VersionAction.SET_APPLIED, value: "hello" })
        expect(results.versions).toEqual([
            { version: "bye", appliedVersion: false },
            { version: "hello", appliedVersion: true }
        ])
    })
})