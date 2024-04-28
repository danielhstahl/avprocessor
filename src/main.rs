#[macro_use]
extern crate rocket;
use chrono::Utc;
use rocket::fairing::{self, AdHoc};
use rocket::fs::{relative, FileServer};
use rocket::response::status::BadRequest;
use rocket::serde::{json, json::Json, Serialize};
use rocket::{Build, Rocket, State};
use rocket_db_pools::sqlx::{self};
use rocket_db_pools::{Connection, Database};
use std::collections::BTreeMap;
use tungstenite::{connect, Message};
use url::Url;

mod devices;
mod filters;
mod mixers;
mod pipeline;
mod processor;

use devices::Devices;
use filters::{compute_peq_filter, create_crossover_filters, create_output_filters, SpeakerAdjust};
use mixers::{
    combine_inputs, combine_mixer_name, get_speaker_counts, input_speaker_count,
    output_speaker_count_no_mixer, split_inputs, split_mixer_name, Mixer, SpeakerCounts,
};
use pipeline::{create_crossover_pipeline, create_per_speaker_pipeline, Pipeline};
use processor::{
    DeviceType, Filter, ProcessorSettings, ProcessorSettingsForCamilla, SelectedDistanceType,
    Speaker, SpeakerForUI,
};

use crate::pipeline::create_per_speaker_pipeline_no_mixer;

#[derive(Database)]
#[database("settings")]
/// Wrapper for database configuration
struct Settings(sqlx::SqlitePool);

#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
/// Represents a valid Camilla configuration, see https://github.com/HEnquist/camilladsp/tree/master/exampleconfigs for examples
struct CamillaConfig {
    mixers: BTreeMap<String, Mixer>,
    filters: BTreeMap<String, SpeakerAdjust>,
    pipeline: Vec<Pipeline>,
    devices: Devices,
}

#[derive(Serialize, sqlx::FromRow)]
#[serde(crate = "rocket::serde", rename_all = "camelCase")]
/// Used in the UI; represents versions of the configuration
struct Version {
    version: i32,
    applied_version: bool,
    version_date: String,
}

#[derive(sqlx::FromRow)]
/// Wrapper to extract version from sqlx macro
struct ConfigVersion {
    version: i32,
}

#[derive(sqlx::FromRow)]
/// Wrapper to extract distance from sqlx macro
struct SelectedDistanceAndDevice {
    selected_distance: SelectedDistanceType,
    device: DeviceType,
}

#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
/// Wrapper for camilla configuration over websockets using JSON.  See https://github.com/HEnquist/camilladsp/blob/master/websocket.md#config-management
struct SetConfig {
    #[serde(rename = "SetConfigJson")]
    set_config_json: String,
}

/// this is used purely to store state and pass to mixer and filter creators
struct ConfigurationMapping<'a> {
    peq_filters: BTreeMap<&'a String, Vec<(usize, &'a Filter)>>,
    speaker_counts: SpeakerCounts,
}

struct CamillaSettings {
    websocket_url: String,
    //config_file_location: String,
}

/// runs on every startup, idempotent table creation
async fn run_migrations(rocket: Rocket<Build>) -> fairing::Result {
    match Settings::fetch(&rocket) {
        Some(db) => match sqlx::migrate!("db/migrations").run(&**db).await {
            Ok(_) => Ok(rocket),
            Err(e) => {
                error!("Failed to initialize SQLx database: {}", e);
                Err(rocket)
            }
        },
        None => Err(rocket),
    }
}

/// settings stored in sqlite are converted to the appropriate camilla configuration
fn convert_processor_settings_to_camilla(
    settings: &ProcessorSettingsForCamilla,
) -> Result<CamillaConfig, json::serde_json::Error> {
    let configuration_mapping = ConfigurationMapping {
        peq_filters: compute_peq_filter(&settings.filters),
        speaker_counts: get_speaker_counts(&settings.speakers),
    };

    let split_mixer = split_inputs(&settings.speakers, &configuration_mapping.speaker_counts);
    let output_filters =
        create_output_filters(&settings.speakers, &configuration_mapping.peq_filters);

    match split_mixer {
        Some((split_mixer, input_channel_mapping, output_channel_mapping)) => {
            let combine_mixer = combine_inputs(
                &configuration_mapping.speaker_counts,
                &split_mixer,
                &output_channel_mapping,
            );
            let mut per_speaker_pipeline = create_per_speaker_pipeline(
                &output_channel_mapping,
                &configuration_mapping.peq_filters,
            );

            let devices = match settings.device {
                DeviceType::MotuMk5 => Devices::motu_mk5(
                    split_mixer.channels.num_in_channel,
                    combine_mixer.channels.num_out_channel,
                ),
                DeviceType::OktoDac8 => Devices::okto_dac8(
                    split_mixer.channels.num_in_channel,
                    combine_mixer.channels.num_out_channel,
                ),
                DeviceType::ToppingDm7 => Devices::topping_dm7(
                    split_mixer.channels.num_in_channel,
                    combine_mixer.channels.num_out_channel,
                ),
                DeviceType::HDMI => Devices::hdmi(
                    split_mixer.channels.num_in_channel,
                    combine_mixer.channels.num_out_channel,
                ),
                DeviceType::HDMIMac => Devices::hdmi_mac(
                    split_mixer.channels.num_in_channel,
                    combine_mixer.channels.num_out_channel,
                ),
            };

            let mixers: BTreeMap<String, Mixer> = BTreeMap::from_iter(
                vec![
                    (split_mixer_name(), split_mixer),
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
                devices,
            };
            Ok(result)
        }
        None => {
            let input_channels = input_speaker_count(&configuration_mapping.speaker_counts);
            let output_channels =
                output_speaker_count_no_mixer(&configuration_mapping.speaker_counts);
            let per_speaker_pipeline = create_per_speaker_pipeline_no_mixer(
                &settings.speakers,
                &configuration_mapping.peq_filters,
            );
            let result = CamillaConfig {
                pipeline: per_speaker_pipeline,
                filters: output_filters,
                mixers: BTreeMap::new(),
                devices: match settings.device {
                    DeviceType::MotuMk5 => Devices::motu_mk5(input_channels, output_channels),
                    DeviceType::OktoDac8 => Devices::okto_dac8(input_channels, output_channels),
                    DeviceType::ToppingDm7 => Devices::topping_dm7(input_channels, output_channels),
                    DeviceType::HDMI => Devices::hdmi(input_channels, output_channels),
                    DeviceType::HDMIMac => Devices::hdmi_mac(input_channels, output_channels),
                },
            };
            Ok(result)
        }
    }
}

/// reads selected distance type (MS, FEET, METERS) for the specific configration version
async fn get_selected_distance_and_device(
    mut db: Connection<Settings>,
    version: i32,
) -> Result<(SelectedDistanceAndDevice, Connection<Settings>), sqlx::Error> {
    let selected_distance_and_device = sqlx::query_as!(
        SelectedDistanceAndDevice,
        r#"SELECT 
            selected_distance as "selected_distance: crate::processor::SelectedDistanceType",
            device as "device: crate::processor::DeviceType"
            from versions where version=?"#,
        version
    )
    .fetch_one(&mut **db)
    .await?;
    Ok((selected_distance_and_device, db))
}

/// reads filters for the specific configration version
async fn get_filters(
    mut db: Connection<Settings>,
    version: i32,
) -> Result<(Vec<Filter>, Connection<Settings>), sqlx::Error> {
    let filters = sqlx::query_as!(
        Filter,
        r#"SELECT speaker, 
        freq as "freq: i32", 
        gain as "gain: f32", 
        q as "q: f32"
        from filters where version=?"#,
        version
    )
    .fetch_all(&mut **db)
    .await?;
    Ok((filters, db))
}

/// reads speakers that map to camilla speakers for the specific configration version
async fn get_speakers_for_camilla(
    mut db: Connection<Settings>,
    version: i32,
) -> Result<(Vec<Speaker>, Connection<Settings>), sqlx::Error> {
    let speakers = sqlx::query_as!(
        Speaker,
        r#"SELECT 
        speaker, 
        crossover as "crossover: i32", 
        delay as "delay: f32", 
        gain as "gain: f32", 
        is_subwoofer as "is_subwoofer: bool"
        from speakers_for_camilla where version=?"#,
        version
    )
    .fetch_all(&mut **db)
    .await?;
    Ok((speakers, db))
}

/// reads speakers that map to the UI for the specific configration version
async fn get_speakers_for_ui(
    mut db: Connection<Settings>,
    version: i32,
) -> Result<Vec<SpeakerForUI>, sqlx::Error> {
    let speakers = sqlx::query_as!(
        SpeakerForUI,
        r#"
        SELECT 
        speaker, 
        crossover as "crossover?: i32", 
        distance as "distance: f32", 
        gain as "gain: f32", 
        is_subwoofer as "is_subwoofer: bool"
        from speakers_settings_for_ui where version=?"#,
        version,
    )
    .fetch_all(&mut **db)
    .await?;
    Ok(speakers)
}

/// gets configuration for camilla from database
async fn get_config_for_camilla_from_db(
    db: Connection<Settings>,
    version: i32,
) -> Result<(ProcessorSettingsForCamilla, Connection<Settings>), sqlx::Error> {
    let (SelectedDistanceAndDevice { device, .. }, db) =
        get_selected_distance_and_device(db, version).await?;
    let (filters, db) = get_filters(db, version).await?;
    let (speakers, db) = get_speakers_for_camilla(db, version).await?;
    Ok((
        ProcessorSettingsForCamilla {
            filters,
            speakers,
            device,
        },
        db,
    ))
}

/// gets configuration for UI from database
async fn get_config_from_db(
    db: Connection<Settings>,
    version: i32,
) -> Result<ProcessorSettings, sqlx::Error> {
    let (
        SelectedDistanceAndDevice {
            selected_distance,
            device,
        },
        db,
    ) = get_selected_distance_and_device(db, version).await?;
    let (filters, db) = get_filters(db, version).await?;
    let speakers = get_speakers_for_ui(db, version).await?;
    Ok(ProcessorSettings {
        filters,
        speakers,
        selected_distance,
        device,
    })
}

const METERS_PER_MS: f32 = 0.3430;
const FEET_PER_MS: f32 = 1.1164;

/// given the speaker with the largest distance,
/// the current speakers distance, and the speed of sound,
/// gets the number of millisecond delay
fn convert_distance_to_delay(
    largest_distance: f32,
    current_distance: f32,
    distance_per_ms: f32,
) -> f32 {
    (largest_distance - current_distance) / distance_per_ms
}

/// apply delays to each speaker given the distance type (MS, METERS, FEET)
fn update_speaker_delays(
    selected_distance: &SelectedDistanceType,
    speakers: &[SpeakerForUI],
) -> Vec<Speaker> {
    let max_distance =
        speakers.iter().fold(
            0.0,
            |accum: f32, speaker: &SpeakerForUI| match selected_distance {
                SelectedDistanceType::MS => 0.0,
                _ => {
                    if accum < speaker.distance {
                        speaker.distance
                    } else {
                        accum
                    }
                }
            },
        );
    speakers
        .iter()
        .map(|speaker| match selected_distance {
            SelectedDistanceType::METERS => Speaker {
                speaker: speaker.speaker.clone(), //Hate doing this, but no great ways to have two arrays sharing same string reference.  Using &' str errors on the sqlx macro
                crossover: speaker.crossover,
                delay: convert_distance_to_delay(max_distance, speaker.distance, METERS_PER_MS),
                gain: speaker.gain,
                is_subwoofer: speaker.is_subwoofer,
            },
            SelectedDistanceType::FEET => Speaker {
                speaker: speaker.speaker.clone(), //Hate doing this, but no great ways to have two arrays sharing same string reference.  Using &' str errors on the sqlx macro
                crossover: speaker.crossover,
                delay: convert_distance_to_delay(max_distance, speaker.distance, FEET_PER_MS),
                gain: speaker.gain,
                is_subwoofer: speaker.is_subwoofer,
            },
            SelectedDistanceType::MS => Speaker {
                speaker: speaker.speaker.clone(), //Hate doing this, but no great ways to have two arrays sharing same string reference.  Using &' str errors on the sqlx macro
                crossover: speaker.crossover,
                delay: speaker.distance,
                gain: speaker.gain,
                is_subwoofer: speaker.is_subwoofer,
            },
        })
        .collect()
}

#[get("/versions")]
async fn get_versions(
    mut db: Connection<Settings>,
) -> Result<Json<Vec<Version>>, BadRequest<String>> {
    let versions = sqlx::query_as!(
        Version,
        r#"
        SELECT 
        t1.version as "version: i32", 
        t1.version_date,
        case when 
            t2.version is null then false 
            else true 
        end as "applied_version: bool"
        FROM versions t1 
        left join applied_version t2 
        on t1.version=t2.version
        "#,
    )
    .fetch_all(&mut **db)
    .await
    .map_err(|e| BadRequest(e.to_string()))?;

    Ok(Json(versions))
}
#[get("/config/latest")]
async fn config_latest(
    mut db: Connection<Settings>,
) -> Result<Json<ProcessorSettings>, BadRequest<String>> {
    let ConfigVersion { version } = sqlx::query_as!(
        ConfigVersion,
        r#"SELECT max(version) as "version!: i32"  from versions"#
    )
    .fetch_one(&mut **db)
    .await
    .map_err(|e| BadRequest(e.to_string()))?;
    get_config_from_db(db, version)
        .await
        .map(|v| Json(v))
        .map_err(|e| BadRequest(e.to_string()))
}

#[get("/config/<version>")]
async fn config_version(
    db: Connection<Settings>,
    version: i32,
) -> Result<Json<ProcessorSettings>, BadRequest<String>> {
    get_config_from_db(db, version)
        .await
        .map(|v| Json(v))
        .map_err(|e| BadRequest(e.to_string()))
}

//TODO, run this on startup as well, to set initial config in camilladsp
#[post("/config/apply/<version>", format = "application/json")]
/// Configurations can be saved without actually be implemented or applied to camilla.  
/// This endpoint applies the selected version to camilla
async fn apply_config_version(
    db: Connection<Settings>,
    version: i32,
    camilla_settings: &State<CamillaSettings>,
) -> Result<(), BadRequest<String>> {
    let (settings, mut db) = get_config_for_camilla_from_db(db, version)
        .await
        .map_err(|e| BadRequest(e.to_string()))?;
    let config =
        convert_processor_settings_to_camilla(&settings).map_err(|e| BadRequest(e.to_string()))?;

    let config_as_str = json::to_string(&config).map_err(|e| BadRequest(e.to_string()))?;

    let ws_url =
        Url::parse(&camilla_settings.websocket_url).map_err(|e| BadRequest(e.to_string()))?;
    let (mut socket, _response) = connect(ws_url).map_err(|e| BadRequest(e.to_string()))?;
    let config_as_json = json::to_string(&SetConfig {
        set_config_json: config_as_str,
    })
    .map_err(|e| BadRequest(e.to_string()))?;
    socket
        .send(Message::Text(config_as_json))
        .map_err(|e| BadRequest(e.to_string()))?;

    let _ = sqlx::query!("DELETE from applied_version")
        .execute(&mut **db)
        .await
        .map_err(|e| BadRequest(e.to_string()))?;
    let _ = sqlx::query!("INSERT INTO applied_version (version) VALUES (?)", version)
        .execute(&mut **db)
        .await
        .map_err(|e| BadRequest(e.to_string()))?;

    Ok(())
}

#[put("/config", format = "application/json", data = "<settings>")]
/// Saves the configuration and auto-increments the version.
/// Does NOT apply the configuration to Camilla.
async fn write_configuration(
    mut db: Connection<Settings>,
    settings: Json<ProcessorSettings>,
) -> Result<Json<Version>, BadRequest<String>> {
    let version_date = Utc::now().to_string();
    let ConfigVersion { version } = sqlx::query_as!(
        ConfigVersion,
        r#"INSERT INTO versions (
            version_date, selected_distance, device
        ) VALUES (?, ?, ?) RETURNING version as "version: i32""#,
        version_date,
        settings.selected_distance,
        settings.device
    )
    .fetch_one(&mut **db)
    .await
    .map_err(|e| BadRequest(e.to_string()))?;

    for (index, filter) in settings.filters.iter().enumerate() {
        let index_i32 = index as i32;
        let _ = sqlx::query!(
            "INSERT INTO filters (version, filter_index, speaker, freq, gain, q) VALUES (?, ?, ?, ?, ?, ?)",
            version, index_i32, filter.speaker, filter.freq, filter.gain, filter.q
        )
        .execute(&mut **db)
        .await;
    }
    for speaker in settings.speakers.iter() {
        let _ = sqlx::query!(
            "INSERT INTO speakers_settings_for_ui (
            version, 
            speaker, 
            crossover, 
            distance, 
            gain, 
            is_subwoofer
        ) VALUES (?, ?, ?, ?, ?, ?)",
            version,
            speaker.speaker,
            speaker.crossover,
            speaker.distance,
            speaker.gain,
            speaker.is_subwoofer
        )
        .execute(&mut **db)
        .await;
    }
    let speakers = update_speaker_delays(&settings.selected_distance, &settings.speakers);
    for speaker in speakers.iter() {
        let _ = sqlx::query!(
            "INSERT INTO speakers_for_camilla (
                version, 
                speaker, 
                crossover, 
                delay, 
                gain, 
                is_subwoofer
            ) VALUES (?, ?, ?, ?, ?, ?)",
            version,
            speaker.speaker,
            speaker.crossover,
            speaker.delay,
            speaker.gain,
            speaker.is_subwoofer
        )
        .execute(&mut **db)
        .await;
    }
    Ok(Json(Version {
        version,
        applied_version: false,
        version_date,
    }))
}

#[delete("/config/<version>")]
async fn delete_configuration(
    mut db: Connection<Settings>,
    version: i32,
) -> Result<(), BadRequest<String>> {
    let _ = sqlx::query!("DELETE FROM versions WHERE version=?", version)
        .execute(&mut **db)
        .await;
    let _ = sqlx::query!("DELETE FROM filters WHERE version=?", version)
        .execute(&mut **db)
        .await;
    let _ = sqlx::query!(
        "DELETE FROM speakers_settings_for_ui WHERE version=?",
        version
    )
    .execute(&mut **db)
    .await;
    let _ = sqlx::query!("DELETE FROM speakers_for_camilla WHERE version=?", version)
        .execute(&mut **db)
        .await;

    let _ = sqlx::query!("DELETE FROM applied_version WHERE version=?", version)
        .execute(&mut **db)
        .await;
    Ok(())
}

#[launch]
fn rocket() -> _ {
    let mut args = std::env::args();
    args.next(); //first item is the app name, skip it

    //Websocket URL
    let websocket_url = args.next().unwrap_or("ws://127.0.0.1:1234".to_string());

    let camilla_settings = CamillaSettings { websocket_url };
    let html_files = args
        .next()
        .unwrap_or(relative!("avprocessor-ui/build").to_string());
    rocket::build()
        .mount("/", FileServer::from(html_files))
        .manage(camilla_settings)
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
    use super::update_speaker_delays;
    use super::FEET_PER_MS;
    use super::METERS_PER_MS;
    use crate::processor::{
        DeviceType, Filter, ProcessorSettingsForCamilla, SelectedDistanceType, Speaker,
        SpeakerForUI,
    };
    use rocket::serde::json;

    #[test]
    fn test_update_speaker_delays_meters() {
        let speakers: Vec<SpeakerForUI> = vec![
            SpeakerForUI {
                speaker: "l".to_string(),
                crossover: Some(80),
                distance: 0.0,
                gain: 1.0,
                is_subwoofer: false,
            },
            SpeakerForUI {
                speaker: "r".to_string(),
                crossover: Some(80),
                distance: 0.0,
                gain: 1.0,
                is_subwoofer: false,
            },
            SpeakerForUI {
                speaker: "c".to_string(),
                crossover: Some(80),
                distance: 1.0,
                gain: 1.0,
                is_subwoofer: false,
            },
            SpeakerForUI {
                speaker: "sub1".to_string(),
                crossover: None,
                distance: 3.0,
                gain: 1.0,
                is_subwoofer: true,
            },
            SpeakerForUI {
                speaker: "sub2".to_string(),
                crossover: None,
                distance: 3.0,
                gain: 1.0,
                is_subwoofer: true,
            },
        ];
        let speakers = update_speaker_delays(&SelectedDistanceType::METERS, &speakers);
        assert_eq!(speakers[0].delay, 3.0 / METERS_PER_MS);
        assert_eq!(speakers[1].delay, 3.0 / METERS_PER_MS);
        assert_eq!(speakers[2].delay, 2.0 / METERS_PER_MS);
        assert_eq!(speakers[3].delay, 0.0 / METERS_PER_MS);
        assert_eq!(speakers[4].delay, 0.0 / METERS_PER_MS);
    }

    #[test]
    fn test_update_speaker_delays_feet() {
        let speakers: Vec<SpeakerForUI> = vec![
            SpeakerForUI {
                speaker: "l".to_string(),
                crossover: Some(80),
                distance: 0.0,
                gain: 1.0,
                is_subwoofer: false,
            },
            SpeakerForUI {
                speaker: "r".to_string(),
                crossover: Some(80),
                distance: 0.0,
                gain: 1.0,
                is_subwoofer: false,
            },
            SpeakerForUI {
                speaker: "c".to_string(),
                crossover: Some(80),
                distance: 1.0,
                gain: 1.0,
                is_subwoofer: false,
            },
            SpeakerForUI {
                speaker: "sub1".to_string(),
                crossover: None,
                distance: 3.0,
                gain: 1.0,
                is_subwoofer: true,
            },
            SpeakerForUI {
                speaker: "sub2".to_string(),
                crossover: None,
                distance: 3.0,
                gain: 1.0,
                is_subwoofer: true,
            },
        ];
        let speakers = update_speaker_delays(&SelectedDistanceType::FEET, &speakers);
        assert_eq!(speakers[0].delay, 3.0 / FEET_PER_MS);
        assert_eq!(speakers[1].delay, 3.0 / FEET_PER_MS);
        assert_eq!(speakers[2].delay, 2.0 / FEET_PER_MS);
        assert_eq!(speakers[3].delay, 0.0 / FEET_PER_MS);
        assert_eq!(speakers[4].delay, 0.0 / FEET_PER_MS);
    }
    #[test]
    fn test_update_speaker_delays_ms() {
        let speakers: Vec<SpeakerForUI> = vec![
            SpeakerForUI {
                speaker: "l".to_string(),
                crossover: Some(80),
                distance: 0.0,
                gain: 1.0,
                is_subwoofer: false,
            },
            SpeakerForUI {
                speaker: "r".to_string(),
                crossover: Some(80),
                distance: 0.0,
                gain: 1.0,
                is_subwoofer: false,
            },
            SpeakerForUI {
                speaker: "c".to_string(),
                crossover: Some(80),
                distance: 1.0,
                gain: 1.0,
                is_subwoofer: false,
            },
            SpeakerForUI {
                speaker: "sub1".to_string(),
                crossover: None,
                distance: 3.0,
                gain: 1.0,
                is_subwoofer: true,
            },
            SpeakerForUI {
                speaker: "sub2".to_string(),
                crossover: None,
                distance: 3.0,
                gain: 1.0,
                is_subwoofer: true,
            },
        ];
        let speakers = update_speaker_delays(&SelectedDistanceType::MS, &speakers);
        assert_eq!(speakers[0].delay, 0.0);
        assert_eq!(speakers[1].delay, 0.0);
        assert_eq!(speakers[2].delay, 1.0);
        assert_eq!(speakers[3].delay, 3.0);
        assert_eq!(speakers[4].delay, 3.0);
    }
    #[test]
    fn check_processor_to_camilla_one_sub() {
        let settings = ProcessorSettingsForCamilla {
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
                    crossover: None,
                    delay: 10.0,
                    gain: 1.0,
                    is_subwoofer: true,
                },
            ],
            //selected_distance: SelectedDistanceType::MS,
            device: DeviceType::OktoDac8,
        };
        assert_eq!(
            json::to_string(&convert_processor_settings_to_camilla(&settings).unwrap()).unwrap(),
            r#"{"mixers":{"combine_sub":{"channels":{"in":7,"out":4},"mapping":[{"sources":[{"channel":2,"gain":0,"inverted":false}],"dest":1},{"sources":[{"channel":0,"gain":0,"inverted":false}],"dest":0},{"sources":[{"channel":4,"gain":0,"inverted":false}],"dest":2},{"sources":[{"channel":1,"gain":0,"inverted":false},{"channel":3,"gain":0,"inverted":false},{"channel":5,"gain":0,"inverted":false},{"channel":6,"gain":0,"inverted":false}],"dest":3}]},"split_non_sub":{"channels":{"in":4,"out":7},"mapping":[{"sources":[{"channel":1,"gain":0,"inverted":false}],"dest":2},{"sources":[{"channel":1,"gain":0,"inverted":false}],"dest":3},{"sources":[{"channel":0,"gain":0,"inverted":false}],"dest":0},{"sources":[{"channel":0,"gain":0,"inverted":false}],"dest":1},{"sources":[{"channel":2,"gain":0,"inverted":false}],"dest":4},{"sources":[{"channel":2,"gain":0,"inverted":false}],"dest":5},{"sources":[{"channel":3,"gain":10,"inverted":false}],"dest":6}]}},"filters":{"crossover_speaker_c":{"type":"BiquadCombo","parameters":{"freq":80,"order":4,"type":"ButterworthHighpass"}},"crossover_speaker_l":{"type":"BiquadCombo","parameters":{"freq":80,"order":4,"type":"ButterworthHighpass"}},"crossover_speaker_r":{"type":"BiquadCombo","parameters":{"freq":80,"order":4,"type":"ButterworthHighpass"}},"crossover_subwooferc":{"type":"BiquadCombo","parameters":{"freq":80,"order":4,"type":"ButterworthLowpass"}},"crossover_subwooferl":{"type":"BiquadCombo","parameters":{"freq":80,"order":4,"type":"ButterworthLowpass"}},"crossover_subwooferr":{"type":"BiquadCombo","parameters":{"freq":80,"order":4,"type":"ButterworthLowpass"}},"delay_c":{"type":"Delay","parameters":{"delay":10.0,"unit":"ms"}},"delay_l":{"type":"Delay","parameters":{"delay":10.0,"unit":"ms"}},"delay_r":{"type":"Delay","parameters":{"delay":10.0,"unit":"ms"}},"delay_sub1":{"type":"Delay","parameters":{"delay":10.0,"unit":"ms"}},"gain_c":{"type":"Gain","parameters":{"gain":1.0,"inverted":false}},"gain_l":{"type":"Gain","parameters":{"gain":1.0,"inverted":false}},"gain_r":{"type":"Gain","parameters":{"gain":1.0,"inverted":false}},"gain_sub1":{"type":"Gain","parameters":{"gain":1.0,"inverted":false}},"peq_l_0":{"type":"Biquad","parameters":{"freq":1000,"q":0.707,"gain":2.0,"type":"Peaking"}},"peq_l_1":{"type":"Biquad","parameters":{"freq":2000,"q":0.707,"gain":2.0,"type":"Peaking"}},"peq_r_2":{"type":"Biquad","parameters":{"freq":2000,"q":0.707,"gain":1.0,"type":"Peaking"}}},"pipeline":[{"type":"Mixer","name":"split_non_sub"},{"type":"Filter","channel":2,"names":["crossover_speaker_c"]},{"type":"Filter","channel":3,"names":["crossover_subwooferc"]},{"type":"Filter","channel":0,"names":["crossover_speaker_l"]},{"type":"Filter","channel":1,"names":["crossover_subwooferl"]},{"type":"Filter","channel":4,"names":["crossover_speaker_r"]},{"type":"Filter","channel":5,"names":["crossover_subwooferr"]},{"type":"Mixer","name":"combine_sub"},{"type":"Filter","channel":1,"names":["delay_c","gain_c"]},{"type":"Filter","channel":0,"names":["peq_l_0","peq_l_1","delay_l","gain_l"]},{"type":"Filter","channel":2,"names":["peq_r_2","delay_r","gain_r"]},{"type":"Filter","channel":3,"names":["delay_sub1","gain_sub1"]}],"devices":{"samplerate":96000,"chunksize":2048,"queuelimit":1,"capture":{"type":"Alsa","channels":4,"device":"hw:Loopback,1","format":"S32LE"},"playback":{"type":"Alsa","channels":4,"device":"hw:DAC8PRO","format":"S32LE"}}}"#
        )
    }

    #[test]
    fn check_processor_to_camilla_two_sub() {
        let settings = ProcessorSettingsForCamilla {
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
                    crossover: None,
                    delay: 10.0,
                    gain: 1.0,
                    is_subwoofer: true,
                },
                Speaker {
                    speaker: "sub2".to_string(),
                    crossover: None,
                    delay: 10.0,
                    gain: 1.0,
                    is_subwoofer: true,
                },
            ],
            //selected_distance: SelectedDistanceType::MS,
            device: DeviceType::OktoDac8,
        };

        assert_eq!(
            json::to_string(&convert_processor_settings_to_camilla(&settings).unwrap()).unwrap(),
            r#"{"mixers":{"combine_sub":{"channels":{"in":7,"out":5},"mapping":[{"sources":[{"channel":2,"gain":0,"inverted":false}],"dest":1},{"sources":[{"channel":0,"gain":0,"inverted":false}],"dest":0},{"sources":[{"channel":4,"gain":0,"inverted":false}],"dest":2},{"sources":[{"channel":1,"gain":0,"inverted":false},{"channel":3,"gain":0,"inverted":false},{"channel":5,"gain":0,"inverted":false},{"channel":6,"gain":0,"inverted":false}],"dest":3},{"sources":[{"channel":1,"gain":0,"inverted":false},{"channel":3,"gain":0,"inverted":false},{"channel":5,"gain":0,"inverted":false},{"channel":6,"gain":0,"inverted":false}],"dest":4}]},"split_non_sub":{"channels":{"in":4,"out":7},"mapping":[{"sources":[{"channel":1,"gain":0,"inverted":false}],"dest":2},{"sources":[{"channel":1,"gain":0,"inverted":false}],"dest":3},{"sources":[{"channel":0,"gain":0,"inverted":false}],"dest":0},{"sources":[{"channel":0,"gain":0,"inverted":false}],"dest":1},{"sources":[{"channel":2,"gain":0,"inverted":false}],"dest":4},{"sources":[{"channel":2,"gain":0,"inverted":false}],"dest":5},{"sources":[{"channel":3,"gain":10,"inverted":false}],"dest":6}]}},"filters":{"crossover_speaker_c":{"type":"BiquadCombo","parameters":{"freq":80,"order":4,"type":"ButterworthHighpass"}},"crossover_speaker_l":{"type":"BiquadCombo","parameters":{"freq":80,"order":4,"type":"ButterworthHighpass"}},"crossover_speaker_r":{"type":"BiquadCombo","parameters":{"freq":80,"order":4,"type":"ButterworthHighpass"}},"crossover_subwooferc":{"type":"BiquadCombo","parameters":{"freq":80,"order":4,"type":"ButterworthLowpass"}},"crossover_subwooferl":{"type":"BiquadCombo","parameters":{"freq":80,"order":4,"type":"ButterworthLowpass"}},"crossover_subwooferr":{"type":"BiquadCombo","parameters":{"freq":80,"order":4,"type":"ButterworthLowpass"}},"delay_c":{"type":"Delay","parameters":{"delay":10.0,"unit":"ms"}},"delay_l":{"type":"Delay","parameters":{"delay":10.0,"unit":"ms"}},"delay_r":{"type":"Delay","parameters":{"delay":10.0,"unit":"ms"}},"delay_sub1":{"type":"Delay","parameters":{"delay":10.0,"unit":"ms"}},"delay_sub2":{"type":"Delay","parameters":{"delay":10.0,"unit":"ms"}},"gain_c":{"type":"Gain","parameters":{"gain":1.0,"inverted":false}},"gain_l":{"type":"Gain","parameters":{"gain":1.0,"inverted":false}},"gain_r":{"type":"Gain","parameters":{"gain":1.0,"inverted":false}},"gain_sub1":{"type":"Gain","parameters":{"gain":1.0,"inverted":false}},"gain_sub2":{"type":"Gain","parameters":{"gain":1.0,"inverted":false}},"peq_l_0":{"type":"Biquad","parameters":{"freq":1000,"q":0.707,"gain":2.0,"type":"Peaking"}},"peq_l_1":{"type":"Biquad","parameters":{"freq":2000,"q":0.707,"gain":2.0,"type":"Peaking"}},"peq_r_2":{"type":"Biquad","parameters":{"freq":2000,"q":0.707,"gain":1.0,"type":"Peaking"}}},"pipeline":[{"type":"Mixer","name":"split_non_sub"},{"type":"Filter","channel":2,"names":["crossover_speaker_c"]},{"type":"Filter","channel":3,"names":["crossover_subwooferc"]},{"type":"Filter","channel":0,"names":["crossover_speaker_l"]},{"type":"Filter","channel":1,"names":["crossover_subwooferl"]},{"type":"Filter","channel":4,"names":["crossover_speaker_r"]},{"type":"Filter","channel":5,"names":["crossover_subwooferr"]},{"type":"Mixer","name":"combine_sub"},{"type":"Filter","channel":1,"names":["delay_c","gain_c"]},{"type":"Filter","channel":0,"names":["peq_l_0","peq_l_1","delay_l","gain_l"]},{"type":"Filter","channel":2,"names":["peq_r_2","delay_r","gain_r"]},{"type":"Filter","channel":3,"names":["delay_sub1","gain_sub1"]},{"type":"Filter","channel":4,"names":["delay_sub2","gain_sub2"]}],"devices":{"samplerate":96000,"chunksize":2048,"queuelimit":1,"capture":{"type":"Alsa","channels":4,"device":"hw:Loopback,1","format":"S32LE"},"playback":{"type":"Alsa","channels":5,"device":"hw:DAC8PRO","format":"S32LE"}}}"#
        )
    }
    #[test]
    fn check_processor_to_camilla_two_sub_no_crossover() {
        let settings = ProcessorSettingsForCamilla {
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
                    delay: 10.0,
                    gain: 1.0,
                    is_subwoofer: false,
                },
                Speaker {
                    speaker: "c".to_string(),
                    crossover: None,
                    delay: 10.0,
                    gain: 1.0,
                    is_subwoofer: false,
                },
                Speaker {
                    speaker: "r".to_string(),
                    crossover: None,
                    delay: 10.0,
                    gain: 1.0,
                    is_subwoofer: false,
                },
                Speaker {
                    speaker: "sub1".to_string(),
                    crossover: None,
                    delay: 10.0,
                    gain: 1.0,
                    is_subwoofer: true,
                },
                Speaker {
                    speaker: "sub2".to_string(),
                    crossover: None,
                    delay: 10.0,
                    gain: 1.0,
                    is_subwoofer: true,
                },
            ],
            //selected_distance: SelectedDistanceType::MS,
            device: DeviceType::OktoDac8,
        };

        assert_eq!(
            json::to_string(&convert_processor_settings_to_camilla(&settings).unwrap()).unwrap(),
            r#"{"mixers":{"combine_sub":{"channels":{"in":4,"out":5},"mapping":[{"sources":[{"channel":1,"gain":0,"inverted":false}],"dest":1},{"sources":[{"channel":0,"gain":0,"inverted":false}],"dest":0},{"sources":[{"channel":2,"gain":0,"inverted":false}],"dest":2},{"sources":[{"channel":3,"gain":0,"inverted":false}],"dest":3},{"sources":[{"channel":3,"gain":0,"inverted":false}],"dest":4}]},"split_non_sub":{"channels":{"in":4,"out":4},"mapping":[{"sources":[{"channel":1,"gain":0,"inverted":false}],"dest":1},{"sources":[{"channel":0,"gain":0,"inverted":false}],"dest":0},{"sources":[{"channel":2,"gain":0,"inverted":false}],"dest":2},{"sources":[{"channel":3,"gain":10,"inverted":false}],"dest":3}]}},"filters":{"delay_c":{"type":"Delay","parameters":{"delay":10.0,"unit":"ms"}},"delay_l":{"type":"Delay","parameters":{"delay":10.0,"unit":"ms"}},"delay_r":{"type":"Delay","parameters":{"delay":10.0,"unit":"ms"}},"delay_sub1":{"type":"Delay","parameters":{"delay":10.0,"unit":"ms"}},"delay_sub2":{"type":"Delay","parameters":{"delay":10.0,"unit":"ms"}},"gain_c":{"type":"Gain","parameters":{"gain":1.0,"inverted":false}},"gain_l":{"type":"Gain","parameters":{"gain":1.0,"inverted":false}},"gain_r":{"type":"Gain","parameters":{"gain":1.0,"inverted":false}},"gain_sub1":{"type":"Gain","parameters":{"gain":1.0,"inverted":false}},"gain_sub2":{"type":"Gain","parameters":{"gain":1.0,"inverted":false}},"peq_l_0":{"type":"Biquad","parameters":{"freq":1000,"q":0.707,"gain":2.0,"type":"Peaking"}},"peq_l_1":{"type":"Biquad","parameters":{"freq":2000,"q":0.707,"gain":2.0,"type":"Peaking"}},"peq_r_2":{"type":"Biquad","parameters":{"freq":2000,"q":0.707,"gain":1.0,"type":"Peaking"}}},"pipeline":[{"type":"Mixer","name":"split_non_sub"},{"type":"Mixer","name":"combine_sub"},{"type":"Filter","channel":1,"names":["delay_c","gain_c"]},{"type":"Filter","channel":0,"names":["peq_l_0","peq_l_1","delay_l","gain_l"]},{"type":"Filter","channel":2,"names":["peq_r_2","delay_r","gain_r"]},{"type":"Filter","channel":3,"names":["delay_sub1","gain_sub1"]},{"type":"Filter","channel":4,"names":["delay_sub2","gain_sub2"]}],"devices":{"samplerate":96000,"chunksize":2048,"queuelimit":1,"capture":{"type":"Alsa","channels":4,"device":"hw:Loopback,1","format":"S32LE"},"playback":{"type":"Alsa","channels":5,"device":"hw:DAC8PRO","format":"S32LE"}}}"#
        )
    }
    #[test]
    fn check_processor_to_camilla_two_sub_partial_crossover() {
        let settings = ProcessorSettingsForCamilla {
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
                    delay: 10.0,
                    gain: 1.0,
                    is_subwoofer: false,
                },
                Speaker {
                    speaker: "c".to_string(),
                    crossover: None,
                    delay: 10.0,
                    gain: 1.0,
                    is_subwoofer: false,
                },
                Speaker {
                    speaker: "r".to_string(),
                    crossover: None,
                    delay: 10.0,
                    gain: 1.0,
                    is_subwoofer: false,
                },
                Speaker {
                    speaker: "sub1".to_string(),
                    crossover: None,
                    delay: 10.0,
                    gain: 1.0,
                    is_subwoofer: true,
                },
                Speaker {
                    speaker: "sub2".to_string(),
                    crossover: None,
                    delay: 10.0,
                    gain: 1.0,
                    is_subwoofer: true,
                },
            ],
            //selected_distance: SelectedDistanceType::MS,
            device: DeviceType::OktoDac8,
        };

        assert_eq!(
            json::to_string(&convert_processor_settings_to_camilla(&settings).unwrap()).unwrap(),
            r#"{"mixers":{"combine_sub":{"channels":{"in":5,"out":5},"mapping":[{"sources":[{"channel":2,"gain":0,"inverted":false}],"dest":1},{"sources":[{"channel":0,"gain":0,"inverted":false}],"dest":0},{"sources":[{"channel":3,"gain":0,"inverted":false}],"dest":2},{"sources":[{"channel":1,"gain":0,"inverted":false},{"channel":4,"gain":0,"inverted":false}],"dest":3},{"sources":[{"channel":1,"gain":0,"inverted":false},{"channel":4,"gain":0,"inverted":false}],"dest":4}]},"split_non_sub":{"channels":{"in":4,"out":5},"mapping":[{"sources":[{"channel":1,"gain":0,"inverted":false}],"dest":2},{"sources":[{"channel":0,"gain":0,"inverted":false}],"dest":0},{"sources":[{"channel":0,"gain":0,"inverted":false}],"dest":1},{"sources":[{"channel":2,"gain":0,"inverted":false}],"dest":3},{"sources":[{"channel":3,"gain":10,"inverted":false}],"dest":4}]}},"filters":{"crossover_speaker_l":{"type":"BiquadCombo","parameters":{"freq":80,"order":4,"type":"ButterworthHighpass"}},"crossover_subwooferl":{"type":"BiquadCombo","parameters":{"freq":80,"order":4,"type":"ButterworthLowpass"}},"delay_c":{"type":"Delay","parameters":{"delay":10.0,"unit":"ms"}},"delay_l":{"type":"Delay","parameters":{"delay":10.0,"unit":"ms"}},"delay_r":{"type":"Delay","parameters":{"delay":10.0,"unit":"ms"}},"delay_sub1":{"type":"Delay","parameters":{"delay":10.0,"unit":"ms"}},"delay_sub2":{"type":"Delay","parameters":{"delay":10.0,"unit":"ms"}},"gain_c":{"type":"Gain","parameters":{"gain":1.0,"inverted":false}},"gain_l":{"type":"Gain","parameters":{"gain":1.0,"inverted":false}},"gain_r":{"type":"Gain","parameters":{"gain":1.0,"inverted":false}},"gain_sub1":{"type":"Gain","parameters":{"gain":1.0,"inverted":false}},"gain_sub2":{"type":"Gain","parameters":{"gain":1.0,"inverted":false}},"peq_l_0":{"type":"Biquad","parameters":{"freq":1000,"q":0.707,"gain":2.0,"type":"Peaking"}},"peq_l_1":{"type":"Biquad","parameters":{"freq":2000,"q":0.707,"gain":2.0,"type":"Peaking"}},"peq_r_2":{"type":"Biquad","parameters":{"freq":2000,"q":0.707,"gain":1.0,"type":"Peaking"}}},"pipeline":[{"type":"Mixer","name":"split_non_sub"},{"type":"Filter","channel":0,"names":["crossover_speaker_l"]},{"type":"Filter","channel":1,"names":["crossover_subwooferl"]},{"type":"Mixer","name":"combine_sub"},{"type":"Filter","channel":1,"names":["delay_c","gain_c"]},{"type":"Filter","channel":0,"names":["peq_l_0","peq_l_1","delay_l","gain_l"]},{"type":"Filter","channel":2,"names":["peq_r_2","delay_r","gain_r"]},{"type":"Filter","channel":3,"names":["delay_sub1","gain_sub1"]},{"type":"Filter","channel":4,"names":["delay_sub2","gain_sub2"]}],"devices":{"samplerate":96000,"chunksize":2048,"queuelimit":1,"capture":{"type":"Alsa","channels":4,"device":"hw:Loopback,1","format":"S32LE"},"playback":{"type":"Alsa","channels":5,"device":"hw:DAC8PRO","format":"S32LE"}}}"#
        )
    }
}
