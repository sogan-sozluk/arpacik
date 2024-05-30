use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Debug, Serialize, Deserialize)]
pub struct EntryDto {
    pub id: i32,
    pub title_id: i32,
    pub title: String,
    pub content: String,
    pub user_id: i32,
    pub user_nickname: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct CreateEntryRequest {
    #[validate(length(min = 1, max = 75))]
    pub title: String,
    #[validate(length(min = 1, max = 65535))]
    pub content: String,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct UpdateEntryRequest {
    #[validate(length(min = 1, max = 65535))]
    pub content: String,
}
