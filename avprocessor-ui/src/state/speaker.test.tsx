import { DelayType } from './delay'
import { SpeakerAction, getSpeakerConfigurationFromSpeakers, setSpeakerBase, speakerReducer } from './speaker'


describe("getSpeakerConfigurationFromSpeakers", () => {
    it("returns 4.1 when given 4 speakers and a sub", () => {
        expect(getSpeakerConfigurationFromSpeakers([{
            speaker: "sp1",
            isSubwoofer: false,
            crossover: 100,
            delay: 4,
            gain: 2
        },
        {
            speaker: "sp2",
            isSubwoofer: false,
            crossover: 100,
            delay: 4,
            gain: 2
        },
        {
            speaker: "sp3",
            isSubwoofer: false,
            crossover: 100,
            delay: 4,
            gain: 2
        },
        {
            speaker: "sp4",
            isSubwoofer: false,
            crossover: 100,
            delay: 4,
            gain: 2
        },
        {
            speaker: "sp5",
            isSubwoofer: true,
            crossover: 100,
            delay: 4,
            gain: 2
        }])).toEqual("4.1")
    })
    it("returns 2.3 when given 2 speakers and 3 sub", () => {
        expect(getSpeakerConfigurationFromSpeakers([{
            speaker: "sp1",
            isSubwoofer: false,
            crossover: 100,
            delay: 4,
            gain: 2
        },
        {
            speaker: "sp2",
            isSubwoofer: false,
            crossover: 100,
            delay: 4,
            gain: 2
        },
        {
            speaker: "sp3",
            isSubwoofer: true,
            crossover: 100,
            delay: 4,
            gain: 2
        },
        {
            speaker: "sp4",
            isSubwoofer: true,
            crossover: 100,
            delay: 4,
            gain: 2
        },
        {
            speaker: "sp5",
            isSubwoofer: true,
            crossover: 100,
            delay: 4,
            gain: 2
        }])).toEqual("2.3")
    })
})

describe("setSpeakerBase", () => {
    it("updates where exists and adds where doesn't", () => {
        const speakers = [{
            speaker: "Left",
            isSubwoofer: false,
            crossover: 100,
            delay: 4,
            gain: 2
        },
        {
            speaker: "Right",
            isSubwoofer: false,
            crossover: 150,
            delay: 3,
            gain: 1
        }]
        const result = setSpeakerBase(speakers, "3.2")
        expect(result).toEqual([{
            speaker: "Left",
            isSubwoofer: false,
            crossover: 100,
            delay: 4,
            gain: 2
        },
        {
            speaker: "Center",
            isSubwoofer: false,
            crossover: null,
            delay: 0,
            gain: 0
        },
        {
            speaker: "Right",
            isSubwoofer: false,
            crossover: 150,
            delay: 3,
            gain: 1
        },
        {
            speaker: "Subwoofer 1",
            isSubwoofer: true,
            crossover: null,
            delay: 0,
            gain: 0
        },
        {
            speaker: "Subwoofer 2",
            isSubwoofer: true,
            crossover: null,
            delay: 0,
            gain: 0
        }])
    })
})

describe("speakerReducer", () => {
    it("sets config", () => {
        const results = speakerReducer({
            speakers: [],
            speakerConfiguration: ""
        }, { type: SpeakerAction.CONFIG, value: "3.2" })
        expect(results.speakerConfiguration).toEqual("3.2")
        expect(results.speakers).toEqual([])
    })
    it("updates speaker", () => {
        const results = speakerReducer({
            speakers: [{
                speaker: "Subwoofer 2",
                isSubwoofer: true,
                crossover: null,
                delay: 0,
                gain: 0
            }],
            speakerConfiguration: ""
        }, {
            type: SpeakerAction.UPDATE, value: {
                speaker: "Subwoofer 2",
                isSubwoofer: true,
                crossover: null,
                delay: 4,
                gain: 2
            }
        })
        expect(results.speakerConfiguration).toEqual("")
        expect(results.speakers).toEqual([{
            speaker: "Subwoofer 2",
            isSubwoofer: true,
            crossover: null,
            delay: 4,
            gain: 2
        }])
    })
    it("inits", () => {
        const results = speakerReducer({
            speakers: [{
                speaker: "Subwoofer 2",
                isSubwoofer: true,
                crossover: null,
                delay: 4,
                gain: 2
            }],
            speakerConfiguration: ""
        }, {
            type: SpeakerAction.INIT, value: "3.2"
        })
        expect(results.speakerConfiguration).toEqual("")//we don't actually set this at init
        expect(results.speakers).toEqual([{
            speaker: "Left",
            isSubwoofer: false,
            crossover: null,
            delay: 0,
            gain: 0
        },
        {
            speaker: "Center",
            isSubwoofer: false,
            crossover: null,
            delay: 0,
            gain: 0
        },
        {
            speaker: "Right",
            isSubwoofer: false,
            crossover: null,
            delay: 0,
            gain: 0
        },
        {
            speaker: "Subwoofer 1",
            isSubwoofer: true,
            crossover: null,
            delay: 0,
            gain: 0
        },
        {
            speaker: "Subwoofer 2",
            isSubwoofer: true,
            crossover: null,
            delay: 4,
            gain: 2
        }])
    })
    it("returns existing if speaker configuration is invalid", () => {
        const results = speakerReducer({
            speakers: [{
                speaker: "Subwoofer 2",
                isSubwoofer: true,
                crossover: null,
                delay: 0,
                gain: 0
            }],
            speakerConfiguration: ""
        }, {
            type: SpeakerAction.INIT, value: "notvalid"
        })
        expect(results.speakerConfiguration).toEqual("")//we don't actually set this at init
        expect(results.speakers).toEqual([{
            speaker: "Subwoofer 2",
            isSubwoofer: true,
            crossover: null,
            delay: 0,
            gain: 0
        }])
    })
    it("sets everything if provided speakers", () => {
        const results = speakerReducer({
            speakers: [{
                speaker: "Subwoofer 2",
                isSubwoofer: true,
                crossover: null,
                delay: 3,
                gain: 2
            }],
            speakerConfiguration: ""
        }, {
            type: SpeakerAction.SET, value: [{
                speaker: "Left",
                isSubwoofer: false,
                crossover: 100,
                delay: 4,
                gain: 2
            },
            {
                speaker: "Center",
                isSubwoofer: false,
                crossover: null,
                delay: 0,
                gain: 0
            },
            {
                speaker: "Right",
                isSubwoofer: false,
                crossover: 150,
                delay: 3,
                gain: 1
            },
            {
                speaker: "Subwoofer 1",
                isSubwoofer: true,
                crossover: null,
                delay: 0,
                gain: 0
            },
            {
                speaker: "Subwoofer 2",
                isSubwoofer: true,
                crossover: null,
                delay: 0,
                gain: 0
            }]
        })
        expect(results.speakerConfiguration).toEqual("3.2")//derived from speakers
        //note that subwoofer 2 is overwritten, speakers are completely reset
        expect(results.speakers).toEqual([{
            speaker: "Left",
            isSubwoofer: false,
            crossover: 100,
            delay: 4,
            gain: 2
        },
        {
            speaker: "Center",
            isSubwoofer: false,
            crossover: null,
            delay: 0,
            gain: 0
        },
        {
            speaker: "Right",
            isSubwoofer: false,
            crossover: 150,
            delay: 3,
            gain: 1
        },
        {
            speaker: "Subwoofer 1",
            isSubwoofer: true,
            crossover: null,
            delay: 0,
            gain: 0
        },
        {
            speaker: "Subwoofer 2",
            isSubwoofer: true,
            crossover: null,
            delay: 0,
            gain: 0
        }])
    })
    it("sets delays with feet", () => {
        const results = speakerReducer({
            speakers: [{
                speaker: "Left",
                isSubwoofer: false,
                crossover: null,
                delay: 3,
                gain: 2
            }],
            speakerConfiguration: ""
        }, {
            type: SpeakerAction.UPDATE_DELAY, value: {
                speaker: {
                    speaker: "Left",
                    isSubwoofer: false,
                    crossover: 100,
                    delay: 4,
                    gain: 2
                },
                delayType: DelayType.FEET,
                delayValue: 3.0
            }
        })
        expect(results.speakers).toEqual([{
            speaker: "Left",
            isSubwoofer: false,
            crossover: 100,
            delay: 0.0, //zero'd out
            gain: 2,
            distanceInMeters: 3.3,
            distanceInFeet: 3.0
        }])
    })
    it("sets delays with ms", () => {
        const results = speakerReducer({
            speakers: [{
                speaker: "Left",
                isSubwoofer: false,
                crossover: null,
                delay: 3,
                gain: 2,
                distanceInMeters: 4,
                distanceInFeet: 5
            }],
            speakerConfiguration: ""
        }, {
            type: SpeakerAction.UPDATE_DELAY, value: {
                speaker: {
                    speaker: "Left",
                    isSubwoofer: false,
                    crossover: 100,
                    delay: 4,
                    gain: 2
                },
                delayType: DelayType.MS,
                delayValue: 3.0
            }
        })
        //note that subwoofer 2 is overwritten, speakers are completely reset
        expect(results.speakers).toEqual([{
            speaker: "Left",
            isSubwoofer: false,
            crossover: 100,
            delay: 4, //updated
            gain: 2,
            distanceInMeters: undefined, //note that original is overwritten
            distanceInFeet: undefined
        }])
    })
    it("sets delays with meters", () => {
        const results = speakerReducer({
            speakers: [{
                speaker: "Left",
                isSubwoofer: false,
                crossover: null,
                delay: 3,
                gain: 2,
                distanceInMeters: 4,
                distanceInFeet: 5
            }],
            speakerConfiguration: ""
        }, {
            type: SpeakerAction.UPDATE_DELAY, value: {
                speaker: {
                    speaker: "Left",
                    isSubwoofer: false,
                    crossover: 100,
                    delay: 4,
                    gain: 2
                },
                delayType: DelayType.METERS,
                delayValue: 3.0
            }
        })
        expect(results.speakers).toEqual([{
            speaker: "Left",
            isSubwoofer: false,
            crossover: 100,
            delay: 0, //zero'd out
            gain: 2,
            distanceInMeters: 3.0,
            distanceInFeet: 5.0
        }])
    })
})