use crate::filters::{
    crossover_speaker_name, crossover_subwoofer_name, delay_filter_name, gain_filter_name,
    peq_filter_name, volume_filter_name,
};
use crate::processor::{Filter, Speaker};
use rocket::serde::Serialize;
use std::collections::BTreeMap;

pub fn create_per_speaker_pipeline_no_mixer(
    speakers: &[Speaker],
    peq_filters: &BTreeMap<&String, Vec<(usize, &Filter)>>,
) -> Vec<Pipeline> {
    let mut hold_speakers: BTreeMap<&String, (usize, Vec<usize>)> = BTreeMap::new();
    for (index, s) in speakers.iter().enumerate() {
        hold_speakers
            .entry(&s.speaker)
            .and_modify(|v| v.0 = index)
            .or_insert((index, vec![]));
    }
    create_per_speaker_pipeline(&hold_speakers, peq_filters)
}

pub fn create_per_speaker_pipeline(
    output_channel_mapping: &BTreeMap<&String, (usize, Vec<usize>)>,
    peq_filters: &BTreeMap<&String, Vec<(usize, &Filter)>>,
) -> Vec<Pipeline> {
    output_channel_mapping
        .iter()
        .map(|(speaker, (i, _))| {
            Pipeline::Filter(PipelineFilter {
                pipeline_type: PipelineType::Filter,
                channel: *i,
                names: peq_filters
                    .get(speaker)
                    .unwrap_or(&vec![])
                    .iter()
                    .map(|(index, _)| peq_filter_name(&speaker, *index))
                    .chain(std::iter::once(delay_filter_name(&speaker)))
                    .chain(std::iter::once(gain_filter_name(&speaker)))
                    .chain(std::iter::once(volume_filter_name()))
                    .collect(),
            })
        })
        .collect()
}

/** */

pub fn create_crossover_pipeline(
    split_mixer_name: String,
    combine_mixer_name: String,
    input_channel_mapping: &BTreeMap<&String, (bool, bool, usize, Vec<usize>)>,
) -> Vec<Pipeline> {
    std::iter::once(Pipeline::Mixer(PipelineMixer {
        pipeline_type: PipelineType::Mixer,
        name: split_mixer_name,
    }))
    .chain(
        input_channel_mapping
            .iter()
            .filter(|(_, (is_crossover, _, _, _))| *is_crossover)
            .map(|(key, (_, _, _, channel_indeces))| {
                vec![
                    Pipeline::Filter(PipelineFilter {
                        pipeline_type: PipelineType::Filter,
                        channel: channel_indeces[0],
                        names: vec![crossover_speaker_name(&key)],
                    }),
                    Pipeline::Filter(PipelineFilter {
                        pipeline_type: PipelineType::Filter,
                        channel: channel_indeces[1],
                        names: vec![crossover_subwoofer_name(&key)],
                    }),
                ]
                .into_iter()
            })
            .flatten(),
    )
    .chain(std::iter::once(Pipeline::Mixer(PipelineMixer {
        pipeline_type: PipelineType::Mixer,
        name: combine_mixer_name,
    })))
    .collect()
}

#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
enum PipelineType {
    Filter,
    Mixer,
}

#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
pub struct PipelineFilter {
    #[serde(rename = "type")]
    pipeline_type: PipelineType,
    channel: usize,
    pub(crate) names: Vec<String>, //these are keys in the Filter hashmap
}

#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
pub struct PipelineMixer {
    #[serde(rename = "type")]
    pipeline_type: PipelineType,
    name: String,
}

#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
#[serde(untagged)]
pub enum Pipeline {
    Filter(PipelineFilter),
    Mixer(PipelineMixer),
}

#[cfg(test)]
mod tests {
    use super::{create_crossover_pipeline, create_per_speaker_pipeline, Pipeline};
    use crate::filters::compute_peq_filter;
    use crate::pipeline::create_per_speaker_pipeline_no_mixer;
    use crate::processor::{Filter, Speaker};
    use std::collections::BTreeMap;
    #[test]
    fn check_create_pipeline() {
        let mut input_channel_mapping: BTreeMap<&String, (bool, bool, usize, Vec<usize>)> =
            BTreeMap::new();
        let l = "l".to_string();
        let r = "r".to_string();
        let c = "c".to_string();
        let sub1 = "sub1".to_string();
        input_channel_mapping.insert(&l, (true, false, 0, vec![0, 1]));
        input_channel_mapping.insert(&r, (true, false, 1, vec![2, 3]));
        input_channel_mapping.insert(&c, (true, false, 2, vec![4, 5]));
        input_channel_mapping.insert(&sub1, (false, true, 3, vec![6]));
        let result = create_crossover_pipeline(
            "myinitmixer".to_string(),
            "myfinalmixer".to_string(),
            &input_channel_mapping,
        );
        assert!(result.len() == 8); //2*(4-1) for cxfilters+2 for mixer
    }
    #[test]
    fn check_create_per_speaker_pipeline() {
        let mut output_channel_mapping: BTreeMap<&String, (usize, Vec<usize>)> = BTreeMap::new();
        let l = "l".to_string();
        let r = "r".to_string();
        let c = "c".to_string();
        let sub1: String = "sub1".to_string();
        output_channel_mapping.insert(&l, (0, vec![0, 1]));
        output_channel_mapping.insert(&r, (1, vec![2, 3]));
        output_channel_mapping.insert(&c, (2, vec![4, 5]));
        output_channel_mapping.insert(&sub1, (3, vec![6]));
        let filters = vec![
            Filter {
                freq: 1000,
                gain: 2.0,
                q: 0.707,
                speaker: "l".to_string(),
            },
            Filter {
                freq: 2000,
                gain: 2.0,
                q: 0.707,
                speaker: "l".to_string(),
            },
            Filter {
                freq: 2000,
                gain: 1.0,
                q: 0.707,
                speaker: "r".to_string(),
            },
        ];
        let result =
            create_per_speaker_pipeline(&output_channel_mapping, &compute_peq_filter(&filters));
        assert!(result.len() == 4);
        match &result[1] {
            //left, keys are alphabetized
            Pipeline::Filter(f) => {
                assert!(f.names.len() == 5); //2 peq, 1 gain, 1 delay, 1 volume
            }
            Pipeline::Mixer(_) => {
                assert!(false); //should not get here
            }
        }
        match &result[0] {
            //center, keys are alphabetized
            Pipeline::Filter(f) => {
                assert!(f.names.len() == 3); //1 gain, 1 delay, 1 volume
            }
            Pipeline::Mixer(_) => {
                assert!(false); //should not get here
            }
        }
        match &result[2] {
            //right, keys are alphabetized
            Pipeline::Filter(f) => {
                assert!(f.names.len() == 4); //1peq, 1 gain, 1 delay, 1 volume
            }
            Pipeline::Mixer(_) => {
                assert!(false); //should not get here
            }
        }
        match &result[3] {
            //sub, keys are alphabetized
            Pipeline::Filter(f) => {
                assert!(f.names.len() == 3); //1 gain, 1 delay, 1 volume
            }
            Pipeline::Mixer(_) => {
                assert!(false); //should not get here
            }
        }
    }
    #[test]
    fn check_create_per_speaker_pipeline_no_mixer() {
        let speakers = vec![
            Speaker {
                speaker: "l".to_string(),
                crossover: Some(80),
                delay: 10.0,
                gain: 1.0,
                is_subwoofer: false,
            },
            Speaker {
                speaker: "c".to_string(),
                crossover: Some(80),
                delay: 10.0,
                gain: 1.0,
                is_subwoofer: false,
            },
            Speaker {
                speaker: "r".to_string(),
                crossover: Some(80),
                delay: 10.0,
                gain: 1.0,
                is_subwoofer: false,
            },
            Speaker {
                speaker: "sub1".to_string(),
                crossover: Some(80),
                delay: 10.0,
                gain: 1.0,
                is_subwoofer: true,
            },
        ];
        let filters = vec![
            Filter {
                freq: 1000,
                gain: 2.0,
                q: 0.707,
                speaker: "l".to_string(),
            },
            Filter {
                freq: 2000,
                gain: 2.0,
                q: 0.707,
                speaker: "l".to_string(),
            },
            Filter {
                freq: 2000,
                gain: 1.0,
                q: 0.707,
                speaker: "r".to_string(),
            },
        ];
        let result = create_per_speaker_pipeline_no_mixer(&speakers, &compute_peq_filter(&filters));
        assert!(result.len() == 4);
        match &result[1] {
            //left, keys are alphabetized
            Pipeline::Filter(f) => {
                assert!(f.names.len() == 5); //2 peq, 1 gain, 1 delay, 1 volume
            }
            Pipeline::Mixer(_) => {
                assert!(false); //should not get here
            }
        }
        match &result[0] {
            //center, keys are alphabetized
            Pipeline::Filter(f) => {
                assert!(f.names.len() == 3); //1 gain, 1 delay, 1 volume
            }
            Pipeline::Mixer(_) => {
                assert!(false); //should not get here
            }
        }
        match &result[2] {
            //right, keys are alphabetized
            Pipeline::Filter(f) => {
                assert!(f.names.len() == 4); //1peq, 1 gain, 1 delay, 1 volume
            }
            Pipeline::Mixer(_) => {
                assert!(false); //should not get here
            }
        }
        match &result[3] {
            //sub, keys are alphabetized
            Pipeline::Filter(f) => {
                assert!(f.names.len() == 3); //1 gain, 1 delay, 1 volume
            }
            Pipeline::Mixer(_) => {
                assert!(false); //should not get here
            }
        }
    }
}
