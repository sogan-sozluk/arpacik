use crate::{dto::title::TitleDto, Error, Result};
use ::entity::prelude::*;
use sea_orm::*;

pub async fn title_id_by_name(db: &DbConn, name: &str) -> Option<i32> {
    // TODO: Do not use variable for title_id
    let title_id: Option<i32> = Title::find()
        .filter(TitleColumn::Name.eq(name))
        .filter(TitleColumn::IsVisible.eq(true))
        .select_only()
        .column(TitleColumn::Id)
        .into_tuple()
        .one(db)
        .await
        .ok()?;

    title_id
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

pub async fn set_title_visibility(db: &DbConn, id: i32, is_visible: bool) -> Result<TitleDto> {
    let mut title = Title::find()
        .filter(TitleColumn::Id.eq(id))
        .one(db)
        .await
        .map_err(|_| Error::InternalError("Başlık bulunamadı.".to_string()))
        .and_then(|title| title.ok_or(Error::InternalError("Başlık bulunamadı.".to_string())))?
        .into_active_model();

    title.is_visible = Set(is_visible);

    title
        .clone()
        .save(db)
        .await
        .map_err(|_| Error::InternalError("Başlık güncellenemedi.".to_string()))?;

    Ok(TitleDto::from(title))
}
