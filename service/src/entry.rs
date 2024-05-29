use crate::{
    cookie::extract_cookie_value,
    dto::{entry::EntryDto, pagination::PaginationRequest},
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

pub async fn delete_entry(db: &DbConn, id: i32, cookie: &str) -> Result<()> {
    let token = extract_cookie_value(cookie, "token")
        .ok_or_else(|| Error::Unauthorized("Yetkisiz istek.".to_string()))?;

    let user = user_by_token(db, token).await?;

    let entry = Entry::find()
        .filter(EntryColumn::Id.eq(id))
        .one(db)
        .await
        .map_err(|_| Error::InternalError("Girdi bulunamadı.".to_string()))
        .and_then(|entry| entry.ok_or(Error::NotFound("Girdi bulunamadı.".to_string())))?;

    if entry.user_id != user.id && !user.is_admin && !user.is_moderator {
        return Err(Error::Unauthorized("Yetkisiz istek.".to_string()));
    }

    let mut entry: EntryActiveModel = entry.into();
    entry.deleted_at = Set(Some(chrono::Utc::now().naive_utc()));

    entry
        .save(db)
        .await
        .map_err(|_| Error::InternalError("Girdi silinemedi.".to_string()))?;

    Ok(())
}

pub async fn get_entry(db: &DbConn, id: i32) -> Result<EntryDto> {
    let entry = Entry::find()
        .filter(EntryColumn::Id.eq(id))
        .filter(EntryColumn::DeletedAt.is_null())
        .one(db)
        .await
        .map_err(|_| Error::InternalError("Girdi bulunamadı.".to_string()))
        .and_then(|entry| entry.ok_or(Error::NotFound("Girdi bulunamadı.".to_string())))?;

    let user = User::find()
        .filter(UserColumn::Id.eq(entry.user_id))
        .one(db)
        .await
        .map_err(|_| Error::InternalError("Kullanıcı bulunamadı.".to_string()))
        .and_then(|user| user.ok_or(Error::NotFound("Kullanıcı bulunamadı.".to_string())))?;

    let title = Title::find()
        .filter(TitleColumn::Id.eq(entry.title_id))
        .one(db)
        .await
        .map_err(|_| Error::InternalError("Başlık bulunamadı.".to_string()))
        .and_then(|title| title.ok_or(Error::NotFound("Başlık bulunamadı.".to_string())))?;

    Ok(EntryDto {
        id: entry.id,
        title_id: title.id,
        title: title.name,
        content: entry.content,
        user_id: user.id,
        user_nickname: user.nickname,
        created_at: entry.created_at.to_string(),
        updated_at: entry.updated_at.to_string(),
    })
}

pub async fn get_title_entries(
    db: &DbConn,
    title_id: i32,
    pagination: PaginationRequest,
) -> Result<Vec<EntryDto>> {
    pagination.validate().map_err(|_| {
        Error::InvalidRequest("Geçersiz istek. Lütfen girilen bilgileri kontrol edin.".to_string())
    })?;

    let entry_pages = Entry::find()
        .filter(EntryColumn::TitleId.eq(title_id))
        .filter(EntryColumn::DeletedAt.is_null())
        .order_by_asc(EntryColumn::CreatedAt)
        .paginate(db, pagination.per_page);

    let mut entry_dtos = Vec::new();
    let entries = entry_pages
        .fetch_page(pagination.page)
        .await
        .map_err(|_| Error::InternalError("Girdiler getirilemedi.".to_string()))?;

    entries.into_iter().for_each(|entry| {
        entry_dtos.push(EntryDto {
            id: entry.id,
            title_id: entry.title_id,
            title: "".to_string(),
            content: entry.content,
            user_id: entry.user_id,
            user_nickname: "".to_string(),
            created_at: entry.created_at.to_string(),
            updated_at: entry.updated_at.to_string(),
        });
    });

    Ok(entry_dtos)
}
