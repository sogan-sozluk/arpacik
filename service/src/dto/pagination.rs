use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct PaginationQuery {
    #[validate(range(min = 0))]
    pub page: u8,
    #[validate(range(min = 1, max = 100))]
    #[serde(rename = "perPage")]
    pub per_page: u8,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PaginationResponse<T> {
    pub total: u64,
    pub page: u8,
    #[serde(rename = "perPage")]
    pub per_page: u8,
    pub items: Vec<T>,
}
