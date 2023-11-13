use std::collections::BTreeMap;

use crate::processor::Speaker;
use rocket::serde::Serialize;

pub fn split_mixer_name() -> String {
    "split_non_sub".to_string()
}
pub fn combine_mixer_name() -> String {
    "combine_sub".to_string()
}

#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
pub(crate) struct ChannelCount {
    #[serde(rename = "in")]
    pub(crate) num_in_channel: usize,
    #[serde(rename = "out")]
    pub(crate) num_out_channel: usize,
}

#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
struct Source {
    channel: usize,
    gain: i32, //this should be 0, there is a "Gain" filter https://github.com/HEnquist/camilladsp/blob/master/exampleconfigs/pulseconfig.yml#L26 this is ONLY for inputs not output
    inverted: bool, //always false in my case
}
#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
pub(crate) struct Mapping {
    sources: Vec<Source>, //inputs.  This will be used for crossover (all sources will be mapped to subwoofers)
    dest: usize,          //index of destination speaker
}
#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
pub struct Mixer {
    pub(crate) channels: ChannelCount,
    pub(crate) mapping: Vec<Mapping>,
}

const NUM_INPUT_SUBWOOFERS: usize = 1;

pub struct SpeakerCounts {
    speakers_exclude_sub: usize,
    input_subwoofers: usize,
    output_subwoofers: usize,
    pub(crate) input_subwoofer_speakers: Vec<String>,
}

pub fn get_speaker_counts(speakers: &[Speaker]) -> SpeakerCounts {
    let output_subwoofers = speakers
        .iter()
        .enumerate()
        .filter(|(_, s)| s.is_subwoofer)
        .count();

    let input_subwoofers = if output_subwoofers > 0 {
        NUM_INPUT_SUBWOOFERS
    } else {
        0
    };

    let speakers_exclude_sub = speakers.len() - output_subwoofers;

    SpeakerCounts {
        speakers_exclude_sub,
        input_subwoofers,
        output_subwoofers,
        input_subwoofer_speakers: (0..input_subwoofers)
            .map(|index| format!("subwoofer_input_{}", index))
            .collect(),
    }
}

pub fn input_speaker_count(speaker_counts: &SpeakerCounts) -> usize {
    speaker_counts.speakers_exclude_sub + speaker_counts.input_subwoofers
}

pub fn output_speaker_count_no_mixer(speaker_counts: &SpeakerCounts) -> usize {
    speaker_counts.speakers_exclude_sub + speaker_counts.output_subwoofers
}

pub fn split_inputs<'a>(
    speakers: &'a [Speaker],
    speaker_counts: &'a SpeakerCounts,
) -> Option<(
    Mixer,
    BTreeMap<&'a String, (bool, bool, usize, Vec<usize>)>,
    BTreeMap<&'a String, (usize, Vec<usize>)>,
)> {
    let SpeakerCounts {
        speakers_exclude_sub,
        output_subwoofers,
        input_subwoofer_speakers,
        ..
    } = speaker_counts;

    // what if input has a subwoofer?  Do I need to mix sub back to speakers?
    if *output_subwoofers > 0 {
        //key is speakername, then tuple of (whether it has a crossover, whether is a sub, input index, output indeces)
        let mut input_channel_mapping: BTreeMap<&String, (bool, bool, usize, Vec<usize>)> =
            BTreeMap::new();
        let mut output_channel_mapping: BTreeMap<&String, (usize, Vec<usize>)> = BTreeMap::new();

        let mut track_index = 0;

        let subs: Vec<_> = speakers
            .iter()
            .enumerate()
            .filter(|(_, v)| v.is_subwoofer)
            .collect();

        //channel_mapping.
        for (speaker_index, speaker) in speakers.iter().enumerate().filter(|(_, v)| !v.is_subwoofer)
        {
            if speaker.crossover.is_some() {
                input_channel_mapping.insert(
                    &speaker.speaker,
                    (
                        true,
                        speaker.is_subwoofer,
                        speaker_index,
                        vec![track_index, track_index + 1],
                    ),
                );

                for (sub_index, sub_name) in subs.iter() {
                    output_channel_mapping
                        .entry(&sub_name.speaker)
                        .and_modify(|(_, v)| v.push(track_index + 1))
                        .or_insert((*sub_index, vec![track_index + 1]));
                }
                output_channel_mapping.insert(&speaker.speaker, (speaker_index, vec![track_index]));
                track_index += 2;
            } else if speaker.crossover.is_none() {
                input_channel_mapping.insert(
                    &speaker.speaker,
                    (
                        false,
                        speaker.is_subwoofer,
                        speaker_index,
                        vec![track_index],
                    ),
                );
                output_channel_mapping.insert(&speaker.speaker, (speaker_index, vec![track_index]));
                track_index += 1;
            }
        }
        for (index, speaker) in input_subwoofer_speakers.iter().enumerate() {
            input_channel_mapping.insert(
                &speaker,
                (false, true, speakers_exclude_sub + index, vec![track_index]),
            );
            for (sub_index, sub_name) in subs.iter() {
                output_channel_mapping
                    .entry(&sub_name.speaker)
                    .and_modify(|(_, v)| v.push(track_index))
                    .or_insert((*sub_index, vec![track_index]));
            }
            track_index += 1;
        }

        let channels = ChannelCount {
            num_in_channel: input_speaker_count(&speaker_counts),
            num_out_channel: track_index,
        };

        let mapping: Vec<Mapping> = input_channel_mapping
            .iter()
            .map(|(_, (_, is_sub, speaker_index, channel_indeces))| {
                channel_indeces.iter().map(|channel_index| Mapping {
                    dest: *channel_index,
                    sources: vec![Source {
                        channel: *speaker_index,
                        gain: if *is_sub { 10 } else { 0 }, //bass boost needed for subwoofer channel.  careful, turning up camilladsp to MAX volume will cause issues because of this
                        inverted: false,
                    }],
                })
            })
            .flatten()
            .collect();

        return Some((
            Mixer { channels, mapping },
            input_channel_mapping,
            output_channel_mapping,
        ));
    } else {
        None
    }
}

//performed after split_inputs and crossover filters in pipeline
pub fn combine_inputs(
    speaker_counts: &SpeakerCounts,
    split_mixer: &Mixer,
    output_channel_mapping: &BTreeMap<&String, (usize, Vec<usize>)>,
) -> Mixer {
    let SpeakerCounts {
        speakers_exclude_sub,
        output_subwoofers,
        ..
    } = *speaker_counts;

    let mapping = output_channel_mapping
        .iter()
        .map(|(_, (speaker_index, channel_indeces))| Mapping {
            dest: *speaker_index,
            sources: channel_indeces
                .iter()
                .map(|channel_index| Source {
                    channel: *channel_index,
                    gain: 0,
                    inverted: false,
                })
                .collect(),
        })
        .collect();
    let channels = ChannelCount {
        num_in_channel: split_mixer.channels.num_out_channel,
        num_out_channel: speakers_exclude_sub + output_subwoofers,
    };
    Mixer { channels, mapping }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::combine_inputs;
    use super::get_speaker_counts;
    use super::split_inputs;
    use super::ChannelCount;
    use super::Mixer;
    use crate::processor::Speaker;

    #[test]
    fn test_speaker_counts_no_sub() {
        let speakers: Vec<Speaker> = vec![
            Speaker {
                speaker: "l".to_string(),
                crossover: Some(100),
                delay: 10.0,
                is_subwoofer: false,
                gain: 2.0,
            },
            Speaker {
                speaker: "r".to_string(),
                crossover: Some(100),
                delay: 10.0,
                is_subwoofer: false,
                gain: 2.0,
            },
            Speaker {
                speaker: "sl".to_string(),
                crossover: Some(100),
                delay: 10.0,
                is_subwoofer: false,
                gain: 2.0,
            },
            Speaker {
                speaker: "sr".to_string(),
                crossover: Some(100),
                delay: 10.0,
                is_subwoofer: false,
                gain: 2.0,
            },
        ];
        let result = get_speaker_counts(&speakers);
        assert!(result.speakers_exclude_sub == 4);
        assert!(result.input_subwoofers == 0);
        assert!(result.output_subwoofers == 0);
    }
    #[test]
    fn test_speaker_counts_one_sub() {
        let speakers: Vec<Speaker> = vec![
            Speaker {
                speaker: "l".to_string(),
                crossover: Some(100),
                delay: 10.0,
                is_subwoofer: false,
                gain: 2.0,
            },
            Speaker {
                speaker: "r".to_string(),
                crossover: Some(100),
                delay: 10.0,
                is_subwoofer: false,
                gain: 2.0,
            },
            Speaker {
                speaker: "sl".to_string(),
                crossover: Some(100),
                delay: 10.0,
                is_subwoofer: false,
                gain: 2.0,
            },
            Speaker {
                speaker: "sr".to_string(),
                crossover: Some(100),
                delay: 10.0,
                is_subwoofer: false,
                gain: 2.0,
            },
            Speaker {
                speaker: "sub1".to_string(),
                crossover: None,
                delay: 10.0,
                is_subwoofer: true,
                gain: 2.0,
            },
        ];
        let result = get_speaker_counts(&speakers);
        assert!(result.speakers_exclude_sub == 4);
        assert!(result.input_subwoofers == 1);
        assert!(result.output_subwoofers == 1);
    }
    #[test]
    fn test_speaker_counts_two_sub() {
        let speakers: Vec<Speaker> = vec![
            Speaker {
                speaker: "l".to_string(),
                crossover: Some(100),
                delay: 10.0,
                is_subwoofer: false,
                gain: 2.0,
            },
            Speaker {
                speaker: "r".to_string(),
                crossover: Some(100),
                delay: 10.0,
                is_subwoofer: false,
                gain: 2.0,
            },
            Speaker {
                speaker: "sl".to_string(),
                crossover: Some(100),
                delay: 10.0,
                is_subwoofer: false,
                gain: 2.0,
            },
            Speaker {
                speaker: "sr".to_string(),
                crossover: Some(100),
                delay: 10.0,
                is_subwoofer: false,
                gain: 2.0,
            },
            Speaker {
                speaker: "sub1".to_string(),
                crossover: None,
                delay: 10.0,
                is_subwoofer: true,
                gain: 2.0,
            },
            Speaker {
                speaker: "sub2".to_string(),
                crossover: None,
                delay: 10.0,
                is_subwoofer: true,
                gain: 2.0,
            },
        ];
        let result = get_speaker_counts(&speakers);
        assert!(result.speakers_exclude_sub == 4);
        assert!(result.input_subwoofers == 1);
        assert!(result.output_subwoofers == 2);
    }

    #[test]
    fn test_init_mixer_no_sub() {
        let speakers: Vec<Speaker> = vec![
            Speaker {
                speaker: "l".to_string(),
                crossover: Some(100),
                delay: 10.0,
                is_subwoofer: false,
                gain: 2.0,
            },
            Speaker {
                speaker: "r".to_string(),
                crossover: Some(100),
                delay: 10.0,
                is_subwoofer: false,
                gain: 2.0,
            },
            Speaker {
                speaker: "sl".to_string(),
                crossover: Some(100),
                delay: 10.0,
                is_subwoofer: false,
                gain: 2.0,
            },
            Speaker {
                speaker: "sr".to_string(),
                crossover: Some(100),
                delay: 10.0,
                is_subwoofer: false,
                gain: 2.0,
            },
        ];
        let speaker_counts = get_speaker_counts(&speakers);
        let result = split_inputs(&speakers, &speaker_counts);
        assert!(result.is_none());
    }
    #[test]
    fn test_init_mixer_one_sub() {
        let speakers: Vec<Speaker> = vec![
            Speaker {
                speaker: "l".to_string(),
                crossover: Some(100),
                delay: 10.0,
                is_subwoofer: false,
                gain: 2.0,
            },
            Speaker {
                speaker: "r".to_string(),
                crossover: Some(100),
                delay: 10.0,
                is_subwoofer: false,
                gain: 2.0,
            },
            Speaker {
                speaker: "sl".to_string(),
                crossover: Some(100),
                delay: 10.0,
                is_subwoofer: false,
                gain: 2.0,
            },
            Speaker {
                speaker: "sr".to_string(),
                crossover: Some(100),
                delay: 10.0,
                is_subwoofer: false,
                gain: 2.0,
            },
            Speaker {
                speaker: "sub1".to_string(),
                crossover: None,
                delay: 10.0,
                is_subwoofer: true,
                gain: 2.0,
            },
        ];
        let speaker_counts = get_speaker_counts(&speakers);
        let result = split_inputs(&speakers, &speaker_counts).unwrap();
        assert!(result.0.mapping.len() == 9);

        assert!(result.0.mapping[0].dest == 0);
        assert!(result.0.mapping[0].sources[0].channel == 0);

        assert!(result.0.mapping[1].dest == 1);
        assert!(result.0.mapping[1].sources[0].channel == 0);

        assert!(result.0.mapping[2].dest == 2);
        assert!(result.0.mapping[2].sources[0].channel == 1);

        assert!(result.0.mapping[3].dest == 3);
        assert!(result.0.mapping[3].sources[0].channel == 1);

        assert!(result.0.mapping[4].dest == 4);
        assert!(result.0.mapping[4].sources[0].channel == 2);

        assert!(result.0.mapping[5].dest == 5);
        assert!(result.0.mapping[5].sources[0].channel == 2);

        assert!(result.0.mapping[6].dest == 6);
        assert!(result.0.mapping[6].sources[0].channel == 3);

        assert!(result.0.mapping[7].dest == 7);
        assert!(result.0.mapping[7].sources[0].channel == 3);

        assert!(result.0.mapping[8].dest == 8);
        assert!(result.0.mapping[8].sources[0].channel == 4);
    }

    #[test]
    fn test_init_mixer_one_sub_passthrough() {
        let speakers: Vec<Speaker> = vec![
            Speaker {
                speaker: "l".to_string(),
                crossover: None, //no subwoofer in mix
                delay: 10.0,
                is_subwoofer: false,
                gain: 2.0,
            },
            Speaker {
                speaker: "r".to_string(),
                crossover: Some(100),
                delay: 10.0,
                is_subwoofer: false,
                gain: 2.0,
            },
            Speaker {
                speaker: "sl".to_string(),
                crossover: Some(100),
                delay: 10.0,
                is_subwoofer: false,
                gain: 2.0,
            },
            Speaker {
                speaker: "sr".to_string(),
                crossover: Some(100),
                delay: 10.0,
                is_subwoofer: false,
                gain: 2.0,
            },
            Speaker {
                speaker: "sub1".to_string(),
                crossover: None,
                delay: 10.0,
                is_subwoofer: true,
                gain: 2.0,
            },
        ];
        let speaker_counts = get_speaker_counts(&speakers);
        let result = split_inputs(&speakers, &speaker_counts).unwrap();
        assert!(result.0.mapping.len() == 8);

        assert!(result.0.mapping[0].dest == 0); //passthrough
        assert!(result.0.mapping[0].sources[0].channel == 0);

        assert!(result.0.mapping[1].dest == 1);
        assert!(result.0.mapping[1].sources[0].channel == 1);

        assert!(result.0.mapping[2].dest == 2);
        assert!(result.0.mapping[2].sources[0].channel == 1);

        assert!(result.0.mapping[3].dest == 3);
        assert!(result.0.mapping[3].sources[0].channel == 2);

        assert!(result.0.mapping[4].dest == 4);
        assert!(result.0.mapping[4].sources[0].channel == 2);

        assert!(result.0.mapping[5].dest == 5);
        assert!(result.0.mapping[5].sources[0].channel == 3);

        assert!(result.0.mapping[6].dest == 6);
        assert!(result.0.mapping[6].sources[0].channel == 3);

        assert!(result.0.mapping[7].dest == 7);
        assert!(result.0.mapping[7].sources[0].channel == 4);
    }

    #[test]
    fn test_init_mixer_one_sub_no_crossover() {
        let speakers: Vec<Speaker> = vec![
            Speaker {
                speaker: "l".to_string(),
                crossover: None, //no subwoofer in mix
                delay: 10.0,
                is_subwoofer: false,
                gain: 2.0,
            },
            Speaker {
                speaker: "r".to_string(),
                crossover: None,
                delay: 10.0,
                is_subwoofer: false,
                gain: 2.0,
            },
            Speaker {
                speaker: "sl".to_string(),
                crossover: None,
                delay: 10.0,
                is_subwoofer: false,
                gain: 2.0,
            },
            Speaker {
                speaker: "sr".to_string(),
                crossover: None,
                delay: 10.0,
                is_subwoofer: false,
                gain: 2.0,
            },
            Speaker {
                speaker: "sub1".to_string(),
                crossover: None,
                delay: 10.0,
                is_subwoofer: true,
                gain: 2.0,
            },
        ];
        let speaker_counts = get_speaker_counts(&speakers);
        let result = split_inputs(&speakers, &speaker_counts).unwrap();
        assert!(result.0.mapping.len() == 5);

        assert!(result.0.mapping[0].dest == 0); //passthrough
        assert!(result.0.mapping[0].sources[0].channel == 0);

        assert!(result.0.mapping[1].dest == 1);
        assert!(result.0.mapping[1].sources[0].channel == 1);

        assert!(result.0.mapping[2].dest == 2);
        assert!(result.0.mapping[2].sources[0].channel == 2);

        assert!(result.0.mapping[3].dest == 3);
        assert!(result.0.mapping[3].sources[0].channel == 3);

        assert!(result.0.mapping[4].dest == 4);
        assert!(result.0.mapping[4].sources[0].channel == 4);
    }
    #[test]
    fn test_init_mixer_two_sub() {
        let speakers: Vec<Speaker> = vec![
            Speaker {
                speaker: "l".to_string(),
                crossover: Some(100),
                delay: 10.0,
                is_subwoofer: false,
                gain: 2.0,
            },
            Speaker {
                speaker: "r".to_string(),
                crossover: Some(100),
                delay: 10.0,
                is_subwoofer: false,
                gain: 2.0,
            },
            Speaker {
                speaker: "sl".to_string(),
                crossover: Some(100),
                delay: 10.0,
                is_subwoofer: false,
                gain: 2.0,
            },
            Speaker {
                speaker: "sr".to_string(),
                crossover: Some(100),
                delay: 10.0,
                is_subwoofer: false,
                gain: 2.0,
            },
            Speaker {
                speaker: "sub1".to_string(),
                crossover: None,
                delay: 10.0,
                is_subwoofer: true,
                gain: 2.0,
            },
            Speaker {
                speaker: "sub2".to_string(),
                crossover: None,
                delay: 10.0,
                is_subwoofer: true,
                gain: 2.0,
            },
        ];
        let speaker_counts = get_speaker_counts(&speakers);
        let result = split_inputs(&speakers, &speaker_counts).unwrap();
        assert!(result.0.mapping.len() == 9);

        assert!(result.0.mapping[0].dest == 0);
        assert!(result.0.mapping[0].sources[0].channel == 0);

        assert!(result.0.mapping[1].dest == 1);
        assert!(result.0.mapping[1].sources[0].channel == 0);

        assert!(result.0.mapping[2].dest == 2);
        assert!(result.0.mapping[2].sources[0].channel == 1);

        assert!(result.0.mapping[3].dest == 3);
        assert!(result.0.mapping[3].sources[0].channel == 1);

        assert!(result.0.mapping[4].dest == 4);
        assert!(result.0.mapping[4].sources[0].channel == 2);

        assert!(result.0.mapping[5].dest == 5);
        assert!(result.0.mapping[5].sources[0].channel == 2);

        assert!(result.0.mapping[6].dest == 6);
        assert!(result.0.mapping[6].sources[0].channel == 3);

        assert!(result.0.mapping[7].dest == 7);
        assert!(result.0.mapping[7].sources[0].channel == 3);

        assert!(result.0.mapping[8].dest == 8);
        assert!(result.0.mapping[8].sources[0].channel == 4);
    }

    #[test]
    fn check_final_mixer_6_speakers_1_sub() {
        let speakers: Vec<Speaker> = vec![
            Speaker {
                speaker: "l".to_string(),
                crossover: Some(100),
                delay: 10.0,
                is_subwoofer: false,
                gain: 2.0,
            },
            Speaker {
                speaker: "r".to_string(),
                crossover: Some(100),
                delay: 10.0,
                is_subwoofer: false,
                gain: 2.0,
            },
            Speaker {
                speaker: "c".to_string(),
                crossover: Some(100),
                delay: 10.0,
                is_subwoofer: false,
                gain: 2.0,
            },
            Speaker {
                speaker: "sl".to_string(),
                crossover: Some(100),
                delay: 10.0,
                is_subwoofer: false,
                gain: 2.0,
            },
            Speaker {
                speaker: "sr".to_string(),
                crossover: Some(100),
                delay: 10.0,
                is_subwoofer: false,
                gain: 2.0,
            },
            Speaker {
                speaker: "sub1".to_string(),
                crossover: Some(100),
                delay: 10.0,
                is_subwoofer: true,
                gain: 2.0,
            },
        ];
        let mut output_channel_mapping: BTreeMap<&String, (usize, Vec<usize>)> = BTreeMap::new();
        let l = "l".to_string();
        let r = "r".to_string();
        let c = "c".to_string();
        let sl = "sl".to_string();
        let sr = "sr".to_string();
        let sub1 = "sub1".to_string();
        output_channel_mapping.insert(&l, (0, vec![0]));
        output_channel_mapping.insert(&r, (1, vec![2]));
        output_channel_mapping.insert(&c, (2, vec![4]));
        output_channel_mapping.insert(&sl, (3, vec![6]));
        output_channel_mapping.insert(&sr, (4, vec![8]));
        output_channel_mapping.insert(&sub1, (5, vec![1, 3, 5, 7, 9, 10]));
        let split_mixer = Mixer {
            channels: ChannelCount {
                num_in_channel: 0,
                num_out_channel: 11,
            },
            mapping: vec![],
        };
        let mix = combine_inputs(
            &get_speaker_counts(&speakers),
            &split_mixer,
            &output_channel_mapping,
            //&speakers,
        );
        assert_eq!(mix.channels.num_in_channel, 11); //2*(6-1)+1 sub passthrough
        assert_eq!(mix.channels.num_out_channel, 6); //total speaker count
        for value in mix.mapping.iter() {
            println!("destination: {}", value.dest);
            println!("sources size: {}", value.sources.len());
        }
        assert_eq!(mix.mapping.len(), 6);

        //everything should have 1 source except the subwoofer, which has size 6
        assert_eq!(
            mix.mapping.iter().filter(|v| v.sources.len() == 1).count(),
            5
        );

        assert_eq!(
            mix.mapping.iter().filter(|v| v.sources.len() == 6).count(), //6 sources including sub passthrough
            1
        );
    }
    #[test]
    fn check_final_mixer_7_speakers_2_sub() {
        let speakers: Vec<Speaker> = vec![
            Speaker {
                speaker: "l".to_string(),
                crossover: Some(100),
                delay: 10.0,
                is_subwoofer: false,
                gain: 2.0,
            },
            Speaker {
                speaker: "r".to_string(),
                crossover: Some(100),
                delay: 10.0,
                is_subwoofer: false,
                gain: 2.0,
            },
            Speaker {
                speaker: "c".to_string(),
                crossover: Some(100),
                delay: 10.0,
                is_subwoofer: false,
                gain: 2.0,
            },
            Speaker {
                speaker: "sl".to_string(),
                crossover: Some(100),
                delay: 10.0,
                is_subwoofer: false,
                gain: 2.0,
            },
            Speaker {
                speaker: "sr".to_string(),
                crossover: Some(100),
                delay: 10.0,
                is_subwoofer: false,
                gain: 2.0,
            },
            Speaker {
                speaker: "sub1".to_string(),
                crossover: None,
                delay: 10.0,
                is_subwoofer: true,
                gain: 2.0,
            },
            Speaker {
                speaker: "sub2".to_string(),
                crossover: None,
                delay: 10.0,
                is_subwoofer: true,
                gain: 2.0,
            },
        ];

        let mut output_channel_mapping: BTreeMap<&String, (usize, Vec<usize>)> = BTreeMap::new();

        let l = "l".to_string();
        let r = "r".to_string();
        let c = "c".to_string();
        let sl = "sl".to_string();
        let sr = "sr".to_string();
        let sub1 = "sub1".to_string();
        let sub2 = "sub2".to_string();
        output_channel_mapping.insert(&l, (0, vec![0]));
        output_channel_mapping.insert(&r, (1, vec![2]));
        output_channel_mapping.insert(&c, (2, vec![4]));
        output_channel_mapping.insert(&sl, (3, vec![6]));
        output_channel_mapping.insert(&sr, (4, vec![8]));
        output_channel_mapping.insert(&sub1, (5, vec![1, 3, 5, 7, 9, 10]));
        output_channel_mapping.insert(&sub2, (6, vec![1, 3, 5, 7, 9, 10]));

        let split_mixer = Mixer {
            channels: ChannelCount {
                num_in_channel: 0,
                num_out_channel: 11,
            },
            mapping: vec![],
        };
        let mix = combine_inputs(
            &get_speaker_counts(&speakers),
            &split_mixer,
            &output_channel_mapping,
            //&speakers,
        );
        assert_eq!(mix.channels.num_in_channel, 11); //2*(6-1)+1 sub input
        assert_eq!(mix.channels.num_out_channel, 7); //total speaker count

        assert_eq!(mix.mapping.len(), 7);
        //everything should have 1 source except the subwoofers, which has size 6
        assert_eq!(
            mix.mapping.iter().filter(|v| v.sources.len() == 1).count(),
            5 //5 speakers
        );

        assert_eq!(
            mix.mapping.iter().filter(|v| v.sources.len() == 6).count(), //6 sources including sub passthrough
            2                                                            //two subs
        );
    }

    #[test]
    fn check_final_mixer_5_speakers_2_sub_one_crossover() {
        let speakers: Vec<Speaker> = vec![
            Speaker {
                speaker: "l".to_string(),
                crossover: Some(100),
                delay: 10.0,
                is_subwoofer: false,
                gain: 2.0,
            },
            Speaker {
                speaker: "r".to_string(),
                crossover: None,
                delay: 10.0,
                is_subwoofer: false,
                gain: 2.0,
            },
            Speaker {
                speaker: "c".to_string(),
                crossover: None,
                delay: 10.0,
                is_subwoofer: false,
                gain: 2.0,
            },
            Speaker {
                speaker: "sub1".to_string(),
                crossover: None,
                delay: 10.0,
                is_subwoofer: true,
                gain: 2.0,
            },
            Speaker {
                speaker: "sub2".to_string(),
                crossover: None,
                delay: 10.0,
                is_subwoofer: true,
                gain: 2.0,
            },
        ];

        let mut output_channel_mapping: BTreeMap<&String, (usize, Vec<usize>)> = BTreeMap::new();
        let l = "l".to_string();
        let r = "r".to_string();
        let c = "c".to_string();
        let sub1 = "sub1".to_string();
        let sub2 = "sub2".to_string();
        output_channel_mapping.insert(&l, (0, vec![0]));
        output_channel_mapping.insert(&r, (1, vec![2]));
        output_channel_mapping.insert(&c, (2, vec![3]));
        output_channel_mapping.insert(&sub1, (3, vec![1, 4]));
        output_channel_mapping.insert(&sub2, (4, vec![1, 4]));
        let split_mixer = Mixer {
            channels: ChannelCount {
                num_in_channel: 0,
                num_out_channel: 6,
            },
            mapping: vec![],
        };
        let mix = combine_inputs(
            &get_speaker_counts(&speakers),
            &split_mixer,
            &output_channel_mapping,
            //&speakers,
        );
        assert_eq!(mix.channels.num_in_channel, 6); //2*(3-1)+1 sub input+1 passthrough
        assert_eq!(mix.channels.num_out_channel, 5); //total speaker count

        assert_eq!(mix.mapping.len(), 5);
        assert_eq!(
            mix.mapping.iter().filter(|v| v.sources.len() == 1).count(),
            3 //3 speakers
        );

        assert_eq!(
            mix.mapping.iter().filter(|v| v.sources.len() == 2).count(), //2 sources including sub passthrough
            2                                                            //two subs
        );
    }
}
