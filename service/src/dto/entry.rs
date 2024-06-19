use serde::{Deserialize, Serialize};
use validator::Validate;

use super::order::{Order, OrderBy};

#[derive(Debug, Serialize, Deserialize)]
pub struct EntryTitleDto {
    pub id: i32,
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EntryAuthorDto {
    pub id: i32,
    pub nickname: String,
    #[serde(rename = "isFaded")]
    pub is_faded: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Vote {
    Up,
    Down,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EntryDto {
    // TODO: Add `is_title_visible`
    pub id: i32,
    pub title: EntryTitleDto,
    pub content: String,
    pub author: EntryAuthorDto,
    #[serde(rename = "isFavorite")]
    pub is_favorite: Option<bool>,
    pub vote: Option<Vote>,
    #[serde(rename = "createdAt")]
    pub created_at: String,
    #[serde(rename = "updatedAt")]
    pub updated_at: Option<String>,
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

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct GetTitleEntriesQuery {
    #[validate(range(min = 1, max = 100))]
    #[serde(rename = "perPage")]
    pub per_page: u8,
    #[validate(range(min = 0))]
    pub page: u8,
    pub from: Option<chrono::DateTime<chrono::Utc>>,
    pub to: Option<chrono::DateTime<chrono::Utc>>,
    #[serde(rename = "orderBy")]
    pub order_by: Option<OrderBy>,
    pub order: Option<Order>,
}
