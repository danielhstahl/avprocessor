
import { FilterAction, filterReducer, setFilterBase, perSpeakerFilters } from './filter'

describe("setFiltersPure", () => {
    it("filters correctly", () => {
        const results = perSpeakerFilters([
            {
                speaker: "speaker1",
                q: 0.1,
                freq: 300,
                gain: 3
            },
            {
                speaker: "speaker1",
                q: 0.1,
                freq: 800,
                gain: 3
            },
            {
                speaker: "speaker2",
                q: 0.1,
                freq: 800,
                gain: 3
            }
        ])
        expect(results).toEqual({
            speaker1: [
                {
                    speaker: "speaker1",
                    q: 0.1,
                    freq: 300,
                    gain: 3,
                    index: 1
                },
                {
                    speaker: "speaker1",
                    q: 0.1,
                    freq: 800,
                    gain: 3,
                    index: 2
                }],
            speaker2: [
                {
                    speaker: "speaker2",
                    q: 0.1,
                    freq: 800,
                    gain: 3,
                    index: 1
                }
            ]
        })
    })
})

describe("setfilterBase", () => {

    it("correctly sets filters", async () => {
        const filters = perSpeakerFilters([
            {
                speaker: "Left",
                q: 0.1,
                freq: 300,
                gain: 3
            },
            {
                speaker: "Right",
                q: 0.1,
                freq: 800,
                gain: 3
            },
            {
                speaker: "Right",
                q: 0.1,
                freq: 800,
                gain: 3
            }
        ])
        const results = setFilterBase("3.2", filters)

        expect(results).toEqual({
            Center: [],
            Left: [
                {
                    speaker: "Left",
                    q: 0.1,
                    freq: 300,
                    gain: 3,
                    index: 1
                }]
            ,
            Right: [{
                speaker: "Right",
                q: 0.1,
                freq: 800,
                gain: 3,
                index: 1
            },
            {
                speaker: "Right",
                q: 0.1,
                freq: 800,
                gain: 3,
                index: 2
            }],
            "Subwoofer 1": [],
            "Subwoofer 2": [],
        })
    })
    it("returns undefined if label doesnt exist", async () => {
        const filters = perSpeakerFilters([
            {
                speaker: "Left",
                q: 0.1,
                freq: 300,
                gain: 3
            },
            {
                speaker: "Right",
                q: 0.1,
                freq: 800,
                gain: 3
            },
            {
                speaker: "Right",
                q: 0.1,
                freq: 800,
                gain: 3
            }
        ])
        const results = setFilterBase("nonexistentlabel", filters)

        expect(results).toBeUndefined()
    })

})

describe("filterReducer", () => {
    it("correctly updates", () => {
        const results = filterReducer({
            filters: {
                "Subwoofer 2": [{
                    speaker: "Subwoofer 2",
                    q: 0.0,
                    freq: 0,
                    gain: 0,
                    index: 1
                }]
            }
        }, {
            type: FilterAction.UPDATE, value: {
                speaker: "Subwoofer 2",
                q: 1.0,
                freq: 100,
                gain: 0,
                index: 1
            }
        })
        expect(results.filters).toEqual({
            "Subwoofer 2": [{
                speaker: "Subwoofer 2",
                q: 1.0,
                freq: 100,
                gain: 0,
                index: 1
            }]
        })
    })
    it("correctly adds", () => {
        const results = filterReducer({
            filters: {
                "Subwoofer 2": [{
                    speaker: "Subwoofer 2",
                    q: 1.0,
                    freq: 100,
                    gain: 0,
                    index: 1
                }]
            }
        }, {
            type: FilterAction.ADD, value: "Subwoofer 2"
        })
        expect(results.filters).toEqual({
            "Subwoofer 2": [{
                speaker: "Subwoofer 2",
                q: 1.0,
                freq: 100,
                gain: 0,
                index: 1
            },
            {
                speaker: "Subwoofer 2",
                q: 0.0,
                freq: 100,
                gain: 0,
                index: 2
            }]
        })
    })
    it("correctly inits with real speakerConfiguration", () => {
        const results = filterReducer({
            filters: {}
        }, {
            type: FilterAction.INIT, value: "3.2"
        })
        expect(results.filters).toEqual({
            "Center": [],
            "Left": [],
            "Right": [],
            "Subwoofer 1": [],
            "Subwoofer 2": []
        })
    })
    it("correctly returns existing when not speakerConfiguration", () => {
        const results = filterReducer({
            filters: {}
        }, {
            type: FilterAction.INIT, value: "not real"
        })
        expect(results.filters).toEqual({})
    })
    it("correctly removes when filter exist", () => {
        const results = filterReducer({
            filters: {
                "Subwoofer 2": [{
                    speaker: "Subwoofer 2",
                    q: 1.0,
                    freq: 100,
                    gain: 0,
                    index: 1
                }]
            }
        }, {
            type: FilterAction.REMOVE,
            value: {
                speaker: "Subwoofer 2",
                q: 1.0,
                freq: 100,
                gain: 0,
                index: 1
            }
        })
        expect(results.filters).toEqual({
            "Subwoofer 2": []
        })
    })
    it("does nothing when trying to move filter that does not exist", () => {
        const results = filterReducer({
            filters: {
                "Subwoofer 2": [{
                    speaker: "Subwoofer 2",
                    q: 1.0,
                    freq: 100,
                    gain: 0,
                    index: 1
                }]
            }
        }, {
            type: FilterAction.REMOVE, value: {
                speaker: "Subwoofer 3",
                q: 1.0,
                freq: 100,
                gain: 0,
                index: 1
            }
        })
        expect(results.filters).toEqual({
            "Subwoofer 2": [{
                speaker: "Subwoofer 2",
                q: 1.0,
                freq: 100,
                gain: 0,
                index: 1
            }]
        })
    })

    it("sets filters", () => {
        const results = filterReducer({
            filters: {
                "Subwoofer 2": [{
                    speaker: "Subwoofer 2",
                    q: 1.0,
                    freq: 100,
                    gain: 0,
                    index: 1
                }]
            }
        }, {
            type: FilterAction.SET, value: [{
                speaker: "speaker1",
                q: 0.1,
                freq: 300,
                gain: 3
            },
            {
                speaker: "speaker1",
                q: 0.1,
                freq: 800,
                gain: 3
            },
            {
                speaker: "speaker2",
                q: 0.1,
                freq: 800,
                gain: 3
            }]
        })
        expect(results.filters).toEqual({
            speaker1: [
                {
                    speaker: "speaker1",
                    q: 0.1,
                    freq: 300,
                    gain: 3,
                    index: 1
                },
                {
                    speaker: "speaker1",
                    q: 0.1,
                    freq: 800,
                    gain: 3,
                    index: 2
                }],
            speaker2: [
                {
                    speaker: "speaker2",
                    q: 0.1,
                    freq: 800,
                    gain: 3,
                    index: 1
                }
            ]
        })
    })

})
