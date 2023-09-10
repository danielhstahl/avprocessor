#[macro_use]
extern crate rocket;
use chrono::Utc;
use rocket::fairing::{self, AdHoc};
use rocket::response::status::BadRequest;
use rocket::serde::{json, json::Json, Deserialize, Serialize};
use rocket::{Build, Rocket};
use rocket_db_pools::sqlx::{self};
use rocket_db_pools::{Connection, Database};
use std::collections::HashMap;
#[derive(Database)]
#[database("settings")]
struct Settings(sqlx::SqlitePool);

async fn run_migrations(rocket: Rocket<Build>) -> fairing::Result {
    if let Some(db) = Settings::fetch(&rocket) {
        let _1=sqlx::query(
            "CREATE TABLE if not exists filters (version text, filter_index integer, speaker text, freq integer, gain integer, q real, PRIMARY KEY (version, filter_index, speaker));",
        )
        .execute(&db.0)
        .await;

        let _2=sqlx::query(
        "CREATE TABLE if not exists speakers (version text, speaker text, crossover integer, delay integer, gain integer, is_subwoofer integer, PRIMARY KEY (version, speaker));",
        )
        .execute(&db.0)
        .await;

        let _3 = sqlx::query("CREATE TABLE if not exists versions (version text PRIMARY KEY);")
            .execute(&db.0)
            .await;

        Ok(rocket)
    } else {
        Err(rocket)
    }
}

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
struct Mixer {
    channels: ChannelCount,
    mapping: Vec<Mapping>,
}

const NUM_INPUT_SUBWOOFERS: usize = 1;

struct SpeakerCounts {
    speakers_exclude_sub: usize,
    input_subwoofers: usize,
    output_subwoofers: usize,
}
fn get_speaker_counts(speakers: &[Speaker]) -> SpeakerCounts {
    let output_subwoofers = speakers
        .iter()
        .enumerate()
        .filter(|(_, s)| s.is_subwoofer)
        //.map(|(i, _)| i)
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

struct CrossoverChannels {
    speaker_channels: Vec<usize>,
    subwoofer_channels: Vec<usize>, //these will be the same size; split each main speaker into subwoofer channels
}

fn split_inputs(speaker_counts: &SpeakerCounts) -> Option<(Mixer, CrossoverChannels)> {
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

fn create_crossover_filters(speakers: &[Speaker]) -> HashMap<String, SpeakerAdjust> {
    HashMap::from_iter(
        speakers
            .iter()
            .filter(|s| !s.is_subwoofer)
            .map(|speaker| {
                vec![
                    (
                        crossover_speaker_name(&speaker.speaker),
                        SpeakerAdjust::CrossoverFilter(CrossoverFilter {
                            filter_type: FilterType::BiquadCombo,
                            parameters: CrossoverParameters {
                                freq: speaker.crossover,
                                order: 4,
                                crossover_type: CrossoverType::ButterworthHighpass,
                            },
                        }),
                    ),
                    (
                        crossover_subwoofer_name(&speaker.speaker),
                        SpeakerAdjust::CrossoverFilter(CrossoverFilter {
                            filter_type: FilterType::BiquadCombo,
                            parameters: CrossoverParameters {
                                freq: speaker.crossover,
                                order: 4,
                                crossover_type: CrossoverType::ButterworthLowpass,
                            },
                        }),
                    ),
                ]
                .into_iter()
            })
            .flatten(),
    )
}

//performed after split_inputs and crossover filters in pipeline
fn combine_inputs(
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

fn create_output_filters(
    speakers: &[Speaker],
    peq_filters: &HashMap<&String, Vec<(usize, &Filter)>>,
) -> HashMap<String, SpeakerAdjust> {
    HashMap::from_iter(
        peq_filters
            .iter()
            .map(|(speaker, peq)| {
                peq.iter().map(move |(index, f)| {
                    (
                        peq_filter_name(&speaker, *index),
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
            })
            .flatten()
            .chain(speakers.iter().map(|s| {
                (
                    delay_filter_name(&s.speaker),
                    SpeakerAdjust::DelayFilter(DelayFilter {
                        filter_type: FilterType::Delay,
                        parameters: DelayParameters {
                            delay: s.delay,
                            unit: DelayUnit::Ms,
                        },
                    }),
                )
            }))
            .chain(speakers.iter().map(|s| {
                (
                    gain_filter_name(&s.speaker),
                    SpeakerAdjust::GainFilter(GainFilter {
                        filter_type: FilterType::Gain,
                        parameters: GainParameters {
                            gain: s.gain,
                            inverted: false,
                        },
                    }),
                )
            })),
    )
}

fn create_per_speaker_pipeline(
    speakers: &[Speaker],
    peq_filters: &HashMap<&String, Vec<(usize, &Filter)>>,
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

fn create_pipeline(
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

/**End Mixers */

/**Filters */

#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
enum FilterType {
    //PEAKING,
    Delay,
    Biquad, //both peaking and highpass are BIQUAD filters
    BiquadCombo,
    Gain,
}

#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
enum DelayUnit {
    //more may be added later
    #[serde(rename = "ms")]
    Ms,
}

#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
enum CrossoverType {
    //more may be added later
    ButterworthHighpass,
    ButterworthLowpass,
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
struct GainParameters {
    gain: i32,
    inverted: bool,
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
struct GainFilter {
    #[serde(rename = "type")]
    filter_type: FilterType,
    parameters: GainParameters,
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
#[serde(untagged)]
enum SpeakerAdjust {
    DelayFilter(DelayFilter),
    PeakingFilter(PeakingFilter),
    CrossoverFilter(CrossoverFilter),
    GainFilter(GainFilter),
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
#[serde(untagged)]
enum Pipeline {
    Filter(PipelineFilter),
    Mixer(PipelineMixer),
}

fn delay_filter_name(speaker_name: &str) -> String {
    format!("delay_{}", speaker_name)
}
fn gain_filter_name(speaker_name: &str) -> String {
    format!("gain_{}", speaker_name)
}
fn peq_filter_name(speaker_name: &str, peq_index: usize) -> String {
    format!("peq_{}_{}", speaker_name, peq_index)
}
fn crossover_speaker_name(speaker_name: &str) -> String {
    format!("crossover_speaker_{}", speaker_name)
}
fn crossover_subwoofer_name(speaker_name: &str) -> String {
    format!("crossover_subwoofer{}", speaker_name)
}
fn split_mixer_name() -> String {
    "split_non_sub".to_string()
}
fn combine_mixer_name() -> String {
    "combine_sub".to_string()
}

/**end pipeline */

#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
struct CamillaConfig {
    mixers: HashMap<String, Mixer>,
    filters: HashMap<String, SpeakerAdjust>,
    pipeline: Vec<Pipeline>,
}

fn compute_peq_filter<'a>(filters: &'a [Filter]) -> HashMap<&'a String, Vec<(usize, &'a Filter)>> {
    let mut hold_filters: HashMap<&String, Vec<(usize, &Filter)>> = HashMap::new();
    for (index, filter) in filters.iter().enumerate() {
        hold_filters
            .entry(&filter.speaker)
            .and_modify(|v| v.push((index, filter)))
            .or_insert(vec![(index, filter)]);
    }
    hold_filters
}

//this is used purely to store state and pass to mixer and filter creators
struct ConfigurationMapping<'a> {
    peq_filters: HashMap<&'a String, Vec<(usize, &'a Filter)>>,
    speaker_counts: SpeakerCounts,
}

/// how to handle WITH subs but no crossover?  Will need a mix only for the sub channel (split source sub into two)
fn convert_processor_settings_to_camilla(
    settings: &ProcessorSettings,
) -> Result<String, json::serde_json::Error> {
    let configuration_mapping = ConfigurationMapping {
        peq_filters: compute_peq_filter(&settings.filters),
        speaker_counts: get_speaker_counts(&settings.speakers),
    };

    let split_mixer = split_inputs(&configuration_mapping.speaker_counts);
    let output_filters =
        create_output_filters(&settings.speakers, &configuration_mapping.peq_filters);

    let mut per_speaker_pipeline =
        create_per_speaker_pipeline(&settings.speakers, &configuration_mapping.peq_filters);
    match split_mixer {
        Some((mixer, crossover_channels)) => {
            let combine_mixer = combine_inputs(
                &configuration_mapping.speaker_counts,
                &crossover_channels,
                &settings.speakers,
            );
            let mixers: HashMap<String, Mixer> = HashMap::from_iter(
                vec![
                    (split_mixer_name(), mixer),
                    (combine_mixer_name(), combine_mixer),
                ]
                .into_iter(),
            );
            let mut filters = create_crossover_filters(&settings.speakers);
            let mut pipeline = create_pipeline(
                split_mixer_name(),
                combine_mixer_name(),
                //&filters,
                &crossover_channels,
                &settings.speakers,
            );
            pipeline.append(&mut per_speaker_pipeline);
            //pipeline.extend(per_speaker_pipeline);
            filters.extend(output_filters);
            let result = CamillaConfig {
                pipeline,
                filters,
                mixers,
            };
            json::to_string(&result)
        }
        None => {
            let result = CamillaConfig {
                pipeline: per_speaker_pipeline,
                filters: output_filters,
                mixers: HashMap::new(),
            };
            json::to_string(&result)
        }
    }

    //filters.extend(create_output_filters(&settings.speakers, &settings.filters));

    /*let pipeline = create_pipeline(
        split_mixer_name(),
        combine_mixer_name(),
        &settings.speakers,
        &settings.filters,
    );*/
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

use tungstenite::{connect, Message};
use url::Url;

#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
struct SetConfig {
    #[serde(rename = "SetConfigJson")]
    set_config_json: String,
}

#[put("/config", format = "application/json", data = "<settings>")]
async fn write_configuration(
    mut db: Connection<Settings>,
    settings: Json<ProcessorSettings>,
) -> Result<(), BadRequest<String>> {
    let version = Utc::now().to_string();

    let config = convert_processor_settings_to_camilla(&settings)
        .map_err(|e| BadRequest(Some(e.to_string())))?;

    println!("json: {}", config);
    let (mut socket, _response) =
        connect(Url::parse("ws://127.0.0.1:1234").unwrap()).expect("Can't connect");
    socket
        .send(Message::Text(
            json::to_string(&SetConfig {
                set_config_json: config,
            })
            .unwrap(),
        )) //SetConfigJson
        .unwrap();

    //write config to camilla here
    let _ = sqlx::query("INSERT INTO versions (version) VALUES (?)")
        .bind(&version)
        .execute(&mut *db)
        .await;
    for (index, filter) in settings.filters.iter().enumerate() {
        let _ = sqlx::query(
            "INSERT INTO filters (version, filter_index, speaker, freq, gain, q) VALUES (?, ?, ?, ?, ?, ?)",
        )
        .bind(&version)
        .bind(index as i32)
        .bind(&filter.speaker)
        .bind(filter.freq)
        .bind(filter.gain)
        .bind(filter.q)
        .execute(&mut *db)
        .await;
    }
    for speaker in settings.speakers.iter() {
        let _ =sqlx::query(
            "INSERT INTO speakers (version, speaker, crossover, delay, gain, is_subwoofer) VALUES (?, ?, ?, ?, ?, ?)",
        )
        .bind(&version)
        .bind(&speaker.speaker)
        .bind(speaker.crossover)
        .bind(speaker.delay)
        .bind(speaker.gain)
        .bind(speaker.is_subwoofer)
        .execute(&mut *db)
        .await;
    }
    Ok(())
}

#[launch]
fn rocket() -> _ {
    rocket::build()
        .attach(Settings::init())
        .attach(AdHoc::try_on_ignite("DB Migrations", run_migrations))
        .mount(
            "/",
            routes![config_latest, config_version, write_configuration],
        )
}

#[cfg(test)]
mod tests {
    use crate::combine_inputs;
    use crate::convert_processor_settings_to_camilla;
    use crate::create_pipeline;
    use crate::get_speaker_counts;
    use crate::split_inputs;
    use crate::CrossoverChannels;
    use crate::Filter;
    use crate::ProcessorSettings;
    use crate::Speaker;
    #[test]
    fn test_speaker_counts_no_sub() {
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
            Speaker {
                speaker: "sub1".to_string(),
                crossover: 100,
                delay: 10,
                is_subwoofer: true,
                gain: 2,
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
        let result = split_inputs(&get_speaker_counts(&speakers));
        assert!(result.is_none());
    }
    #[test]
    fn test_init_mixer_one_sub() {
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
            Speaker {
                speaker: "sub1".to_string(),
                crossover: 100,
                delay: 10,
                is_subwoofer: true,
                gain: 2,
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
                    gain: 1,
                    is_subwoofer: false,
                },
                Speaker {
                    speaker: "c".to_string(),
                    crossover: 80,
                    delay: 10,
                    gain: 1,
                    is_subwoofer: false,
                },
                Speaker {
                    speaker: "r".to_string(),
                    crossover: 80,
                    delay: 10,
                    gain: 1,
                    is_subwoofer: false,
                },
                Speaker {
                    speaker: "sub1".to_string(),
                    crossover: 80,
                    delay: 10,
                    gain: 1,
                    is_subwoofer: true,
                },
            ],
        };
        println!(
            "Yaml {}",
            convert_processor_settings_to_camilla(&settings).unwrap()
        );
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
        let result = create_pipeline(
            "myinitmixer".to_string(),
            "myfinalmixer".to_string(),
            &CrossoverChannels {
                speaker_channels: vec![0, 1, 2],
                subwoofer_channels: vec![3, 4, 5],
            },
            &speakers,
        );
        assert!(result.len() == 8); //2*(4-1) for cxfilters+2 for mixer
    }
}
