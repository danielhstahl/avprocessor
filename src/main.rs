#[macro_use]
extern crate rocket;
use core::num;
use rocket::response::status::{BadRequest, NotFound};
use rocket::serde::{json, json::Json, Deserialize, Serialize};
use rocket::Error;
use rocket_db_pools::sqlx::{self, Row};
use rocket_db_pools::{Connection, Database};
use std::collections::HashMap;

#[derive(Database)]
#[database("settings")]
struct Settings(sqlx::SqlitePool);

#[derive(Deserialize, Serialize)]
#[serde(crate = "rocket::serde")]
#[derive(sqlx::FromRow)]
struct Filter {
    freq: i32,
    gain: i32,
    q: f32,
    speaker: String,
}

#[derive(Deserialize, Serialize)]
#[serde(crate = "rocket::serde")]
#[derive(sqlx::FromRow)]
struct Speaker {
    speaker: String,
    crossover: i32,
    delay: i32,
    gain: i32,
    is_subwoofer: bool,
}
struct Input {
    is_subwoofer: bool,
}
#[derive(Deserialize, Serialize)]
#[serde(crate = "rocket::serde")]
struct ProcessorSettings {
    filters: Vec<Filter>,
    speakers: Vec<Speaker>,
}

/**Mixers */
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
    gain: i32,      //should this be zero always?  Where to set per-speaker gain
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
struct Mixer {
    channels: ChannelCount,
    mapping: Vec<Mapping>,
}

const NUM_INPUT_SUBWOOFERS: usize = 1;

//assumption is input is x.1, with the .1 coming at the end.
// So 5.1 has 5 "normal" speakers (indeces 1 through 4) and 1 subwoofer with index 5.
impl Mixer {
    fn init(speakers: &[Speaker]) -> Self {
        let num_output_subwoofers = speakers
            .iter()
            .enumerate()
            .filter(|(_, s)| s.is_subwoofer)
            .map(|(i, _)| i)
            .count();
        let num_speakers: usize = speakers.len();
        let num_non_sub_channels = num_speakers - num_output_subwoofers;

        //if no output subwoofers, don't worry about input subwoofers
        let num_input_channels = num_non_sub_channels
            + if num_output_subwoofers > 0 {
                NUM_INPUT_SUBWOOFERS
            } else {
                0
            };

        Self {
            channels: ChannelCount {
                num_in_channel: num_input_channels,
                num_out_channel: num_speakers,
            },
            mapping: speakers
                .iter()
                .enumerate()
                .filter(|(i, v)| !v.is_subwoofer) //(0..num_non_sub_channels)
                .map(|(i, v)| Mapping {
                    //input to output mapping
                    dest: i,
                    sources: vec![Source {
                        channel: i,
                        gain: v.gain,
                        inverted: false,
                    }],
                })
                .chain(
                    speakers
                        .iter()
                        .enumerate()
                        .filter(|(i, v)| v.is_subwoofer) //(0..num_non_sub_channels)
                        .map(|(i, v)| Mapping {
                            //input to output mapping
                            dest: i,
                            sources: vec![Source {
                                channel: num_non_sub_channels, //only one input sub channel
                                gain: v.gain,
                                inverted: false,
                            }],
                        }),
                )
                .chain(
                    //crossover
                    speakers
                        .iter()
                        .enumerate()
                        .filter(|(i, v)| v.is_subwoofer) //(0..num_non_sub_channels)
                        .map(|(sub_index, sub)| Mapping {
                            //input to output mapping
                            dest: sub_index,
                            sources: speakers
                                .iter()
                                .enumerate()
                                .filter(|(speaker_index, v)| !v.is_subwoofer)
                                .map(|(speaker_index, _)| Source {
                                    channel: speaker_index,
                                    gain: sub.gain,
                                    inverted: false,
                                })
                                .collect(),
                        }),
                )
                .collect(),
        }
    }
}

/**End Mixers */

/**Filters */

#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
enum FilterType {
    //PEAKING,
    Delay,
    Biquad, //both peaking and highpass are BIQUAD filters
    BiquadCombo,
}

#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
enum DelayUnit {
    //more may be added later
    MS,
}

#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
enum CrossoverType {
    //more may be added later
    ButterworthHighpass,
}

#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
enum PeakingType {
    //more may be added later
    Peaking,
}

#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
struct PeakingParameters {
    freq: i32,
    q: f32,
    gain: i32,
    #[serde(rename = "type")]
    peaking_type: PeakingType,
}

#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
struct CrossoverParameters {
    freq: i32,
    order: i32, //4 is 24db/oct, 2 is 12db/oct
    #[serde(rename = "type")]
    crossover_type: CrossoverType,
}

#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
struct DelayParameters {
    delay: i32,
    unit: DelayUnit,
}

#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
struct PeakingFilter {
    #[serde(rename = "type")]
    filter_type: FilterType,
    parameters: PeakingParameters,
}

#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
struct DelayFilter {
    #[serde(rename = "type")]
    filter_type: FilterType,
    parameters: DelayParameters,
}

#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
struct CrossoverFilter {
    #[serde(rename = "type")]
    filter_type: FilterType,
    parameters: CrossoverParameters,
}

#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
enum SpeakerAdjust {
    DelayFilter(DelayFilter),
    PeakingFilter(PeakingFilter),
    CrossoverFilter(CrossoverFilter),
}

/**End Filters */

/**begin pipelin */

#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
enum PipelineType {
    Filter,
    Mixer,
}

#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
struct PipelineFilter {
    #[serde(rename = "type")]
    pipeline_type: PipelineType,
    channel: usize,
    names: Vec<String>, //these are keys in the Filter hashmap
}

#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
struct PipelineMixer {
    #[serde(rename = "type")]
    pipeline_type: PipelineType,
    name: String,
}

#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
enum Pipeline {
    Filter(PipelineFilter),
    Mixer(PipelineMixer),
}

fn delay_filter_name(speaker_name: &str) -> String {
    format!("delay_{}", speaker_name)
}
fn peq_filter_name(speaker_name: &str, peq_index: usize) -> String {
    format!("peq_{}_{}", speaker_name, peq_index)
}
fn crossover_filter_name(speaker_name: &str) -> String {
    format!("crossover_{}", speaker_name)
}
fn subwoofer_mixer_name() -> String {
    "subwoofer".to_string()
}

fn create_pipeline(speakers: &[Speaker], filters: &[Filter]) -> Vec<Pipeline> {
    let mut hold_filters: HashMap<&String, Vec<(usize, &Filter)>> = HashMap::new();
    for (index, filter) in filters.iter().enumerate() {
        hold_filters
            .entry(&filter.speaker)
            .and_modify(|v| v.push((index, filter)))
            .or_insert(vec![(index, filter)]);
    }

    speakers
        .iter()
        .enumerate()
        .map(|(i, s)| {
            Pipeline::Filter(PipelineFilter {
                pipeline_type: PipelineType::Filter,
                channel: i,
                names: vec![delay_filter_name(&s.speaker)],
            })
        })
        .chain(
            speakers
                .iter()
                .enumerate()
                .filter(|(_, s)| !s.is_subwoofer)
                .map(|(i, s)| {
                    Pipeline::Filter(PipelineFilter {
                        pipeline_type: PipelineType::Filter,
                        channel: i,
                        names: vec![crossover_filter_name(&s.speaker)],
                    })
                }),
        )
        .chain(
            vec![Pipeline::Mixer(PipelineMixer {
                pipeline_type: PipelineType::Mixer,
                name: subwoofer_mixer_name(),
            })]
            .into_iter(),
        )
        .chain(
            hold_filters.iter().map(|(key, v)| {
                Pipeline::Filter(PipelineFilter {
                    pipeline_type: PipelineType::Filter,
                    channel: speakers.iter().position(|s| &s.speaker == *key).unwrap(),
                    names: v
                        .iter()
                        .map(|(i, f)| peq_filter_name(&f.speaker, *i))
                        .collect(),
                })
            }), /*filters
                .iter()
                .enumerate()
                .group_by(|(i, f)| f.speaker)
                .map::<String, (usize, Iterator<Filter>)>(|key, group| {
                    Pipeline::Filter(PipelineFilter {
                        pipeline_type: PipelineType::Filter,
                        channel: speakers.iter().position(|s| s.speaker == key).unwrap(),
                        names: group.map(|(i, f)| peq_filter_name(&f.speaker, i)).collect(),
                    })
                }),*/ /*filters.iter().enumerate().map(|(i, f)| {
                    Pipeline::Filter(PipelineFilter {
                        pipeline_type: PipelineType::Filter,
                        channel: speakers
                            .iter()
                            .position(|s| s.speaker == f.speaker)
                            .unwrap(),
                        names: vec![peq_filter_name(&f.speaker, i)],
                    })*/
        )
        .collect()
}

/**end pipeline */

#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
struct CamillaConfig {
    mixers: HashMap<String, Mixer>,
    filters: HashMap<String, SpeakerAdjust>,
    pipeline: Vec<Pipeline>,
}

fn convert_processor_settings_to_camilla(settings: &ProcessorSettings) -> String {
    let mixers: HashMap<String, Mixer> =
        HashMap::from([(subwoofer_mixer_name(), Mixer::init(&settings.speakers))]);

    let filters: HashMap<String, SpeakerAdjust> = HashMap::from_iter(
        settings
            .filters
            .iter()
            .enumerate()
            .map(|(i, f)| {
                (
                    peq_filter_name(&f.speaker, i),
                    SpeakerAdjust::PeakingFilter(PeakingFilter {
                        filter_type: FilterType::Biquad,
                        parameters: PeakingParameters {
                            freq: f.freq,
                            q: f.q,
                            gain: f.gain,
                            peaking_type: PeakingType::Peaking,
                        },
                    }),
                )
            })
            .chain(settings.speakers.iter().map(|f| {
                (
                    delay_filter_name(&f.speaker),
                    SpeakerAdjust::DelayFilter(DelayFilter {
                        filter_type: FilterType::Delay,
                        parameters: DelayParameters {
                            delay: f.delay,
                            unit: DelayUnit::MS,
                        },
                    }),
                )
            }))
            .chain(
                settings
                    .speakers
                    .iter()
                    .filter(|s| !s.is_subwoofer)
                    .map(|f| {
                        (
                            crossover_filter_name(&f.speaker),
                            SpeakerAdjust::CrossoverFilter(CrossoverFilter {
                                filter_type: FilterType::BiquadCombo,
                                parameters: CrossoverParameters {
                                    freq: f.crossover,
                                    crossover_type: CrossoverType::ButterworthHighpass,
                                    order: 4, //24 db/octave.  TODO make this variable?
                                },
                            }),
                        )
                    }),
            ),
    );

    let pipeline = create_pipeline(&settings.speakers, &settings.filters);
    let result = CamillaConfig {
        pipeline,
        filters,
        mixers,
    };
    json::to_string(&result).unwrap()
}

#[get("/config/latest")]
async fn config_latest(
    mut db: Connection<Settings>,
) -> Result<Json<ProcessorSettings>, BadRequest<String>> {
    let filters=sqlx::query_as::<_, Filter>("SELECT speaker, freq, gain, q from filters where version=(select max(version) as mxversion FROM versions)")
        .fetch_all(&mut *db)
        .await.map_err(|e| BadRequest(Some(e.to_string())))?;
    let speakers=sqlx::query_as::<_, Speaker>("SELECT speaker, crossover, delay, gain, is_subwoofer from speakers where version=(select max(version) as mxversion FROM versions)")
    .fetch_all(&mut *db)
    .await.map_err(|e| BadRequest(Some(e.to_string())))?;
    Ok(Json(ProcessorSettings { filters, speakers }))
}

#[get("/config/<version>")]
async fn config_version(
    mut db: Connection<Settings>,
    version: &str,
) -> Result<Json<ProcessorSettings>, BadRequest<String>> {
    let filters =
        sqlx::query_as::<_, Filter>("SELECT speaker, freq, gain, q from filters where version=?")
            .bind(version)
            .fetch_all(&mut *db)
            .await
            .map_err(|e| BadRequest(Some(e.to_string())))?;
    let speakers = sqlx::query_as::<_, Speaker>(
        "SELECT speaker, crossover, delay, gain, is_subwoofer from speakers where version=?",
    )
    .bind(version)
    .fetch_all(&mut *db)
    .await
    .map_err(|e| BadRequest(Some(e.to_string())))?;
    Ok(Json(ProcessorSettings { filters, speakers }))
}

/*
#[put("/configuration", format = "application/json", data = "<settings>")]
async fn write_configuration(
    mut db: Connection<Settings>,
    settings: Json<ProcessorSettings<'_>>,
) -> Option<String> {
    sqlx::query("SELECT content FROM logs WHERE id = ?")
        .bind(id)
        .fetch_one(&mut *db)
        .await
        .and_then(|r| Ok(r.try_get(0)?))
        .ok()
}*/

#[launch]
fn rocket() -> _ {
    rocket::build()
        .attach(Settings::init())
        .mount("/", routes![config_latest, config_version])
}

#[cfg(test)]
mod tests {
    use crate::FilterType;
    use crate::PeakingFilter;
    use crate::PeakingParameters;
    use crate::PeakingType;

    use crate::SpeakerAdjust;

    use crate::convert_processor_settings_to_camilla;
    use crate::create_pipeline;
    use crate::Filter;
    use crate::Mixer;
    use crate::ProcessorSettings;
    use crate::Speaker;
    use std::collections::HashMap;
    #[test]
    fn check_mixer_4_speakers_0_sub() {
        let num_speakers = 4;
        let speakers: Vec<Speaker> = vec![
            Speaker {
                speaker: "l".to_string(),
                crossover: 100,
                delay: 10,
                is_subwoofer: false,
                gain: 2,
            },
            Speaker {
                speaker: "r".to_string(),
                crossover: 100,
                delay: 10,
                is_subwoofer: false,
                gain: 2,
            },
            Speaker {
                speaker: "sl".to_string(),
                crossover: 100,
                delay: 10,
                is_subwoofer: false,
                gain: 2,
            },
            Speaker {
                speaker: "sr".to_string(),
                crossover: 100,
                delay: 10,
                is_subwoofer: false,
                gain: 2,
            },
        ];
        let mix = Mixer::init(&speakers);
        assert!(mix.channels.num_in_channel == 4);
        assert!(mix.channels.num_out_channel == 4);
        assert!(mix.mapping.len() == 4);
        assert!(mix.mapping[0].dest == 0);
        assert!(mix.mapping[0].sources.len() == 1);
        assert!(mix.mapping[0].sources[0].channel == 0);

        assert!(mix.mapping[1].dest == 1);
        assert!(mix.mapping[1].sources.len() == 1);
        assert!(mix.mapping[1].sources[0].channel == 1);

        assert!(mix.mapping[2].dest == 2);
        assert!(mix.mapping[2].sources.len() == 1);
        assert!(mix.mapping[2].sources[0].channel == 2);

        assert!(mix.mapping[3].dest == 3);
        assert!(mix.mapping[3].sources.len() == 1);
        assert!(mix.mapping[3].sources[0].channel == 3);
    }
    #[test]
    fn check_mixer_6_speakers_1_sub() {
        let speakers: Vec<Speaker> = vec![
            Speaker {
                speaker: "l".to_string(),
                crossover: 100,
                delay: 10,
                is_subwoofer: false,
                gain: 2,
            },
            Speaker {
                speaker: "r".to_string(),
                crossover: 100,
                delay: 10,
                is_subwoofer: false,
                gain: 2,
            },
            Speaker {
                speaker: "c".to_string(),
                crossover: 100,
                delay: 10,
                is_subwoofer: false,
                gain: 2,
            },
            Speaker {
                speaker: "sl".to_string(),
                crossover: 100,
                delay: 10,
                is_subwoofer: false,
                gain: 2,
            },
            Speaker {
                speaker: "sr".to_string(),
                crossover: 100,
                delay: 10,
                is_subwoofer: false,
                gain: 2,
            },
            Speaker {
                speaker: "sub1".to_string(),
                crossover: 100,
                delay: 10,
                is_subwoofer: true,
                gain: 2,
            },
        ];
        let mix = Mixer::init(&speakers);
        assert!(mix.channels.num_in_channel == 6);
        assert!(mix.channels.num_out_channel == 6);
        assert!(mix.mapping.len() == 7);
        assert!(mix.mapping[0].dest == 0);
        assert!(mix.mapping[0].sources.len() == 1);
        assert!(mix.mapping[0].sources[0].channel == 0);

        assert!(mix.mapping[1].dest == 1);
        assert!(mix.mapping[1].sources.len() == 1);
        assert!(mix.mapping[1].sources[0].channel == 1);

        assert!(mix.mapping[2].dest == 2);
        assert!(mix.mapping[2].sources.len() == 1);
        assert!(mix.mapping[2].sources[0].channel == 2);

        assert!(mix.mapping[3].dest == 3);
        assert!(mix.mapping[3].sources.len() == 1);
        assert!(mix.mapping[3].sources[0].channel == 3);

        assert!(mix.mapping[4].dest == 4);
        assert!(mix.mapping[4].sources.len() == 1);
        assert!(mix.mapping[4].sources[0].channel == 4);

        //sub mapped to sub
        assert!(mix.mapping[5].dest == 5);
        assert!(mix.mapping[5].sources.len() == 1);
        assert!(mix.mapping[5].sources[0].channel == 5);

        //speakers mapped to sub (for crossover purpose)
        assert!(mix.mapping[6].dest == 5);
        assert!(mix.mapping[6].sources.len() == 5);
        assert!(mix.mapping[6].sources[0].channel == 0);
        assert!(mix.mapping[6].sources[1].channel == 1);
        assert!(mix.mapping[6].sources[2].channel == 2);
        assert!(mix.mapping[6].sources[3].channel == 3);
        assert!(mix.mapping[6].sources[4].channel == 4);
    }
    #[test]
    fn check_mixer_7_speakers_2_sub() {
        let speakers: Vec<Speaker> = vec![
            Speaker {
                speaker: "l".to_string(),
                crossover: 100,
                delay: 10,
                is_subwoofer: false,
                gain: 2,
            },
            Speaker {
                speaker: "r".to_string(),
                crossover: 100,
                delay: 10,
                is_subwoofer: false,
                gain: 2,
            },
            Speaker {
                speaker: "c".to_string(),
                crossover: 100,
                delay: 10,
                is_subwoofer: false,
                gain: 2,
            },
            Speaker {
                speaker: "sl".to_string(),
                crossover: 100,
                delay: 10,
                is_subwoofer: false,
                gain: 2,
            },
            Speaker {
                speaker: "sr".to_string(),
                crossover: 100,
                delay: 10,
                is_subwoofer: false,
                gain: 2,
            },
            Speaker {
                speaker: "sub1".to_string(),
                crossover: 100,
                delay: 10,
                is_subwoofer: true,
                gain: 2,
            },
            Speaker {
                speaker: "sub2".to_string(),
                crossover: 100,
                delay: 10,
                is_subwoofer: true,
                gain: 2,
            },
        ];
        let mix = Mixer::init(&speakers);
        assert!(mix.channels.num_in_channel == 6);
        assert!(mix.channels.num_out_channel == 7);
        assert!(mix.mapping.len() == 9);
        assert!(mix.mapping[0].dest == 0);
        assert!(mix.mapping[0].sources.len() == 1);
        assert!(mix.mapping[0].sources[0].channel == 0);

        assert!(mix.mapping[1].dest == 1);
        assert!(mix.mapping[1].sources.len() == 1);
        assert!(mix.mapping[1].sources[0].channel == 1);

        assert!(mix.mapping[2].dest == 2);
        assert!(mix.mapping[2].sources.len() == 1);
        assert!(mix.mapping[2].sources[0].channel == 2);

        assert!(mix.mapping[3].dest == 3);
        assert!(mix.mapping[3].sources.len() == 1);
        assert!(mix.mapping[3].sources[0].channel == 3);

        assert!(mix.mapping[4].dest == 4);
        assert!(mix.mapping[4].sources.len() == 1);
        assert!(mix.mapping[4].sources[0].channel == 4);

        //sub mapped to sub 1
        assert!(mix.mapping[5].dest == 5);
        assert!(mix.mapping[5].sources.len() == 1);
        assert!(mix.mapping[5].sources[0].channel == 5);

        //sub mapped to sub 2
        assert!(mix.mapping[6].dest == 6);
        assert!(mix.mapping[6].sources.len() == 1);
        assert!(mix.mapping[6].sources[0].channel == 5);

        //speakers mapped to subs (for crossover purpose)
        assert!(mix.mapping[7].dest == 5);
        assert!(mix.mapping[7].sources.len() == 5);
        assert!(mix.mapping[7].sources[0].channel == 0);
        assert!(mix.mapping[7].sources[1].channel == 1);
        assert!(mix.mapping[7].sources[2].channel == 2);
        assert!(mix.mapping[7].sources[3].channel == 3);
        assert!(mix.mapping[7].sources[4].channel == 4);

        assert!(mix.mapping[8].dest == 6);
        assert!(mix.mapping[8].sources.len() == 5);
        assert!(mix.mapping[8].sources[0].channel == 0);
        assert!(mix.mapping[8].sources[1].channel == 1);
        assert!(mix.mapping[8].sources[2].channel == 2);
        assert!(mix.mapping[8].sources[3].channel == 3);
        assert!(mix.mapping[8].sources[4].channel == 4);
    }

    #[test]
    fn check_processor_to_camilla() {
        let settings = ProcessorSettings {
            filters: vec![
                Filter {
                    freq: 1000,
                    gain: 2,
                    q: 0.707,
                    speaker: "l".to_string(),
                },
                Filter {
                    freq: 2000,
                    gain: 2,
                    q: 0.707,
                    speaker: "l".to_string(),
                },
                Filter {
                    freq: 2000,
                    gain: 1,
                    q: 0.707,
                    speaker: "r".to_string(),
                },
            ],
            speakers: vec![
                Speaker {
                    speaker: "l".to_string(),
                    crossover: 80,
                    delay: 10,
                    gain: 1, //ugh, I forgot about gains...
                    is_subwoofer: false,
                },
                Speaker {
                    speaker: "c".to_string(),
                    crossover: 80,
                    delay: 10,
                    gain: 1, //ugh, I forgot about gains...
                    is_subwoofer: false,
                },
                Speaker {
                    speaker: "r".to_string(),
                    crossover: 80,
                    delay: 10,
                    gain: 1, //ugh, I forgot about gains...
                    is_subwoofer: false,
                },
                Speaker {
                    speaker: "sub1".to_string(),
                    crossover: 80,
                    delay: 10,
                    gain: 1, //ugh, I forgot about gains...
                    is_subwoofer: true,
                },
            ],
        };
        println!("Json {}", convert_processor_settings_to_camilla(&settings));
        //let result=convert_processor_settings_to_camilla()
    }

    #[test]
    fn check_create_pipeline() {
        let speakers = vec![
            Speaker {
                speaker: "l".to_string(),
                crossover: 80,
                delay: 10,
                gain: 1, //ugh, I forgot about gains...
                is_subwoofer: false,
            },
            Speaker {
                speaker: "c".to_string(),
                crossover: 80,
                delay: 10,
                gain: 1, //ugh, I forgot about gains...
                is_subwoofer: false,
            },
            Speaker {
                speaker: "r".to_string(),
                crossover: 80,
                delay: 10,
                gain: 1, //ugh, I forgot about gains...
                is_subwoofer: false,
            },
            Speaker {
                speaker: "sub1".to_string(),
                crossover: 80,
                delay: 10,
                gain: 1, //ugh, I forgot about gains...
                is_subwoofer: true,
            },
        ];
        let filters = vec![
            Filter {
                q: 0.7,
                freq: 1000,
                gain: 2,
                speaker: "l".to_string(),
            },
            Filter {
                q: 0.7,
                freq: 2000,
                gain: 2,
                speaker: "l".to_string(),
            },
        ];
        let result = create_pipeline(&speakers, &filters);
        assert!(result.len() == 9); //only have peq for left speaker
    }
}
