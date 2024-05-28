use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct CreateEntryRequest {
    #[validate(length(min = 1, max = 75))]
    pub title: String,
    #[validate(length(min = 1, max = 65535))]
    pub content: String,
}
