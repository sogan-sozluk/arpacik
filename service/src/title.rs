use crate::{Error, Result};
use ::entity::prelude::*;
use sea_orm::*;

pub async fn title_id_by_name(db: &DbConn, name: &str) -> Option<i32> {
    let title = Title::find()
        .filter(TitleColumn::Name.eq(name))
        .filter(TitleColumn::IsVisible.eq(true))
        .one(db)
        .await
        .map_err(|_| Error::InternalError("Başlık bulunamadı.".to_string()))
        .and_then(|title| title.ok_or(Error::InternalError("Başlık bulunamadı.".to_string())));

    match title {
        Ok(title) => Some(title.id),
        Err(_) => None,
    }
}

pub async fn create_title(db: &DbConn, name: &str) -> Result<TitleActiveModel> {
    TitleActiveModel {
        name: Set(name.to_string()),
        is_visible: Set(true),
        ..Default::default()
    }
    .save(db)
    .await
    .map_err(|_| Error::InternalError("Başlık oluşturulamadı.".to_string()))
}
