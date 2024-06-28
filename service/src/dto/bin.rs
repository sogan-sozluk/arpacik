use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct GetUserBinQuery {
    #[validate(range(min = 1, max = 100))]
    #[serde(rename = "perPage")]
    pub per_page: u8,
    #[validate(range(min = 1))]
    pub page: u8,
}
