import { getSpeakerConfigurationFromSpeakers } from './speaker'

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