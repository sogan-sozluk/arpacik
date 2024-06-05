use sea_orm::FromQueryResult;
use serde::{Deserialize, Serialize};

#[derive(FromQueryResult, Debug, Serialize, Deserialize)]

pub struct TrendTitleDto {
    pub id: i32,
    pub name: String,
    pub entry_count: i64,
}
