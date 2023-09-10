#[macro_use]
extern crate rocket;
use chrono::Utc;
use rocket::fairing::{self, AdHoc};
use rocket::response::status::BadRequest;
use rocket::serde::{json, json::Json, Serialize};
use rocket::{Build, Rocket};
use rocket_db_pools::sqlx::{self};
use rocket_db_pools::{Connection, Database};
use std::collections::HashMap;

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

#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
struct CamillaConfig {
    mixers: HashMap<String, Mixer>,
    filters: HashMap<String, SpeakerAdjust>,
    pipeline: Vec<Pipeline>,
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
            let mut pipeline = create_crossover_pipeline(
                split_mixer_name(),
                combine_mixer_name(),
                &crossover_channels,
                &settings.speakers,
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
                mixers: HashMap::new(),
            };
            json::to_string(&result)
        }
    }
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
    use super::convert_processor_settings_to_camilla;
    use crate::processor::ProcessorSettings;
    use crate::processor::{Filter, Speaker};
    #[test]
    fn check_processor_to_camilla() {
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
        println!(
            "Yaml {}",
            convert_processor_settings_to_camilla(&settings).unwrap()
        );
        //let result=convert_processor_settings_to_camilla()
    }
}
