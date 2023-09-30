import { createContext, PropsWithChildren, useReducer, useContext } from "react"

type BaseSpeaker = {
    speaker: string
    crossover: number | null
    gain: number
    isSubwoofer: boolean
}
export interface Speaker extends BaseSpeaker {
    delay: number
}

//this needs to be converted to Speaker, delay_slash_distance will hold the distance in meters or feet, or the ms delay
export interface SpeakerForm extends BaseSpeaker {
    delay: number | null
    distanceInMeters: number | null
    distanceInFeet: number | null
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

const initialState: State = {
    speakerConfiguration: SPEAKER_OPTIONS[0].label,
    speakers: [],
}


type State = {
    speakerConfiguration: string,
    speakers: Speaker[]
}
export enum SpeakerAction {
    UPDATE,
    INIT,
    SET,
    CONFIG
}

interface ActionInterface {
    type: SpeakerAction;
}

interface SpeakerActionInterface extends ActionInterface {
    value: Speaker;
}

interface ConfigurationActionInterface extends ActionInterface {
    value: string;
}
interface SpeakersActionInterface extends ActionInterface {
    value: Speaker[];
}

type Action = SpeakersActionInterface | ConfigurationActionInterface | SpeakerActionInterface;

const SpeakerContext = createContext({
    state: initialState,
    dispatch: (_: Action) => { }
})

//export for testing
export const setSpeakerBase = (speakers: Speaker[], speakerConfiguration: string) => {
    const baseSpeakers = SPEAKER_OPTIONS.find(s => s.label === speakerConfiguration)
    return baseSpeakers ? baseSpeakers.speakers.map(baseSpeaker => {
        const existingSpeaker = speakers.find(s => s.speaker === baseSpeaker.speaker)
        return existingSpeaker || { ...DEFAULT_SPEAKER_SETTINGS, ...baseSpeaker }
    }) : undefined
}
//exported for testing
export const getSpeakerConfigurationFromSpeakers = (speakers: Speaker[]) => {
    const { numSpeak, numSub } = speakers.reduce((agg, curr) => ({
        numSpeak: agg.numSpeak + (curr.isSubwoofer ? 0 : 1), numSub: agg.numSub + (curr.isSubwoofer ? 1 : 0)
    }), { numSpeak: 0, numSub: 0 })
    return `${numSpeak}.${numSub}`
}

export function speakerReducer(state: State, action: Action): State {
    switch (action.type) {
        case SpeakerAction.CONFIG:
            return { ...state, speakerConfiguration: action.value as string }

        case SpeakerAction.UPDATE:
            const speaker = action.value as Speaker
            return {
                ...state,
                speakers: state.speakers.map(v => v.speaker === speaker.speaker ? speaker : v)
            }
        case SpeakerAction.INIT:
            const speakerConfiguration = action.value as string
            return {
                ...state,
                speakers: setSpeakerBase(state.speakers, speakerConfiguration) || state.speakers
            }
        case SpeakerAction.SET:
            const speakers = action.value as Speaker[]
            return {
                speakers: speakers,
                speakerConfiguration: getSpeakerConfigurationFromSpeakers(speakers)
            }
        default:
            return state
    }
}

export const SpeakerProvider = ({ children }: PropsWithChildren) => {
    const [state, dispatch] = useReducer(speakerReducer, initialState);

    return (
        <SpeakerContext.Provider value={{ state, dispatch }}>
            {children}
        </SpeakerContext.Provider>
    );
};

export const useSpeaker = () => {
    const context = useContext(SpeakerContext);
    if (!context) {
        throw new Error("useSpeaker must be used within a SpeakerProvider");
    }
    return context;
}
