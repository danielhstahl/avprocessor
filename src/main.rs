#[macro_use]
extern crate rocket;
use core::num;
use rocket::serde::{json, json::Json, Deserialize, Serialize};
use std::collections::HashMap;
use std::iter;

use rocket_db_pools::sqlx::{self, Row};
use rocket_db_pools::{Connection, Database};

#[derive(Database)]
#[database("settings")]
struct Settings(sqlx::SqlitePool);

#[get("/<id>")]
async fn read(mut db: Connection<Settings>, id: i64) -> Option<String> {
    sqlx::query("SELECT content FROM logs WHERE id = ?")
        .bind(id)
        .fetch_one(&mut *db)
        .await
        .and_then(|r| Ok(r.try_get(0)?))
        .ok()
}

#[derive(Deserialize)]
#[serde(crate = "rocket::serde")]
struct Filter {
    //filter_type: FilterType, //&'a str, //delay or peaking
    freq: i32,
    gain: i32,
    q: f32,
    //key: &'a str,
    speaker: String,
}

/*
#[derive(Deserialize)]
#[serde(crate = "rocket::serde")]
struct Delay<'a> {
    //filter_type: FilterType, //&'a str, //delay or peaking
    delay: f32,
    unit: DelayUnit,
    //key: &'a str,
    speaker: &'a str,
}*/

#[derive(Deserialize)]
#[serde(crate = "rocket::serde")]
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
#[derive(Deserialize)]
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
    num_in_channel: usize, //will need to convert this to "in"
    #[serde(rename = "out")]
    num_out_channel: usize, //will need to convert this to "out"
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
    fn init(num_speakers: usize, subwoofer_indeces: &[usize]) -> Self {
        let num_output_subwoofers = subwoofer_indeces.len();
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
            mapping: (0..num_non_sub_channels)
                .map(|i: usize| Mapping {
                    //input to output mapping
                    dest: i,
                    sources: vec![Source {
                        channel: i,
                        gain: 0,
                        inverted: false,
                    }],
                })
                .chain(
                    //subwoofer input to subwoofer(s) output
                    subwoofer_indeces.iter().map(|sub_index| Mapping {
                        dest: *sub_index,
                        sources: vec![Source {
                            channel: num_non_sub_channels,
                            gain: 0,
                            inverted: false,
                        }],
                    }),
                )
                .chain(
                    //crossover
                    subwoofer_indeces.iter().map(|sub_index| Mapping {
                        dest: *sub_index,
                        sources: (0..num_non_sub_channels)
                            .map(|i| Source {
                                channel: i,
                                gain: 0,
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
#[serde(untagged)]
enum FilterType {
    //PEAKING,
    Delay,
    Biquad, //both peaking and highpass are BIQUAD filters
    BiquadCombo,
}

#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
#[serde(untagged)]
enum DelayUnit {
    //more may be added later
    MS,
}

#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
#[serde(untagged)]
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
    filter_type: FilterType, // filter_type needs to be changed to "type" on serialization
    parameters: PeakingParameters,
}

#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
struct DelayFilter {
    #[serde(rename = "type")]
    filter_type: FilterType, // filter_type needs to be changed to "type" on serialization
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
#[serde(untagged)]
enum SpeakerAdjust {
    DelayFilter(DelayFilter),
    PeakingFilter(PeakingFilter),
    CrossoverFilter(CrossoverFilter),
}

/**End Filters */

/**begin pipelin */

#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
#[serde(untagged)]
enum PipelineType {
    FILTER,
    MIXER,
}

#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
struct PipelineFilter {
    #[serde(rename = "type")]
    pipeline_type: PipelineType, //change to "type" on serialization
    channel: usize,
    names: Vec<String>, //these are keys in the Filter hashmap
}

#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
struct PipelineMixer {
    #[serde(rename = "type")]
    pipeline_type: PipelineType, //change to "type" on serialization
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
fn peq_filter_name(speaker_name: &str) -> String {
    format!("peq_{}", speaker_name)
}
fn crossover_filter_name(speaker_name: &str) -> String {
    format!("crossover_{}", speaker_name)
}
fn subwoofer_mixer_name() -> String {
    "subwoofer".to_string()
}

fn create_pipeline(speakers: &[Speaker]) -> Vec<Pipeline> {
    speakers
        .iter()
        .enumerate()
        .map(|(i, s)| {
            Pipeline::Filter(PipelineFilter {
                pipeline_type: PipelineType::FILTER,
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
                        pipeline_type: PipelineType::FILTER,
                        channel: i,
                        names: vec![crossover_filter_name(&s.speaker)],
                    })
                }),
        )
        .chain(
            vec![Pipeline::Mixer(PipelineMixer {
                pipeline_type: PipelineType::MIXER,
                name: subwoofer_mixer_name(),
            })]
            .into_iter(),
        )
        .chain(speakers.iter().enumerate().map(|(i, s)| {
            Pipeline::Filter(PipelineFilter {
                pipeline_type: PipelineType::FILTER,
                channel: i,
                names: vec![peq_filter_name(&s.speaker)],
            })
        }))
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
    let num_speakers = settings.speakers.len();
    let subwoofer_indeces: Vec<usize> = settings
        .speakers
        .iter()
        .enumerate()
        .filter(|(i, s)| s.is_subwoofer)
        .map(|(i, s)| i)
        .collect();
    let mixers: HashMap<String, Mixer> = HashMap::from([(
        subwoofer_mixer_name(),
        Mixer::init(num_speakers, &subwoofer_indeces),
    )]);

    let filters: HashMap<String, SpeakerAdjust> = HashMap::from_iter(
        settings
            .filters
            .iter()
            .map(|f| {
                (
                    peq_filter_name(&f.speaker),
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

    let pipeline = create_pipeline(&settings.speakers);
    let result = CamillaConfig {
        pipeline,
        filters,
        mixers,
    };
    json::to_string(&result).unwrap()
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
        .mount("/", routes![read])
}

#[cfg(test)]
mod tests {
    use super::convert_processor_settings_to_camilla;
    use super::Mixer;
    #[test]
    fn check_mixer_4_speakers_0_sub() {
        let num_speakers = 4;
        let subwoofer_indeces: Vec<usize> = vec![];
        let mix = Mixer::init(num_speakers, &subwoofer_indeces);
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
        let num_speakers = 6;
        let subwoofer_indeces: Vec<usize> = vec![5];
        let mix = Mixer::init(num_speakers, &subwoofer_indeces);
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
        let num_speakers = 7;
        let subwoofer_indeces: Vec<usize> = vec![5, 6];
        let mix = Mixer::init(num_speakers, &subwoofer_indeces);
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
        //let result=convert_processor_settings_to_camilla()
    }
}
