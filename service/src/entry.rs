use crate::{
    cookie::extract_cookie_value,
    title::{create_title, title_id_by_name},
    user::user_by_token,
    Error, Result,
};
use ::entity::prelude::*;
use sea_orm::*;
use validator::Validate;

use crate::dto::entry::CreateEntryRequest;

pub async fn create_entry(db: &DbConn, request: CreateEntryRequest, cookie: &str) -> Result<()> {
    request.validate().map_err(|_| {
        Error::InvalidRequest("Geçersiz istek. Lütfen girilen bilgileri kontrol edin.".to_string())
    })?;

    let token = extract_cookie_value(cookie, "token")
        .ok_or_else(|| Error::Unauthorized("Yetkisiz istek.".to_string()))?;

    let user = user_by_token(db, token).await?;
    let title_id = match title_id_by_name(db, &request.title).await {
        Some(title_id) => title_id,
        None => {
            if user.is_author {
                create_title(db, &request.title).await?.id.unwrap()
            } else {
                return Err(Error::InvalidRequest("Başlık bulunamadı.".to_string()));
            }
        }
    };

    EntryActiveModel {
        title_id: Set(title_id),
        user_id: Set(user.id),
        content: Set(request.content),
        ..Default::default()
    }
    .save(db)
    .await
    .map_err(|e| {
        if e.to_string().contains("duplicate key value") {
            return Error::InvalidRequest("Böyle bir girdi zaten var.".to_string());
        }
        Error::InternalError("Girdi oluşturulamadı.".to_string())
    })?;

    Ok(())
}
