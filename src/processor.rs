use rocket::serde::{Deserialize, Serialize};

pub struct ProcessorSettingsForCamilla {
    pub filters: Vec<Filter>,
    pub speakers: Vec<Speaker>,
    pub selected_distance: SelectedDistanceType,
}

#[derive(Serialize, Deserialize)]
#[serde(crate = "rocket::serde", rename_all = "camelCase")]
pub struct ProcessorSettings {
    pub filters: Vec<Filter>,
    pub speakers: Vec<SpeakerForUI>,
    pub selected_distance: SelectedDistanceType,
}

#[derive(sqlx::FromRow)]
pub struct Speaker {
    pub speaker: String,
    pub crossover: Option<i32>,
    pub delay: f32,
    pub gain: f32,
    pub is_subwoofer: bool,
}

#[derive(Serialize, Deserialize, Debug, sqlx::Type)]
#[serde(crate = "rocket::serde", rename_all = "lowercase")]
pub enum SelectedDistanceType {
    MS,
    FEET,
    METERS,
}

#[derive(Serialize, Deserialize, sqlx::FromRow)]
#[serde(crate = "rocket::serde", rename_all = "camelCase")]
pub struct SpeakerForUI {
    pub speaker: String,
    pub crossover: Option<i32>,
    pub distance: f32, //contains meters, feet, or milliseconds
    pub gain: f32,
    pub is_subwoofer: bool,
}

#[derive(Deserialize, Serialize, sqlx::FromRow)]
#[serde(crate = "rocket::serde")]
pub struct Filter {
    pub freq: i32,
    pub gain: f32,
    pub q: f32,
    pub speaker: String,
}
