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
struct ChannelCount {
    #[serde(rename = "in")]
    num_in_channel: usize,
    #[serde(rename = "out")]
    num_out_channel: usize,
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
struct Mapping {
    sources: Vec<Source>, //inputs.  This will be used for crossover (all sources will be mapped to subwoofers)
    dest: usize,          //index of destination speaker
}
#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
pub struct Mixer {
    channels: ChannelCount,
    mapping: Vec<Mapping>,
}

const NUM_INPUT_SUBWOOFERS: usize = 1;

pub struct SpeakerCounts {
    speakers_exclude_sub: usize,
    input_subwoofers: usize,
    output_subwoofers: usize,
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
    }
}

pub struct CrossoverChannels {
    pub speaker_channels: Vec<usize>,
    pub subwoofer_channels: Vec<usize>, //these will be the same size; split each main speaker into subwoofer channels
}

pub fn split_inputs(speaker_counts: &SpeakerCounts) -> Option<(Mixer, CrossoverChannels)> {
    let SpeakerCounts {
        speakers_exclude_sub,
        input_subwoofers,
        output_subwoofers,
    } = *speaker_counts;

    if output_subwoofers > 0 {
        let channels = ChannelCount {
            num_in_channel: speakers_exclude_sub + input_subwoofers,
            num_out_channel: speakers_exclude_sub * 2 + input_subwoofers, //split each input speaker to go to subwoofer channel
        };

        let speaker_channels: Vec<usize> =
            (0..speakers_exclude_sub).map(|index| index * 2).collect();

        let subwoofer_channels: Vec<usize> = (0..speakers_exclude_sub)
            .map(|index| index * 2 + 1)
            .collect();

        let mapping = speaker_channels
            .iter()
            .enumerate()
            .zip(subwoofer_channels.iter())
            .map(|((index, speaker_index), subwoofer_index)| {
                vec![
                    Mapping {
                        dest: *speaker_index,
                        sources: vec![Source {
                            channel: index,
                            gain: 0,
                            inverted: false,
                        }],
                    },
                    Mapping {
                        dest: *subwoofer_index,
                        sources: vec![Source {
                            channel: index,
                            gain: 0,
                            inverted: false,
                        }],
                    },
                ]
                .into_iter()
            })
            .flatten()
            .chain(
                (speakers_exclude_sub..(speakers_exclude_sub + input_subwoofers)).map(|index| {
                    Mapping {
                        dest: speakers_exclude_sub + index,
                        sources: vec![Source {
                            channel: index,
                            gain: 0,
                            inverted: false,
                        }],
                    }
                }),
            )
            .collect();
        return Some((
            Mixer { channels, mapping },
            CrossoverChannels {
                speaker_channels,
                subwoofer_channels,
            },
        ));
    } else {
        None
    }
}

//performed after split_inputs and crossover filters in pipeline
pub fn combine_inputs(
    speaker_counts: &SpeakerCounts,
    crossover_channels: &CrossoverChannels,
    speakers: &[Speaker],
) -> Mixer {
    let SpeakerCounts {
        speakers_exclude_sub,
        input_subwoofers,
        output_subwoofers,
    } = *speaker_counts;

    let channels = ChannelCount {
        num_in_channel: speakers_exclude_sub * 2 + input_subwoofers, //split_input.channels.num_out_channel,
        num_out_channel: speakers_exclude_sub + output_subwoofers, //split each input speaker to go to subwoofer channel
    };

    let mapping = speakers
        .iter()
        .enumerate()
        .filter(|(_, v)| v.is_subwoofer)
        .map(|(i, _)| Mapping {
            dest: i,
            sources: crossover_channels
                .subwoofer_channels
                .iter()
                .map(|index| Source {
                    channel: *index,
                    gain: 0,
                    inverted: false,
                })
                .chain(std::iter::once(Source {
                    channel: channels.num_in_channel - 1, //source is last index of previous mixer
                    gain: 0,
                    inverted: false,
                }))
                .collect(),
        })
        .chain(
            crossover_channels
                .speaker_channels
                .iter()
                .zip(
                    speakers
                        .iter()
                        .enumerate()
                        .filter(|(_, v)| !v.is_subwoofer)
                        .map(|(i, _)| i),
                )
                .map(|(source_index, dest_index)| Mapping {
                    dest: dest_index,
                    sources: vec![Source {
                        channel: *source_index,
                        gain: 0,
                        inverted: false,
                    }],
                }),
        )
        .collect();

    Mixer { channels, mapping }
}

#[cfg(test)]
mod tests {
    use super::combine_inputs;
    use super::get_speaker_counts;
    use super::split_inputs;
    use super::CrossoverChannels;
    use crate::processor::Speaker;

    #[test]
    fn test_speaker_counts_no_sub() {
        let speakers: Vec<Speaker> = vec![
            Speaker {
                speaker: "l".to_string(),
                crossover: Some(100),
                delay: 10,
                is_subwoofer: false,
                gain: 2.0,
            },
            Speaker {
                speaker: "r".to_string(),
                crossover: Some(100),
                delay: 10,
                is_subwoofer: false,
                gain: 2.0,
            },
            Speaker {
                speaker: "sl".to_string(),
                crossover: Some(100),
                delay: 10,
                is_subwoofer: false,
                gain: 2.0,
            },
            Speaker {
                speaker: "sr".to_string(),
                crossover: Some(100),
                delay: 10,
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
                delay: 10,
                is_subwoofer: false,
                gain: 2.0,
            },
            Speaker {
                speaker: "r".to_string(),
                crossover: Some(100),
                delay: 10,
                is_subwoofer: false,
                gain: 2.0,
            },
            Speaker {
                speaker: "sl".to_string(),
                crossover: Some(100),
                delay: 10,
                is_subwoofer: false,
                gain: 2.0,
            },
            Speaker {
                speaker: "sr".to_string(),
                crossover: Some(100),
                delay: 10,
                is_subwoofer: false,
                gain: 2.0,
            },
            Speaker {
                speaker: "sub1".to_string(),
                crossover: None,
                delay: 10,
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
                delay: 10,
                is_subwoofer: false,
                gain: 2.0,
            },
            Speaker {
                speaker: "r".to_string(),
                crossover: Some(100),
                delay: 10,
                is_subwoofer: false,
                gain: 2.0,
            },
            Speaker {
                speaker: "sl".to_string(),
                crossover: Some(100),
                delay: 10,
                is_subwoofer: false,
                gain: 2.0,
            },
            Speaker {
                speaker: "sr".to_string(),
                crossover: Some(100),
                delay: 10,
                is_subwoofer: false,
                gain: 2.0,
            },
            Speaker {
                speaker: "sub1".to_string(),
                crossover: None,
                delay: 10,
                is_subwoofer: true,
                gain: 2.0,
            },
            Speaker {
                speaker: "sub2".to_string(),
                crossover: None,
                delay: 10,
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
                delay: 10,
                is_subwoofer: false,
                gain: 2.0,
            },
            Speaker {
                speaker: "r".to_string(),
                crossover: Some(100),
                delay: 10,
                is_subwoofer: false,
                gain: 2.0,
            },
            Speaker {
                speaker: "sl".to_string(),
                crossover: Some(100),
                delay: 10,
                is_subwoofer: false,
                gain: 2.0,
            },
            Speaker {
                speaker: "sr".to_string(),
                crossover: Some(100),
                delay: 10,
                is_subwoofer: false,
                gain: 2.0,
            },
        ];
        let result = split_inputs(&get_speaker_counts(&speakers));
        assert!(result.is_none());
    }
    #[test]
    fn test_init_mixer_one_sub() {
        let speakers: Vec<Speaker> = vec![
            Speaker {
                speaker: "l".to_string(),
                crossover: Some(100),
                delay: 10,
                is_subwoofer: false,
                gain: 2.0,
            },
            Speaker {
                speaker: "r".to_string(),
                crossover: Some(100),
                delay: 10,
                is_subwoofer: false,
                gain: 2.0,
            },
            Speaker {
                speaker: "sl".to_string(),
                crossover: Some(100),
                delay: 10,
                is_subwoofer: false,
                gain: 2.0,
            },
            Speaker {
                speaker: "sr".to_string(),
                crossover: Some(100),
                delay: 10,
                is_subwoofer: false,
                gain: 2.0,
            },
            Speaker {
                speaker: "sub1".to_string(),
                crossover: None,
                delay: 10,
                is_subwoofer: true,
                gain: 2.0,
            },
        ];
        let result = split_inputs(&get_speaker_counts(&speakers)).unwrap();
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
    fn test_init_mixer_two_sub() {
        let speakers: Vec<Speaker> = vec![
            Speaker {
                speaker: "l".to_string(),
                crossover: Some(100),
                delay: 10,
                is_subwoofer: false,
                gain: 2.0,
            },
            Speaker {
                speaker: "r".to_string(),
                crossover: Some(100),
                delay: 10,
                is_subwoofer: false,
                gain: 2.0,
            },
            Speaker {
                speaker: "sl".to_string(),
                crossover: Some(100),
                delay: 10,
                is_subwoofer: false,
                gain: 2.0,
            },
            Speaker {
                speaker: "sr".to_string(),
                crossover: Some(100),
                delay: 10,
                is_subwoofer: false,
                gain: 2.0,
            },
            Speaker {
                speaker: "sub1".to_string(),
                crossover: None,
                delay: 10,
                is_subwoofer: true,
                gain: 2.0,
            },
            Speaker {
                speaker: "sub2".to_string(),
                crossover: None,
                delay: 10,
                is_subwoofer: true,
                gain: 2.0,
            },
        ];
        let result = split_inputs(&get_speaker_counts(&speakers)).unwrap();
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
                delay: 10,
                is_subwoofer: false,
                gain: 2.0,
            },
            Speaker {
                speaker: "r".to_string(),
                crossover: Some(100),
                delay: 10,
                is_subwoofer: false,
                gain: 2.0,
            },
            Speaker {
                speaker: "c".to_string(),
                crossover: Some(100),
                delay: 10,
                is_subwoofer: false,
                gain: 2.0,
            },
            Speaker {
                speaker: "sl".to_string(),
                crossover: Some(100),
                delay: 10,
                is_subwoofer: false,
                gain: 2.0,
            },
            Speaker {
                speaker: "sr".to_string(),
                crossover: Some(100),
                delay: 10,
                is_subwoofer: false,
                gain: 2.0,
            },
            Speaker {
                speaker: "sub1".to_string(),
                crossover: Some(100),
                delay: 10,
                is_subwoofer: true,
                gain: 2.0,
            },
        ];
        let mix = combine_inputs(
            &get_speaker_counts(&speakers),
            &CrossoverChannels {
                speaker_channels: vec![0, 2, 4, 6, 8],
                subwoofer_channels: vec![1, 3, 5, 7, 9],
            },
            &speakers,
        );
        assert!(mix.channels.num_in_channel == 11); //2*(6-1)+1 sub passthrough
        assert!(mix.channels.num_out_channel == 6); //total speaker count

        assert!(mix.mapping.len() == 6);
        assert!(mix.mapping[0].dest == 5); //last channel is sub channel
        assert!(mix.mapping[0].sources.len() == 6); //6 speakers feeding subwoofer inluding sub passthrough

        assert!(mix.mapping[1].dest == 0);
        assert!(mix.mapping[1].sources.len() == 1);
        assert!(mix.mapping[1].sources[0].channel == 0);

        assert!(mix.mapping[2].dest == 1);
        assert!(mix.mapping[2].sources.len() == 1);
        assert!(mix.mapping[2].sources[0].channel == 2);

        assert!(mix.mapping[3].dest == 2);
        assert!(mix.mapping[3].sources.len() == 1);
        assert!(mix.mapping[3].sources[0].channel == 4);

        assert!(mix.mapping[4].dest == 3);
        assert!(mix.mapping[4].sources.len() == 1);
        assert!(mix.mapping[4].sources[0].channel == 6);

        assert!(mix.mapping[5].dest == 4);
        assert!(mix.mapping[5].sources.len() == 1);
        assert!(mix.mapping[5].sources[0].channel == 8);
    }
    #[test]
    fn check_final_mixer_7_speakers_2_sub() {
        let speakers: Vec<Speaker> = vec![
            Speaker {
                speaker: "l".to_string(),
                crossover: Some(100),
                delay: 10,
                is_subwoofer: false,
                gain: 2.0,
            },
            Speaker {
                speaker: "r".to_string(),
                crossover: Some(100),
                delay: 10,
                is_subwoofer: false,
                gain: 2.0,
            },
            Speaker {
                speaker: "c".to_string(),
                crossover: Some(100),
                delay: 10,
                is_subwoofer: false,
                gain: 2.0,
            },
            Speaker {
                speaker: "sl".to_string(),
                crossover: Some(100),
                delay: 10,
                is_subwoofer: false,
                gain: 2.0,
            },
            Speaker {
                speaker: "sr".to_string(),
                crossover: Some(100),
                delay: 10,
                is_subwoofer: false,
                gain: 2.0,
            },
            Speaker {
                speaker: "sub1".to_string(),
                crossover: None,
                delay: 10,
                is_subwoofer: true,
                gain: 2.0,
            },
            Speaker {
                speaker: "sub2".to_string(),
                crossover: None,
                delay: 10,
                is_subwoofer: true,
                gain: 2.0,
            },
        ];

        let mix = combine_inputs(
            &get_speaker_counts(&speakers),
            &CrossoverChannels {
                speaker_channels: vec![0, 2, 4, 6, 8],
                subwoofer_channels: vec![1, 3, 5, 7, 9],
            },
            &speakers,
        );
        assert!(mix.channels.num_in_channel == 11); //2*(6-1)+1 sub input
        assert!(mix.channels.num_out_channel == 7); //total speaker count

        assert!(mix.mapping.len() == 7);
        assert!(mix.mapping[0].dest == 5); //last two channels are sub channel, though this isn't strictly necessary
        assert!(mix.mapping[0].sources.len() == 6); //6 input speakers feeding subwoofer including sub passthrough

        assert!(mix.mapping[1].dest == 6); //last two channels are sub channel
        assert!(mix.mapping[1].sources.len() == 6); //6 input speakers feeding subwoofer including sub passthrough

        assert!(mix.mapping[2].dest == 0);
        assert!(mix.mapping[2].sources.len() == 1);
        assert!(mix.mapping[2].sources[0].channel == 0);

        assert!(mix.mapping[3].dest == 1);
        assert!(mix.mapping[3].sources.len() == 1);
        assert!(mix.mapping[3].sources[0].channel == 2);

        assert!(mix.mapping[4].dest == 2);
        assert!(mix.mapping[4].sources.len() == 1);
        assert!(mix.mapping[4].sources[0].channel == 4);

        assert!(mix.mapping[5].dest == 3);
        assert!(mix.mapping[5].sources.len() == 1);
        assert!(mix.mapping[5].sources[0].channel == 6);

        assert!(mix.mapping[6].dest == 4);
        assert!(mix.mapping[6].sources.len() == 1);
        assert!(mix.mapping[6].sources[0].channel == 8);
    }
}
