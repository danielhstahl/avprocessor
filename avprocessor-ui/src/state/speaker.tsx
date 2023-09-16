import React, { useState, PropsWithChildren } from "react"

export type Speaker = {
    speaker: string
    crossover: number | null
    delay: number
    gain: number
    isSubwoofer: boolean
}

type SpeakerOption = { label: string, speakers: { speaker: string, isSubwoofer: boolean }[] }[]
export const SPEAKER_OPTIONS: SpeakerOption = [
    {
        label: "2.0",
        speakers: [
            {
                speaker: "Left",
                isSubwoofer: false
            },
            {
                speaker: "Right",
                isSubwoofer: false
            }
        ]
    },
    {
        label: "2.1",
        speakers: [
            {
                speaker: "Left",
                isSubwoofer: false
            },
            {
                speaker: "Right",
                isSubwoofer: false
            },
            {
                speaker: "Subwoofer",
                isSubwoofer: true
            }
        ]
    },
    {
        label: "2.2",
        speakers: [
            {
                speaker: "Left",
                isSubwoofer: false
            },
            {
                speaker: "Right",
                isSubwoofer: false
            },
            {
                speaker: "Subwoofer 1",
                isSubwoofer: true
            },
            {
                speaker: "Subwoofer 2",
                isSubwoofer: true
            }
        ]
    },
    {
        label: "3.1",
        speakers: [
            {
                speaker: "Left",
                isSubwoofer: false
            },
            {
                speaker: "Center",
                isSubwoofer: false
            },
            {
                speaker: "Right",
                isSubwoofer: false
            },
            {
                speaker: "Subwoofer",
                isSubwoofer: true
            }
        ]
    },
    {
        label: "3.2",
        speakers: [
            {
                speaker: "Left",
                isSubwoofer: false
            },
            {
                speaker: "Center",
                isSubwoofer: false
            },
            {
                speaker: "Right",
                isSubwoofer: false
            },
            {
                speaker: "Subwoofer 1",
                isSubwoofer: true
            },
            {
                speaker: "Subwoofer 2",
                isSubwoofer: true
            }
        ]
    },
    {
        label: "4.0",
        speakers: [
            {
                speaker: "Left",
                isSubwoofer: false
            },

            {
                speaker: "Right",
                isSubwoofer: false
            },
            {
                speaker: "Surround Left",
                isSubwoofer: false
            },
            {
                speaker: "Surround Right",
                isSubwoofer: false
            }
        ]
    },
    {
        label: "4.1",
        speakers: [
            {
                speaker: "Left",
                isSubwoofer: false
            },

            {
                speaker: "Right",
                isSubwoofer: false
            },
            {
                speaker: "Surround Left",
                isSubwoofer: false
            },
            {
                speaker: "Surround Right",
                isSubwoofer: false
            },
            {
                speaker: "Subwoofer",
                isSubwoofer: true
            }
        ]
    },
    {
        label: "4.2",
        speakers: [
            {
                speaker: "Left",
                isSubwoofer: false
            },
            {
                speaker: "Right",
                isSubwoofer: false
            },
            {
                speaker: "Surround Left",
                isSubwoofer: false
            },
            {
                speaker: "Surround Right",
                isSubwoofer: false
            },
            {
                speaker: "Subwoofer 1",
                isSubwoofer: true
            },
            {
                speaker: "Subwoofer 2",
                isSubwoofer: true
            }
        ]
    },
    {
        label: "5.1",
        speakers: [
            {
                speaker: "Left",
                isSubwoofer: false
            },
            {
                speaker: "Center",
                isSubwoofer: false
            },
            {
                speaker: "Right",
                isSubwoofer: false
            },
            {
                speaker: "Surround Left",
                isSubwoofer: false
            },
            {
                speaker: "Surround Right",
                isSubwoofer: false
            },
            {
                speaker: "Subwoofer",
                isSubwoofer: true
            }
        ]
    },
    {
        label: "5.2",
        speakers: [
            {
                speaker: "Left",
                isSubwoofer: false
            },
            {
                speaker: "Center",
                isSubwoofer: false
            },
            {
                speaker: "Right",
                isSubwoofer: false
            },
            {
                speaker: "Surround Left",
                isSubwoofer: false
            },
            {
                speaker: "Surround Right",
                isSubwoofer: false
            },
            {
                speaker: "Subwoofer 1",
                isSubwoofer: true
            },
            {
                speaker: "Subwoofer 2",
                isSubwoofer: true
            }
        ]
    },
    {
        label: "7.1",
        speakers: [
            {
                speaker: "Left",
                isSubwoofer: false
            },
            {
                speaker: "Center",
                isSubwoofer: false
            },
            {
                speaker: "Right",
                isSubwoofer: false
            },
            {
                speaker: "Surround Left",
                isSubwoofer: false
            },
            {
                speaker: "Surround Right",
                isSubwoofer: false
            },
            {
                speaker: "Surround Left Back",
                isSubwoofer: false
            },
            {
                speaker: "Surround Right Back",
                isSubwoofer: false
            },
            {
                speaker: "Subwoofer",
                isSubwoofer: true
            }
        ]
    }
]
const DEFAULT_SPEAKER_SETTINGS = { delay: 0, gain: 0, crossover: null }

const initSpeakers: Speaker[] = []
const initSpeakerConfiguration = SPEAKER_OPTIONS[0].label
const speakerContext = {
    speakerConfiguration: initSpeakerConfiguration,
    speakers: initSpeakers,
    updateSpeaker: (update: Speaker) => { },
    setSpeakerBase: (update: string) => { },
    setSpeakers: (update: Speaker[]) => { },
    setSpeakerConfiguration: (update: string) => { },
}

export const SpeakerContext = React.createContext(speakerContext)
const getSpeakerConfigurationFromSpeakers = (speakers: Speaker[]) => {
    const { numSpeak, numSub } = speakers.reduce((agg, curr) => ({
        numSpeak: agg.numSpeak + (curr.isSubwoofer ? 0 : 1), numSub: agg.numSub + (curr.isSubwoofer ? 1 : 0)
    }), { numSpeak: 0, numSub: 0 })
    return `${numSpeak}.${numSub}`
}
export const SpeakerProviderComponent = ({ children }: PropsWithChildren) => {
    const setSpeakers = (speakers: Speaker[]) => setContext((currentContext) => ({
        ...currentContext,
        speakers: speakers,
        speakerConfiguration: getSpeakerConfigurationFromSpeakers(speakers)
    }))

    const updateSpeaker = (speaker: Speaker) => setContext((currentContext) => ({
        ...currentContext,
        speakers: currentContext.speakers.map(v => v.speaker === speaker.speaker ? speaker : v),
    }))

    //keep existing settings for speakers when possible when changing speaker base
    const setSpeakerBase = (speakerConfiguration: string) => setContext((currentContext) => {
        const baseSpeakers = SPEAKER_OPTIONS.find(s => s.label === speakerConfiguration)
        return baseSpeakers ? {
            ...currentContext,
            speakers: baseSpeakers.speakers.map(baseSpeaker => {
                const existingSpeaker = currentContext.speakers.find(s => s.speaker === baseSpeaker.speaker)
                return existingSpeaker || { ...DEFAULT_SPEAKER_SETTINGS, ...baseSpeaker }
            }),
            speakerConfiguration
        } : currentContext
    })

    const setSpeakerConfiguration = (contextUpdates: string) => setContext((currentContext) => ({
        ...currentContext,
        speakerConfiguration: contextUpdates
    }))



    const initState = {
        speakers: initSpeakers,
        speakerConfiguration: initSpeakerConfiguration,
        setSpeakerBase,
        setSpeakers,
        setSpeakerConfiguration,
        updateSpeaker
    }



    const [context, setContext] = useState(initState)

    return (
        <SpeakerContext.Provider value={context}>
            {children}
        </SpeakerContext.Provider>
    )
}

