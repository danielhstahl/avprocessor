#[macro_use]
extern crate rocket;
use chrono::Utc;
use rocket::fairing::{self, AdHoc};
use rocket::response::status::BadRequest;
use rocket::serde::{json, json::Json, Serialize};
use rocket::State;
use rocket::{Build, Rocket};
use rocket_db_pools::sqlx::{self};
use rocket_db_pools::{Connection, Database};
use std::collections::BTreeMap;
mod filters;
mod mixers;
mod pipeline;
mod processor;

use filters::{compute_peq_filter, create_crossover_filters, create_output_filters, SpeakerAdjust};
use mixers::{
    combine_inputs, combine_mixer_name, get_speaker_counts, split_inputs, split_mixer_name, Mixer,
    SpeakerCounts,
};
use pipeline::{create_crossover_pipeline, create_per_speaker_pipeline, Pipeline};
use processor::{Filter, ProcessorSettings, Speaker};

#[derive(Database)]
#[database("settings")]
struct Settings(sqlx::SqlitePool);

async fn run_migrations(rocket: Rocket<Build>) -> fairing::Result {
    if let Some(db) = Settings::fetch(&rocket) {
        let _1=sqlx::query(
            "CREATE TABLE if not exists filters (version text not null, filter_index integer not null, speaker text not null, freq integer not null, gain real not null, q real not null, PRIMARY KEY (version, filter_index, speaker));",
        )
        .execute(&db.0)
        .await;

        let _2=sqlx::query(
        "CREATE TABLE if not exists speakers (version text not null, speaker text not null, crossover integer, delay integer not null, gain real not null, is_subwoofer integer not null, PRIMARY KEY (version, speaker));",
        )
        .execute(&db.0)
        .await;

        let _3 =
            sqlx::query("CREATE TABLE if not exists versions (version text not null PRIMARY KEY);")
                .execute(&db.0)
                .await;

        Ok(rocket)
    } else {
        Err(rocket)
    }
}

#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
struct CamillaConfig {
    mixers: BTreeMap<String, Mixer>,
    filters: BTreeMap<String, SpeakerAdjust>,
    pipeline: Vec<Pipeline>,
}

//this is used purely to store state and pass to mixer and filter creators
struct ConfigurationMapping<'a> {
    peq_filters: BTreeMap<&'a String, Vec<(usize, &'a Filter)>>,
    speaker_counts: SpeakerCounts,
}

fn convert_processor_settings_to_camilla(
    settings: &ProcessorSettings,
) -> Result<String, json::serde_json::Error> {
    let configuration_mapping = ConfigurationMapping {
        peq_filters: compute_peq_filter(&settings.filters),
        speaker_counts: get_speaker_counts(&settings.speakers),
    };

    let split_mixer = split_inputs(&settings.speakers, &configuration_mapping.speaker_counts);
    let output_filters =
        create_output_filters(&settings.speakers, &configuration_mapping.peq_filters);

    let mut per_speaker_pipeline =
        create_per_speaker_pipeline(&settings.speakers, &configuration_mapping.peq_filters);
    match split_mixer {
        Some((mixer, input_channel_mapping, output_channel_mapping)) => {
            let combine_mixer = combine_inputs(
                &configuration_mapping.speaker_counts,
                &mixer,
                &output_channel_mapping,
                //&settings.speakers,
            );
            let mixers: BTreeMap<String, Mixer> = BTreeMap::from_iter(
                vec![
                    (split_mixer_name(), mixer),
                    (combine_mixer_name(), combine_mixer),
                ]
                .into_iter(),
            );
            let mut filters: BTreeMap<String, SpeakerAdjust> =
                create_crossover_filters(&settings.speakers);
            let mut pipeline = create_crossover_pipeline(
                split_mixer_name(),
                combine_mixer_name(),
                &input_channel_mapping,
            );
            pipeline.append(&mut per_speaker_pipeline);
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
                mixers: BTreeMap::new(),
            };
            json::to_string(&result)
        }
    }
}

#[derive(Serialize, sqlx::FromRow)]
#[serde(crate = "rocket::serde", rename_all = "camelCase")]
struct Version {
    version: String,
}
#[get("/versions")]
async fn get_versions(
    mut db: Connection<Settings>,
) -> Result<Json<Vec<Version>>, BadRequest<String>> {
    let versions = sqlx::query_as::<_, Version>("SELECT version FROM versions")
        .fetch_all(&mut *db)
        .await
        .map_err(|e| BadRequest(Some(e.to_string())))?;

    Ok(Json(versions))
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
    db: Connection<Settings>,
    version: &str,
) -> Result<Json<ProcessorSettings>, BadRequest<String>> {
    get_version_from_db(db, version)
        .await
        .map(|v| Json(v))
        .map_err(|e| BadRequest(Some(e.to_string())))
}

async fn get_version_from_db(
    mut db: Connection<Settings>,
    version: &str,
) -> Result<ProcessorSettings, sqlx::Error> {
    let filters =
        sqlx::query_as::<_, Filter>("SELECT speaker, freq, gain, q from filters where version=?")
            .bind(version)
            .fetch_all(&mut *db)
            .await?;
    let speakers = sqlx::query_as::<_, Speaker>(
        "SELECT speaker, crossover, delay, gain, is_subwoofer from speakers where version=?",
    )
    .bind(version)
    .fetch_all(&mut *db)
    .await?;
    Ok(ProcessorSettings { filters, speakers })
}

use tungstenite::{connect, Message};
use url::Url;

#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
struct SetConfig {
    #[serde(rename = "SetConfigJson")]
    set_config_json: String,
}

#[post("/config/apply/<version>", format = "application/json")]
async fn apply_config_version(
    db: Connection<Settings>,
    version: &str,
    camilla_dsp_url: &State<String>,
) -> Result<(), BadRequest<String>> {
    let settings = get_version_from_db(db, version)
        .await
        .map_err(|e| BadRequest(Some(e.to_string())))?;
    let config = convert_processor_settings_to_camilla(&settings)
        .map_err(|e| BadRequest(Some(e.to_string())))?;

    let ws_url = Url::parse(camilla_dsp_url).map_err(|e| BadRequest(Some(e.to_string())))?;
    let (mut socket, _response) = connect(ws_url).map_err(|e| BadRequest(Some(e.to_string())))?;
    let config_as_json = json::to_string(&SetConfig {
        set_config_json: config,
    })
    .map_err(|e| BadRequest(Some(e.to_string())))?;
    socket
        .send(Message::Text(config_as_json))
        .map_err(|e| BadRequest(Some(e.to_string())))?;
    Ok(())
}

#[put("/config", format = "application/json", data = "<settings>")]
async fn write_configuration(
    mut db: Connection<Settings>,
    settings: Json<ProcessorSettings>,
) -> Result<String, BadRequest<String>> {
    let version = Utc::now().to_string();
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
    Ok(version)
}

#[delete("/config/<version>")]
async fn delete_configuration(
    mut db: Connection<Settings>,
    version: &str,
) -> Result<String, BadRequest<String>> {
    let _ = sqlx::query("DELETE FROM versions WHERE version=?")
        .bind(&version)
        .execute(&mut *db)
        .await;
    let _ = sqlx::query("DELETE FROM filters WHERE version=?")
        .bind(&version)
        .execute(&mut *db)
        .await;
    let _ = sqlx::query("DELETE FROM speakers WHERE version=?")
        .bind(&version)
        .execute(&mut *db)
        .await;
    Ok(version.to_string())
}

#[launch]
fn rocket() -> _ {
    let mut args = std::env::args();
    args.next(); //first item is the app name, skip it
    let camilla_dsp_url = args.next().unwrap_or("ws://127.0.0.1:1234".to_string());
    rocket::build()
        .manage(camilla_dsp_url)
        .attach(Settings::init())
        .attach(AdHoc::try_on_ignite("DB Migrations", run_migrations))
        .mount(
            "/",
            routes![
                config_latest,
                config_version,
                write_configuration,
                apply_config_version,
                delete_configuration,
                get_versions
            ],
        )
}

#[cfg(test)]
mod tests {
    use super::convert_processor_settings_to_camilla;
    use crate::processor::ProcessorSettings;
    use crate::processor::{Filter, Speaker};
    #[test]
    fn check_processor_to_camilla_one_sub() {
        let settings = ProcessorSettings {
            filters: vec![
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
            ],
            speakers: vec![
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
                    crossover: None,
                    delay: 10,
                    gain: 1.0,
                    is_subwoofer: true,
                },
            ],
        };
        /*println!(
            "yaml: {}",
            convert_processor_settings_to_camilla(&settings).unwrap()
        );*/
        assert_eq!(
            convert_processor_settings_to_camilla(&settings).unwrap(),
            r#"{"mixers":{"combine_sub":{"channels":{"in":7,"out":4},"mapping":[{"sources":[{"channel":2,"gain":0,"inverted":false}],"dest":1},{"sources":[{"channel":0,"gain":0,"inverted":false}],"dest":0},{"sources":[{"channel":4,"gain":0,"inverted":false}],"dest":2},{"sources":[{"channel":1,"gain":0,"inverted":false},{"channel":3,"gain":0,"inverted":false},{"channel":5,"gain":0,"inverted":false},{"channel":6,"gain":0,"inverted":false}],"dest":3}]},"split_non_sub":{"channels":{"in":4,"out":7},"mapping":[{"sources":[{"channel":1,"gain":0,"inverted":false}],"dest":2},{"sources":[{"channel":1,"gain":0,"inverted":false}],"dest":3},{"sources":[{"channel":0,"gain":0,"inverted":false}],"dest":0},{"sources":[{"channel":0,"gain":0,"inverted":false}],"dest":1},{"sources":[{"channel":2,"gain":0,"inverted":false}],"dest":4},{"sources":[{"channel":2,"gain":0,"inverted":false}],"dest":5},{"sources":[{"channel":3,"gain":0,"inverted":false}],"dest":6}]}},"filters":{"crossover_speaker_c":{"type":"BiquadCombo","parameters":{"freq":80,"order":4,"type":"ButterworthHighpass"}},"crossover_speaker_l":{"type":"BiquadCombo","parameters":{"freq":80,"order":4,"type":"ButterworthHighpass"}},"crossover_speaker_r":{"type":"BiquadCombo","parameters":{"freq":80,"order":4,"type":"ButterworthHighpass"}},"crossover_subwooferc":{"type":"BiquadCombo","parameters":{"freq":80,"order":4,"type":"ButterworthLowpass"}},"crossover_subwooferl":{"type":"BiquadCombo","parameters":{"freq":80,"order":4,"type":"ButterworthLowpass"}},"crossover_subwooferr":{"type":"BiquadCombo","parameters":{"freq":80,"order":4,"type":"ButterworthLowpass"}},"delay_c":{"type":"Delay","parameters":{"delay":10,"unit":"ms"}},"delay_l":{"type":"Delay","parameters":{"delay":10,"unit":"ms"}},"delay_r":{"type":"Delay","parameters":{"delay":10,"unit":"ms"}},"delay_sub1":{"type":"Delay","parameters":{"delay":10,"unit":"ms"}},"gain_c":{"type":"Gain","parameters":{"gain":1.0,"inverted":false}},"gain_l":{"type":"Gain","parameters":{"gain":1.0,"inverted":false}},"gain_r":{"type":"Gain","parameters":{"gain":1.0,"inverted":false}},"gain_sub1":{"type":"Gain","parameters":{"gain":1.0,"inverted":false}},"peq_l_0":{"type":"Biquad","parameters":{"freq":1000,"q":0.707,"gain":2.0,"type":"Peaking"}},"peq_l_1":{"type":"Biquad","parameters":{"freq":2000,"q":0.707,"gain":2.0,"type":"Peaking"}},"peq_r_2":{"type":"Biquad","parameters":{"freq":2000,"q":0.707,"gain":1.0,"type":"Peaking"}}},"pipeline":[{"type":"Mixer","name":"split_non_sub"},{"type":"Filter","channel":2,"names":["crossover_speaker_c"]},{"type":"Filter","channel":3,"names":["crossover_subwooferc"]},{"type":"Filter","channel":0,"names":["crossover_speaker_l"]},{"type":"Filter","channel":1,"names":["crossover_subwooferl"]},{"type":"Filter","channel":4,"names":["crossover_speaker_r"]},{"type":"Filter","channel":5,"names":["crossover_subwooferr"]},{"type":"Mixer","name":"combine_sub"},{"type":"Filter","channel":0,"names":["peq_l_0","peq_l_1","delay_l","gain_l"]},{"type":"Filter","channel":1,"names":["delay_c","gain_c"]},{"type":"Filter","channel":2,"names":["peq_r_2","delay_r","gain_r"]},{"type":"Filter","channel":3,"names":["delay_sub1","gain_sub1"]}]}"#
        )
    }

    #[test]
    fn check_processor_to_camilla_two_sub() {
        let settings = ProcessorSettings {
            filters: vec![
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
            ],
            speakers: vec![
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
                    crossover: None,
                    delay: 10,
                    gain: 1.0,
                    is_subwoofer: true,
                },
                Speaker {
                    speaker: "sub2".to_string(),
                    crossover: None,
                    delay: 10,
                    gain: 1.0,
                    is_subwoofer: true,
                },
            ],
        };

        assert_eq!(
            convert_processor_settings_to_camilla(&settings).unwrap(),
            r#"{"mixers":{"combine_sub":{"channels":{"in":7,"out":5},"mapping":[{"sources":[{"channel":2,"gain":0,"inverted":false}],"dest":1},{"sources":[{"channel":0,"gain":0,"inverted":false}],"dest":0},{"sources":[{"channel":4,"gain":0,"inverted":false}],"dest":2},{"sources":[{"channel":1,"gain":0,"inverted":false},{"channel":3,"gain":0,"inverted":false},{"channel":5,"gain":0,"inverted":false},{"channel":6,"gain":0,"inverted":false}],"dest":3},{"sources":[{"channel":1,"gain":0,"inverted":false},{"channel":3,"gain":0,"inverted":false},{"channel":5,"gain":0,"inverted":false},{"channel":6,"gain":0,"inverted":false}],"dest":4}]},"split_non_sub":{"channels":{"in":4,"out":7},"mapping":[{"sources":[{"channel":1,"gain":0,"inverted":false}],"dest":2},{"sources":[{"channel":1,"gain":0,"inverted":false}],"dest":3},{"sources":[{"channel":0,"gain":0,"inverted":false}],"dest":0},{"sources":[{"channel":0,"gain":0,"inverted":false}],"dest":1},{"sources":[{"channel":2,"gain":0,"inverted":false}],"dest":4},{"sources":[{"channel":2,"gain":0,"inverted":false}],"dest":5},{"sources":[{"channel":3,"gain":0,"inverted":false}],"dest":6}]}},"filters":{"crossover_speaker_c":{"type":"BiquadCombo","parameters":{"freq":80,"order":4,"type":"ButterworthHighpass"}},"crossover_speaker_l":{"type":"BiquadCombo","parameters":{"freq":80,"order":4,"type":"ButterworthHighpass"}},"crossover_speaker_r":{"type":"BiquadCombo","parameters":{"freq":80,"order":4,"type":"ButterworthHighpass"}},"crossover_subwooferc":{"type":"BiquadCombo","parameters":{"freq":80,"order":4,"type":"ButterworthLowpass"}},"crossover_subwooferl":{"type":"BiquadCombo","parameters":{"freq":80,"order":4,"type":"ButterworthLowpass"}},"crossover_subwooferr":{"type":"BiquadCombo","parameters":{"freq":80,"order":4,"type":"ButterworthLowpass"}},"delay_c":{"type":"Delay","parameters":{"delay":10,"unit":"ms"}},"delay_l":{"type":"Delay","parameters":{"delay":10,"unit":"ms"}},"delay_r":{"type":"Delay","parameters":{"delay":10,"unit":"ms"}},"delay_sub1":{"type":"Delay","parameters":{"delay":10,"unit":"ms"}},"delay_sub2":{"type":"Delay","parameters":{"delay":10,"unit":"ms"}},"gain_c":{"type":"Gain","parameters":{"gain":1.0,"inverted":false}},"gain_l":{"type":"Gain","parameters":{"gain":1.0,"inverted":false}},"gain_r":{"type":"Gain","parameters":{"gain":1.0,"inverted":false}},"gain_sub1":{"type":"Gain","parameters":{"gain":1.0,"inverted":false}},"gain_sub2":{"type":"Gain","parameters":{"gain":1.0,"inverted":false}},"peq_l_0":{"type":"Biquad","parameters":{"freq":1000,"q":0.707,"gain":2.0,"type":"Peaking"}},"peq_l_1":{"type":"Biquad","parameters":{"freq":2000,"q":0.707,"gain":2.0,"type":"Peaking"}},"peq_r_2":{"type":"Biquad","parameters":{"freq":2000,"q":0.707,"gain":1.0,"type":"Peaking"}}},"pipeline":[{"type":"Mixer","name":"split_non_sub"},{"type":"Filter","channel":2,"names":["crossover_speaker_c"]},{"type":"Filter","channel":3,"names":["crossover_subwooferc"]},{"type":"Filter","channel":0,"names":["crossover_speaker_l"]},{"type":"Filter","channel":1,"names":["crossover_subwooferl"]},{"type":"Filter","channel":4,"names":["crossover_speaker_r"]},{"type":"Filter","channel":5,"names":["crossover_subwooferr"]},{"type":"Mixer","name":"combine_sub"},{"type":"Filter","channel":0,"names":["peq_l_0","peq_l_1","delay_l","gain_l"]},{"type":"Filter","channel":1,"names":["delay_c","gain_c"]},{"type":"Filter","channel":2,"names":["peq_r_2","delay_r","gain_r"]},{"type":"Filter","channel":3,"names":["delay_sub1","gain_sub1"]},{"type":"Filter","channel":4,"names":["delay_sub2","gain_sub2"]}]}"#
        )
    }
    #[test]
    fn check_processor_to_camilla_two_sub_no_crossover() {
        let settings = ProcessorSettings {
            filters: vec![
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
            ],
            speakers: vec![
                Speaker {
                    speaker: "l".to_string(),
                    crossover: None,
                    delay: 10,
                    gain: 1.0,
                    is_subwoofer: false,
                },
                Speaker {
                    speaker: "c".to_string(),
                    crossover: None,
                    delay: 10,
                    gain: 1.0,
                    is_subwoofer: false,
                },
                Speaker {
                    speaker: "r".to_string(),
                    crossover: None,
                    delay: 10,
                    gain: 1.0,
                    is_subwoofer: false,
                },
                Speaker {
                    speaker: "sub1".to_string(),
                    crossover: None,
                    delay: 10,
                    gain: 1.0,
                    is_subwoofer: true,
                },
                Speaker {
                    speaker: "sub2".to_string(),
                    crossover: None,
                    delay: 10,
                    gain: 1.0,
                    is_subwoofer: true,
                },
            ],
        };

        assert_eq!(
            convert_processor_settings_to_camilla(&settings).unwrap(),
            r#"{"mixers":{"combine_sub":{"channels":{"in":4,"out":5},"mapping":[{"sources":[{"channel":1,"gain":0,"inverted":false}],"dest":1},{"sources":[{"channel":0,"gain":0,"inverted":false}],"dest":0},{"sources":[{"channel":2,"gain":0,"inverted":false}],"dest":2},{"sources":[{"channel":3,"gain":0,"inverted":false}],"dest":3},{"sources":[{"channel":3,"gain":0,"inverted":false}],"dest":4}]},"split_non_sub":{"channels":{"in":4,"out":4},"mapping":[{"sources":[{"channel":1,"gain":0,"inverted":false}],"dest":1},{"sources":[{"channel":0,"gain":0,"inverted":false}],"dest":0},{"sources":[{"channel":2,"gain":0,"inverted":false}],"dest":2},{"sources":[{"channel":3,"gain":0,"inverted":false}],"dest":3}]}},"filters":{"delay_c":{"type":"Delay","parameters":{"delay":10,"unit":"ms"}},"delay_l":{"type":"Delay","parameters":{"delay":10,"unit":"ms"}},"delay_r":{"type":"Delay","parameters":{"delay":10,"unit":"ms"}},"delay_sub1":{"type":"Delay","parameters":{"delay":10,"unit":"ms"}},"delay_sub2":{"type":"Delay","parameters":{"delay":10,"unit":"ms"}},"gain_c":{"type":"Gain","parameters":{"gain":1.0,"inverted":false}},"gain_l":{"type":"Gain","parameters":{"gain":1.0,"inverted":false}},"gain_r":{"type":"Gain","parameters":{"gain":1.0,"inverted":false}},"gain_sub1":{"type":"Gain","parameters":{"gain":1.0,"inverted":false}},"gain_sub2":{"type":"Gain","parameters":{"gain":1.0,"inverted":false}},"peq_l_0":{"type":"Biquad","parameters":{"freq":1000,"q":0.707,"gain":2.0,"type":"Peaking"}},"peq_l_1":{"type":"Biquad","parameters":{"freq":2000,"q":0.707,"gain":2.0,"type":"Peaking"}},"peq_r_2":{"type":"Biquad","parameters":{"freq":2000,"q":0.707,"gain":1.0,"type":"Peaking"}}},"pipeline":[{"type":"Mixer","name":"split_non_sub"},{"type":"Mixer","name":"combine_sub"},{"type":"Filter","channel":0,"names":["peq_l_0","peq_l_1","delay_l","gain_l"]},{"type":"Filter","channel":1,"names":["delay_c","gain_c"]},{"type":"Filter","channel":2,"names":["peq_r_2","delay_r","gain_r"]},{"type":"Filter","channel":3,"names":["delay_sub1","gain_sub1"]},{"type":"Filter","channel":4,"names":["delay_sub2","gain_sub2"]}]}"#
        )
    }
    #[test]
    fn check_processor_to_camilla_two_sub_partial_crossover() {
        let settings = ProcessorSettings {
            filters: vec![
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
            ],
            speakers: vec![
                Speaker {
                    speaker: "l".to_string(),
                    crossover: Some(80),
                    delay: 10,
                    gain: 1.0,
                    is_subwoofer: false,
                },
                Speaker {
                    speaker: "c".to_string(),
                    crossover: None,
                    delay: 10,
                    gain: 1.0,
                    is_subwoofer: false,
                },
                Speaker {
                    speaker: "r".to_string(),
                    crossover: None,
                    delay: 10,
                    gain: 1.0,
                    is_subwoofer: false,
                },
                Speaker {
                    speaker: "sub1".to_string(),
                    crossover: None,
                    delay: 10,
                    gain: 1.0,
                    is_subwoofer: true,
                },
                Speaker {
                    speaker: "sub2".to_string(),
                    crossover: None,
                    delay: 10,
                    gain: 1.0,
                    is_subwoofer: true,
                },
            ],
        };

        assert_eq!(
            convert_processor_settings_to_camilla(&settings).unwrap(),
            r#"{"mixers":{"combine_sub":{"channels":{"in":5,"out":5},"mapping":[{"sources":[{"channel":2,"gain":0,"inverted":false}],"dest":1},{"sources":[{"channel":0,"gain":0,"inverted":false}],"dest":0},{"sources":[{"channel":3,"gain":0,"inverted":false}],"dest":2},{"sources":[{"channel":1,"gain":0,"inverted":false},{"channel":4,"gain":0,"inverted":false}],"dest":3},{"sources":[{"channel":1,"gain":0,"inverted":false},{"channel":4,"gain":0,"inverted":false}],"dest":4}]},"split_non_sub":{"channels":{"in":4,"out":5},"mapping":[{"sources":[{"channel":1,"gain":0,"inverted":false}],"dest":2},{"sources":[{"channel":0,"gain":0,"inverted":false}],"dest":0},{"sources":[{"channel":0,"gain":0,"inverted":false}],"dest":1},{"sources":[{"channel":2,"gain":0,"inverted":false}],"dest":3},{"sources":[{"channel":3,"gain":0,"inverted":false}],"dest":4}]}},"filters":{"crossover_speaker_l":{"type":"BiquadCombo","parameters":{"freq":80,"order":4,"type":"ButterworthHighpass"}},"crossover_subwooferl":{"type":"BiquadCombo","parameters":{"freq":80,"order":4,"type":"ButterworthLowpass"}},"delay_c":{"type":"Delay","parameters":{"delay":10,"unit":"ms"}},"delay_l":{"type":"Delay","parameters":{"delay":10,"unit":"ms"}},"delay_r":{"type":"Delay","parameters":{"delay":10,"unit":"ms"}},"delay_sub1":{"type":"Delay","parameters":{"delay":10,"unit":"ms"}},"delay_sub2":{"type":"Delay","parameters":{"delay":10,"unit":"ms"}},"gain_c":{"type":"Gain","parameters":{"gain":1.0,"inverted":false}},"gain_l":{"type":"Gain","parameters":{"gain":1.0,"inverted":false}},"gain_r":{"type":"Gain","parameters":{"gain":1.0,"inverted":false}},"gain_sub1":{"type":"Gain","parameters":{"gain":1.0,"inverted":false}},"gain_sub2":{"type":"Gain","parameters":{"gain":1.0,"inverted":false}},"peq_l_0":{"type":"Biquad","parameters":{"freq":1000,"q":0.707,"gain":2.0,"type":"Peaking"}},"peq_l_1":{"type":"Biquad","parameters":{"freq":2000,"q":0.707,"gain":2.0,"type":"Peaking"}},"peq_r_2":{"type":"Biquad","parameters":{"freq":2000,"q":0.707,"gain":1.0,"type":"Peaking"}}},"pipeline":[{"type":"Mixer","name":"split_non_sub"},{"type":"Filter","channel":0,"names":["crossover_speaker_l"]},{"type":"Filter","channel":1,"names":["crossover_subwooferl"]},{"type":"Mixer","name":"combine_sub"},{"type":"Filter","channel":0,"names":["peq_l_0","peq_l_1","delay_l","gain_l"]},{"type":"Filter","channel":1,"names":["delay_c","gain_c"]},{"type":"Filter","channel":2,"names":["peq_r_2","delay_r","gain_r"]},{"type":"Filter","channel":3,"names":["delay_sub1","gain_sub1"]},{"type":"Filter","channel":4,"names":["delay_sub2","gain_sub2"]}]}"#
        )
    }
}
