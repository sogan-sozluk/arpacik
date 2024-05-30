use crate::{
    cookie::extract_cookie_value,
    dto::{
        entry::EntryDto,
        pagination::{PaginationRequest, PaginationResponse},
    },
    title::{create_title, title_id_by_name},
    user::user_by_token,
    Error, Result,
};
use ::entity::prelude::*;
use sea_orm::*;
use validator::Validate;

use crate::dto::entry::CreateEntryRequest;

#[derive(Clone)]
pub enum Deleted {
    Only,
    None,
}

#[derive(Clone)]
pub enum Author {
    Only,
    None,
}

#[derive(Clone)]
pub enum TitleVisible {
    Only,
    None,
}

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
    deleted: Option<Deleted>,
    author: Option<Author>,
) -> Result<PaginationResponse<EntryDto>> {
    pagination.validate().map_err(|_| {
        Error::InvalidRequest("Geçersiz istek. Lütfen girilen bilgileri kontrol edin.".to_string())
    })?;

    let entry_pages = Entry::find()
        .filter(EntryColumn::TitleId.eq(title_id))
        .apply_if(Some(&deleted), |query, v| match v {
            Some(Deleted::Only) => query.filter(EntryColumn::DeletedAt.is_not_null()),
            Some(Deleted::None) => query.filter(EntryColumn::DeletedAt.is_null()),
            None => query,
        })
        .order_by_asc(EntryColumn::CreatedAt)
        .paginate(db, pagination.per_page);

    let entries = entry_pages
        .fetch_page(pagination.page)
        .await
        .map_err(|_| Error::InternalError("Girdiler getirilemedi.".to_string()))?;

    let title_name = Title::find()
        .filter(TitleColumn::Id.eq(title_id))
        .one(db)
        .await
        .map_err(|_| Error::InternalError("Başlık adı getirilemedi.".to_string()))
        .and_then(|title| title.ok_or(Error::NotFound("Başlık adı getirilemedi.".to_string())))?
        .name;

    let entry_dto_futures = entries.into_iter().map(|entry| {
        let db = db.clone();
        let title_name = title_name.clone();
        let author = author.clone();

        async move {
            let user = User::find()
                .filter(UserColumn::Id.eq(entry.user_id))
                .apply_if(Some(&author), |query, v| match v {
                    Some(Author::Only) => query.filter(UserColumn::IsAuthor.eq(true)),
                    Some(Author::None) => query.filter(UserColumn::IsAuthor.eq(false)),
                    None => query,
                })
                .one(&db)
                .await;

            if let Ok(Some(user)) = user {
                Ok(Some(EntryDto {
                    id: entry.id,
                    title_id: entry.title_id,
                    title: title_name,
                    content: entry.content,
                    user_id: entry.user_id,
                    user_nickname: user.nickname,
                    created_at: entry.created_at.to_string(),
                    updated_at: entry.updated_at.to_string(),
                }))
            } else {
                Ok(None)
            }
        }
    });

    let entry_dtos: Result<Vec<Option<EntryDto>>> = futures::future::join_all(entry_dto_futures)
        .await
        .into_iter()
        .collect();

    let entry_dtos: Vec<EntryDto> = entry_dtos?.into_iter().flatten().collect();

    let total = Entry::find()
        .filter(EntryColumn::TitleId.eq(title_id))
        .apply_if(Some(&deleted), |query, v| match v {
            Some(Deleted::Only) => query.filter(EntryColumn::DeletedAt.is_not_null()),
            Some(Deleted::None) => query.filter(EntryColumn::DeletedAt.is_null()),
            None => query,
        })
        .inner_join(User)
        .apply_if(Some(&author), |query, v| match v {
            Some(Author::Only) => query.filter(UserColumn::IsAuthor.eq(true)),
            Some(Author::None) => query.filter(UserColumn::IsAuthor.eq(false)),
            None => query,
        })
        .count(db)
        .await
        .map_err(|_| Error::InternalError("Girdi sayısı getirilemedi.".to_string()))?;

    Ok(PaginationResponse {
        total,
        page: pagination.page,
        per_page: pagination.per_page,
        data: entry_dtos,
    })
}

pub async fn get_user_entries(
    db: &DbConn,
    user_id: i32,
    pagination: PaginationRequest,
    deleted: Option<Deleted>,
    title_visible: Option<TitleVisible>,
) -> Result<PaginationResponse<EntryDto>> {
    pagination.validate().map_err(|_| {
        Error::InvalidRequest("Geçersiz istek. Lütfen girilen bilgileri kontrol edin.".to_string())
    })?;

    let entry_pages = Entry::find()
        .filter(EntryColumn::UserId.eq(user_id))
        .apply_if(Some(&deleted), |query, v| match v {
            Some(Deleted::Only) => query.filter(EntryColumn::DeletedAt.is_not_null()),
            Some(Deleted::None) => query.filter(EntryColumn::DeletedAt.is_null()),
            None => query,
        })
        .order_by_asc(EntryColumn::CreatedAt)
        .paginate(db, pagination.per_page);

    let entries = entry_pages
        .fetch_page(pagination.page)
        .await
        .map_err(|_| Error::InternalError("Girdiler getirilemedi.".to_string()))?;

    let entry_dto_futures = entries.into_iter().map(|entry| {
        let db = db.clone();
        let title_visible = title_visible.clone();

        async move {
            let title = Title::find()
                .filter(TitleColumn::Id.eq(entry.title_id))
                .apply_if(Some(&title_visible), |query, v| match v {
                    Some(TitleVisible::Only) => query.filter(TitleColumn::IsVisible.eq(true)),
                    Some(TitleVisible::None) => query.filter(TitleColumn::IsVisible.eq(false)),
                    None => query,
                })
                .one(&db)
                .await;

            if let Ok(Some(title)) = title {
                Ok(Some(EntryDto {
                    id: entry.id,
                    title_id: entry.title_id,
                    title: title.name,
                    content: entry.content,
                    user_id: entry.user_id,
                    user_nickname: "".to_string(),
                    created_at: entry.created_at.to_string(),
                    updated_at: entry.updated_at.to_string(),
                }))
            } else {
                Ok(None)
            }
        }
    });

    let entry_dtos: Result<Vec<Option<EntryDto>>> = futures::future::join_all(entry_dto_futures)
        .await
        .into_iter()
        .collect();

    let entry_dtos: Vec<EntryDto> = entry_dtos?.into_iter().flatten().collect();

    let total = Entry::find()
        .filter(EntryColumn::UserId.eq(user_id))
        .apply_if(Some(&deleted), |query, v| match v {
            Some(Deleted::Only) => query.filter(EntryColumn::DeletedAt.is_not_null()),
            Some(Deleted::None) => query.filter(EntryColumn::DeletedAt.is_null()),
            None => query,
        })
        .inner_join(Title)
        .apply_if(Some(&title_visible), |query, v| match v {
            Some(TitleVisible::Only) => query.filter(TitleColumn::IsVisible.eq(true)),
            Some(TitleVisible::None) => query.filter(TitleColumn::IsVisible.eq(false)),
            None => query,
        })
        .count(db)
        .await
        .map_err(|_| Error::InternalError("Girdi sayısı getirilemedi.".to_string()))?;

    Ok(PaginationResponse {
        total,
        page: pagination.page,
        per_page: pagination.per_page,
        data: entry_dtos,
    })
}
