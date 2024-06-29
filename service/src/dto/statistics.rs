use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct StatisticsResponse {
    #[serde(rename = "titleCount")]
    pub title_count: u64,
    #[serde(rename = "entryCount")]
    pub entry_count: u64,
    #[serde(rename = "userCount")]
    pub user_count: u64,
    #[serde(rename = "crewCount")]
    pub crew_count: u64,
}
