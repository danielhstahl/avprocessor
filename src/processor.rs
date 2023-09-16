use rocket::serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
#[serde(crate = "rocket::serde")]
pub struct ProcessorSettings {
    pub filters: Vec<Filter>,
    pub speakers: Vec<Speaker>,
}

#[derive(Deserialize, Serialize, sqlx::FromRow)]
#[serde(crate = "rocket::serde", rename_all = "camelCase")]
pub struct Speaker {
    pub speaker: String,
    pub crossover: Option<i32>,
    pub delay: i32,
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
