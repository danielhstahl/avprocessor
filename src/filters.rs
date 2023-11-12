use crate::processor::Filter;
use crate::processor::Speaker;
use rocket::serde::Serialize;
use std::collections::BTreeMap;
pub fn delay_filter_name(speaker_name: &str) -> String {
    format!("delay_{}", speaker_name)
}
pub fn gain_filter_name(speaker_name: &str) -> String {
    format!("gain_{}", speaker_name)
}

pub fn peq_filter_name(speaker_name: &str, peq_index: usize) -> String {
    format!("peq_{}_{}", speaker_name, peq_index)
}
pub fn volume_filter_name() -> String {
    format!("volume")
}
pub fn crossover_speaker_name(speaker_name: &str) -> String {
    format!("crossover_speaker_{}", speaker_name)
}
pub fn crossover_subwoofer_name(speaker_name: &str) -> String {
    format!("crossover_subwoofer{}", speaker_name)
}

/// generates Biquad filters for crossovers for both speakers and subs
pub fn create_crossover_filters(speakers: &[Speaker]) -> BTreeMap<String, SpeakerAdjust> {
    BTreeMap::from_iter(
        speakers
            .iter()
            .filter(|s| !s.is_subwoofer)
            .filter(|s| s.crossover.is_some())
            .map(|speaker| {
                vec![
                    (
                        crossover_speaker_name(&speaker.speaker),
                        SpeakerAdjust::CrossoverFilter(CrossoverFilter {
                            filter_type: FilterType::BiquadCombo,
                            parameters: CrossoverParameters {
                                freq: speaker.crossover.unwrap(), //already filtered out
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
                                freq: speaker.crossover.unwrap(), //already filtered out
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

/// generates PEQ filters
pub fn create_output_filters(
    speakers: &[Speaker],
    peq_filters: &BTreeMap<&String, Vec<(usize, &Filter)>>,
) -> BTreeMap<String, SpeakerAdjust> {
    BTreeMap::from_iter(
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
            }))
            .chain(speakers.iter().map(|_s| {
                (
                    volume_filter_name(),
                    SpeakerAdjust::VolumeFilter(VolumeFilter {
                        filter_type: FilterType::Volume,
                        parameters: VolumeParameters { ramp_time: 200 }, //200 ms
                    }),
                )
            })),
    )
}

pub fn compute_peq_filter<'a>(
    filters: &'a [Filter],
) -> BTreeMap<&'a String, Vec<(usize, &'a Filter)>> {
    let mut hold_filters: BTreeMap<&String, Vec<(usize, &Filter)>> = BTreeMap::new();
    for (index, filter) in filters.iter().enumerate() {
        hold_filters
            .entry(&filter.speaker)
            .and_modify(|v| v.push((index, filter)))
            .or_insert(vec![(index, filter)]);
    }
    hold_filters
}

#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
enum FilterType {
    Delay,
    Biquad,
    BiquadCombo,
    Gain,
    Volume,
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
    gain: f32,
    #[serde(rename = "type")]
    peaking_type: PeakingType,
}

#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
struct VolumeParameters {
    ramp_time: i32,
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
    delay: f32,
    unit: DelayUnit,
}

#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
struct GainParameters {
    gain: f32,
    inverted: bool,
}

#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
pub struct PeakingFilter {
    #[serde(rename = "type")]
    filter_type: FilterType,
    parameters: PeakingParameters,
}

#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
pub struct DelayFilter {
    #[serde(rename = "type")]
    filter_type: FilterType,
    parameters: DelayParameters,
}

#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
pub struct VolumeFilter {
    #[serde(rename = "type")]
    filter_type: FilterType,
    parameters: VolumeParameters,
}

#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
pub struct GainFilter {
    #[serde(rename = "type")]
    filter_type: FilterType,
    parameters: GainParameters,
}

#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
pub struct CrossoverFilter {
    #[serde(rename = "type")]
    filter_type: FilterType,
    parameters: CrossoverParameters,
}

#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
#[serde(untagged)]
pub enum SpeakerAdjust {
    DelayFilter(DelayFilter),
    PeakingFilter(PeakingFilter),
    CrossoverFilter(CrossoverFilter),
    GainFilter(GainFilter),
    VolumeFilter(VolumeFilter),
}
