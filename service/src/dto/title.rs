use chrono::NaiveDateTime;
use entity::prelude::TitleActiveModel;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct TitleDto {
    pub id: i32,
    pub name: String,
    pub last_entry_at: NaiveDateTime,
    pub is_visible: bool,
}

impl From<TitleActiveModel> for TitleDto {
    fn from(title: TitleActiveModel) -> Self {
        Self {
            id: title.id.unwrap(),
            name: title.name.unwrap(),
            last_entry_at: title.last_entry_at.unwrap(),
            is_visible: title.is_visible.unwrap(),
        }
    }
}
