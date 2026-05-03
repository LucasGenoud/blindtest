use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct User {
    pub id: String,
    pub email: String,
    pub name: String,
    #[serde(skip_serializing)]
    pub password: String,
    pub role: String,
    pub clear_mode: bool,
    pub hide_carousel: bool,
    pub email_confirmed: bool,
    pub register_date: String,
    pub deleted: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UserPublic {
    pub id: String,
    pub name: String,
    pub role: String,
    pub email: String,
    pub clear_mode: bool,
    pub hide_carousel: bool,
    pub email_confirmed: bool,
    pub register_date: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Audio {
    #[serde(rename = "_id")]
    pub id: String,
    pub category: String,
    pub answer: String,
    #[serde(rename = "videoUrl")]
    pub video_url: String,
    #[serde(rename = "startTime")]
    pub start_time: i64,
    pub superflus: bool,
    pub count: i64,
    #[serde(rename = "submittedBy")]
    pub submitted_by: String,
    #[serde(rename = "submittedByUsername")]
    pub submitted_by_username: Option<String>,
    #[serde(rename = "addedDate")]
    pub added_date: String,
    pub rating: Option<f64>,
    #[serde(rename = "ratingCount")]
    pub rating_count: Option<i64>,
    pub flagged: Option<Vec<FlaggedAudio>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FlaggedAudio {
    pub id: String,
    pub audio_id: String,
    pub user_id: String,
    pub report_message: String,
    pub date: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CustomBlindtest {
    #[serde(rename = "_id")]
    pub id: String,
    pub name: String,
    pub public: bool,
    #[serde(rename = "ownerId")]
    pub owner_id: String,
    pub username: Option<String>,
    #[serde(rename = "addedDate")]
    pub added_date: String,
    #[serde(rename = "blindtestList")]
    pub blindtest_list: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Rating {
    pub id: String,
    pub audio_id: String,
    pub user_id: String,
    pub rating: f64,
    pub added_date: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ChatMessage {
    pub id: String,
    pub user_id: String,
    pub username: String,
    #[serde(rename = "messageValue")]
    pub message_value: String,
    pub date: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Suggestion {
    #[serde(rename = "_id")]
    pub id: String,
    pub category: String,
    pub answer: String,
    #[serde(rename = "videoUrl")]
    pub video_url: String,
    #[serde(rename = "startTime")]
    pub start_time: i64,
    pub superflus: bool,
    #[serde(rename = "submittedBy")]
    pub submitted_by: String,
    #[serde(rename = "submittedByUsername")]
    pub submitted_by_username: Option<String>,
    #[serde(rename = "addedDate")]
    pub added_date: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Stat {
    pub id: String,
    pub category: String,
    pub user_id: Option<String>,
    pub date: String,
    pub metadata: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PixelData {
    pub x: i64,
    pub y: i64,
    #[serde(rename = "c")]
    pub color: String,
    pub username: Option<String>,
    #[serde(rename = "d")]
    pub updated_at: Option<String>,
}
