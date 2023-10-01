import { createContext, PropsWithChildren, useReducer, useContext } from "react"
import { DelayType } from "./delay"

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
    delay?: number
    distanceInMeters?: number
    distanceInFeet?: number
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
const DEFAULT_SPEAKER_SETTINGS = { delay: 0, distanceInMeters: 0, distanceInFeet: 0, gain: 0, crossover: null }

const initialState: State = {
    speakerConfiguration: SPEAKER_OPTIONS[0].label,
    speakers: [],
}

const FEET_TO_METER_RATIO = 0.3048
const convertFeetToMeters = (feet: number) => feet * FEET_TO_METER_RATIO
const convertMetersToFeet = (meters: number) => meters / FEET_TO_METER_RATIO



type State = {
    speakerConfiguration: string,
    speakers: SpeakerForm[]
}
export enum SpeakerAction {
    UPDATE,
    INIT,
    SET,
    CONFIG,
    UPDATE_DELAY
}
export type SpeakerDelay = {
    speaker: SpeakerForm,
    delayType: DelayType,
    delayValue: number
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
    value: SpeakerForm[];
}
interface SpeakerDelayActionInterface extends ActionInterface {
    value: SpeakerDelay
}

type Action = SpeakersActionInterface | ConfigurationActionInterface | SpeakerActionInterface | SpeakerDelayActionInterface;

const SpeakerContext = createContext({
    state: initialState,
    dispatch: (_: Action) => { }
})

const handleDelay = (delayType: DelayType, delayValue: number) => {
    switch (delayType) {
        case DelayType.FEET:
            return {
                delay: 0.0, //set to zero since does not matter; will be updated when applied
                distanceInFeet: delayValue,
                distanceInMeters: convertFeetToMeters(delayValue)
            }

        case DelayType.METERS:
            return {
                delay: 0.0, //set to zero since does not matter; will be updated when applied
                distanceInFeet: convertMetersToFeet(delayValue),
                distanceInMeters: delayValue
            }

        case DelayType.MS:
            return {
                delay: delayValue,
                distanceInFeet: undefined,  //explicit in case already defined in speaker settings
                distanceInMeters: undefined //explicit in case already defined in speaker settings
            }
        default:
            return {
                delay: delayValue
            }
    }
}

//export for testing
export const setSpeakerBase = (speakers: SpeakerForm[], speakerConfiguration: string) => {
    const baseSpeakers = SPEAKER_OPTIONS.find(s => s.label === speakerConfiguration)
    return baseSpeakers ? baseSpeakers.speakers.map(baseSpeaker => {
        const existingSpeaker = speakers.find(s => s.speaker === baseSpeaker.speaker)
        return existingSpeaker || { ...DEFAULT_SPEAKER_SETTINGS, ...baseSpeaker }
    }) : undefined
}
//exported for testing
export const getSpeakerConfigurationFromSpeakers = (speakers: SpeakerForm[]) => {
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
            const speaker = action.value as SpeakerForm
            return {
                ...state,
                speakers: state.speakers.map(v => v.speaker === speaker.speaker ? speaker : v)
            }
        case SpeakerAction.UPDATE_DELAY:
            const { speaker: speakerDelay, delayType, delayValue } = action.value as SpeakerDelay
            return {
                ...state,
                speakers: state.speakers.map(v => v.speaker === speakerDelay.speaker ? {
                    ...v,
                    ...handleDelay(delayType, delayValue)
                } : v)
            }
        case SpeakerAction.INIT:
            const speakerConfiguration = action.value as string
            return {
                ...state,
                speakers: setSpeakerBase(state.speakers, speakerConfiguration) || state.speakers
            }
        case SpeakerAction.SET:
            const speakers = action.value as SpeakerForm[]
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
