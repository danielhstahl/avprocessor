use crate::filters::{
    crossover_speaker_name, crossover_subwoofer_name, delay_filter_name, gain_filter_name,
    peq_filter_name,
};
use crate::mixers::CrossoverChannels;
use crate::processor::{Filter, Speaker};
use rocket::serde::Serialize;
use std::collections::BTreeMap;

pub fn create_per_speaker_pipeline(
    speakers: &[Speaker],
    peq_filters: &BTreeMap<&String, Vec<(usize, &Filter)>>,
) -> Vec<Pipeline> {
    speakers
        .iter()
        .enumerate()
        .map(|(i, s)| {
            Pipeline::Filter(PipelineFilter {
                pipeline_type: PipelineType::Filter,
                channel: i,
                names: peq_filters
                    .get(&s.speaker)
                    .unwrap_or(&vec![])
                    .iter()
                    .map(|(index, _)| peq_filter_name(&s.speaker, *index))
                    .chain(std::iter::once(delay_filter_name(&s.speaker)))
                    .chain(std::iter::once(gain_filter_name(&s.speaker)))
                    .collect(),
            })
        })
        .collect()
}

pub fn create_crossover_pipeline(
    split_mixer_name: String,
    combine_mixer_name: String,
    crossover_channels: &CrossoverChannels,
    speakers: &[Speaker],
) -> Vec<Pipeline> {
    std::iter::once(Pipeline::Mixer(PipelineMixer {
        pipeline_type: PipelineType::Mixer,
        name: split_mixer_name,
    }))
    .chain(
        crossover_channels
            .speaker_channels
            .iter()
            .zip(speakers.iter().filter(|v| !v.is_subwoofer))
            .map(|(source_index, s)| {
                Pipeline::Filter(PipelineFilter {
                    pipeline_type: PipelineType::Filter,
                    channel: *source_index,
                    names: vec![crossover_speaker_name(&s.speaker)],
                })
            }),
    )
    .chain(
        crossover_channels
            .subwoofer_channels
            .iter()
            .zip(speakers.iter().filter(|v| !v.is_subwoofer))
            .map(|(source_index, s)| {
                Pipeline::Filter(PipelineFilter {
                    pipeline_type: PipelineType::Filter,
                    channel: *source_index,
                    names: vec![crossover_subwoofer_name(&s.speaker)],
                })
            }),
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
pub(crate) struct PipelineFilter {
    #[serde(rename = "type")]
    pipeline_type: PipelineType,
    channel: usize,
    pub(crate) names: Vec<String>, //these are keys in the Filter hashmap
}

#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
pub(crate) struct PipelineMixer {
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
    use crate::mixers::CrossoverChannels;
    use crate::processor::{Filter, Speaker};
    #[test]
    fn check_create_pipeline() {
        let speakers = vec![
            Speaker {
                speaker: "l".to_string(),
                crossover: Some(80),
                delay: 10,
                gain: 1.0,
                is_subwoofer: false,
            },
            Speaker {
                speaker: "c".to_string(),
                crossover: Some(80),
                delay: 10,
                gain: 1.0,
                is_subwoofer: false,
            },
            Speaker {
                speaker: "r".to_string(),
                crossover: Some(80),
                delay: 10,
                gain: 1.0,
                is_subwoofer: false,
            },
            Speaker {
                speaker: "sub1".to_string(),
                crossover: Some(80),
                delay: 10,
                gain: 1.0,
                is_subwoofer: true,
            },
        ];
        let result = create_crossover_pipeline(
            "myinitmixer".to_string(),
            "myfinalmixer".to_string(),
            &CrossoverChannels {
                speaker_channels: vec![0, 1, 2],
                subwoofer_channels: vec![3, 4, 5],
                passthrough_channels: vec![],
            },
            &speakers,
        );
        assert!(result.len() == 8); //2*(4-1) for cxfilters+2 for mixer
    }
    #[test]
    fn check_create_per_speaker_pipeline() {
        let speakers = vec![
            Speaker {
                speaker: "l".to_string(),
                crossover: Some(80),
                delay: 10,
                gain: 1.0,
                is_subwoofer: false,
            },
            Speaker {
                speaker: "c".to_string(),
                crossover: Some(80),
                delay: 10,
                gain: 1.0,
                is_subwoofer: false,
            },
            Speaker {
                speaker: "r".to_string(),
                crossover: Some(80),
                delay: 10,
                gain: 1.0,
                is_subwoofer: false,
            },
            Speaker {
                speaker: "sub1".to_string(),
                crossover: Some(80),
                delay: 10,
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
        let result = create_per_speaker_pipeline(&speakers, &compute_peq_filter(&filters));
        assert!(result.len() == 4);
        match &result[0] {
            Pipeline::Filter(f) => {
                assert!(f.names.len() == 4); //2 peq, 1 gain, 1 delay
            }
            Pipeline::Mixer(_) => {
                assert!(false); //should not get here
            }
        }
        match &result[1] {
            Pipeline::Filter(f) => {
                assert!(f.names.len() == 2); //1 gain, 1 delay
            }
            Pipeline::Mixer(_) => {
                assert!(false); //should not get here
            }
        }
        match &result[2] {
            Pipeline::Filter(f) => {
                assert!(f.names.len() == 3); //1peq, 1 gain, 1 delay
            }
            Pipeline::Mixer(_) => {
                assert!(false); //should not get here
            }
        }
        match &result[3] {
            Pipeline::Filter(f) => {
                assert!(f.names.len() == 2); //1 gain, 1 delay
            }
            Pipeline::Mixer(_) => {
                assert!(false); //should not get here
            }
        }
    }
}
