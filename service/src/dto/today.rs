use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]

pub struct TodayTitleDto {
    pub id: i32,
    pub name: String,
    #[serde(rename = "entryCount")]
    pub entry_count: u64,
}
