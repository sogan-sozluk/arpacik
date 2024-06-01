use serde::{Deserialize, Serialize};
use validator::Validate;

use crate::{Author, Deleted, TitleVisible};

#[derive(Debug, Serialize, Deserialize)]
pub struct EntryDto {
    // TODO: Add `is_title_visible`
    pub id: i32,
    pub title_id: i32,
    pub title: String,
    pub content: String,
    pub user_id: i32,
    pub user_nickname: String,
    pub is_author_entry: bool,
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

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct GetTitleEntriesFilter {
    #[validate(range(min = 1, max = 100))]
    pub per_page: u8,
    #[validate(range(min = 0))]
    pub page: u8,
    pub deleted: Option<Deleted>,
    pub author: Option<Author>,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct GetUserEntriesFilter {
    #[validate(range(min = 1, max = 100))]
    pub per_page: u8,
    #[validate(range(min = 0))]
    pub page: u8,
    pub deleted: Option<Deleted>,
    pub title_visible: Option<TitleVisible>,
}
