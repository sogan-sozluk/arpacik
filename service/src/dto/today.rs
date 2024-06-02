use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]

pub struct TodayTitleDto {
    pub id: i32,
    pub name: String,
    pub entry_count: u64,
}
