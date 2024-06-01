use ::entity::prelude::*;
use sea_orm::*;
use validator::Validate;

use crate::{
    cookie::extract_cookie_value,
    dto::{
        entry::{
            CreateEntryRequest, EntryDto, GetTitleEntriesFilter, GetUserEntriesFilter,
            UpdateEntryRequest,
        },
        pagination::PaginationResponse,
    },
    title::{create_title, title_id_by_name},
    token::{get_id, is_admin, is_moderator},
    user::user_by_token,
    Author, Deleted, Error, Result, TitleVisible,
};

pub async fn create_entry(db: &DbConn, cookie: &str, request: CreateEntryRequest) -> Result<()> {
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

pub async fn delete_entry(db: &DbConn, cookie: &str, id: i32, soft_delete: bool) -> Result<()> {
    let token = extract_cookie_value(cookie, "token")
        .ok_or_else(|| Error::Unauthorized("Yetkisiz istek.".to_string()))?;

    let user = user_by_token(db, token).await?;

    let entry = Entry::find()
        .filter(EntryColumn::Id.eq(id))
        .filter(EntryColumn::DeletedAt.is_null())
        .apply_if(Some(!user.is_admin && !user.is_moderator), |query, v| {
            if v {
                query.filter(EntryColumn::UserId.eq(user.id))
            } else {
                query
            }
        })
        .one(db)
        .await
        .map_err(|_| Error::InternalError("Girdi bulunamadı.".to_string()))
        .and_then(|entry| entry.ok_or(Error::NotFound("Girdi bulunamadı.".to_string())))?;

    if !soft_delete {
        entry
            .delete(db)
            .await
            .map_err(|_| Error::InternalError("Girdi silinemedi.".to_string()))?;

        return Ok(());
    }

    let mut entry: EntryActiveModel = entry.into();
    entry.deleted_at = Set(Some(chrono::Utc::now().naive_utc()));

    entry
        .save(db)
        .await
        .map_err(|_| Error::InternalError("Girdi silinemedi.".to_string()))?;

    Ok(())
}

pub async fn recover_entry(db: &DbConn, cookie: &str, id: i32) -> Result<()> {
    let token = extract_cookie_value(cookie, "token")
        .ok_or_else(|| Error::Unauthorized("Yetkisi istek.".to_string()))?;

    let user = user_by_token(db, token).await?;

    let entry = Entry::find()
        .filter(EntryColumn::Id.eq(id))
        .filter(EntryColumn::DeletedAt.is_not_null())
        .apply_if(Some(!user.is_admin && !user.is_moderator), |query, v| {
            if v {
                query.filter(EntryColumn::UserId.eq(user.id))
            } else {
                query
            }
        })
        .one(db)
        .await
        .map_err(|_| Error::InternalError("Girdi bulunamadı.".to_string()))
        .and_then(|entry| entry.ok_or(Error::NotFound("Girdi bulunamadı.".to_string())))?;

    let mut entry: EntryActiveModel = entry.into();
    entry.deleted_at = Set(None);

    entry
        .save(db)
        .await
        .map_err(|_| Error::InternalError("Girdi geri alınamadı.".to_string()))?;

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
        is_author_entry: user.is_author,
        created_at: entry.created_at.to_string(),
        updated_at: entry.updated_at.to_string(),
    })
}

pub async fn update_entry(
    db: &DbConn,
    cookie: &str,
    id: i32,
    request: UpdateEntryRequest,
) -> Result<()> {
    request.validate().map_err(|_| {
        Error::InvalidRequest("Geçersiz istek. Lütfen girilen bilgileri kontrol edin.".to_string())
    })?;

    let token = extract_cookie_value(cookie, "token")
        .ok_or_else(|| Error::Unauthorized("Yetkisiz istek.".to_string()))?;

    let user = user_by_token(db, token).await?;

    let entry = Entry::find()
        .filter(EntryColumn::Id.eq(id))
        .filter(EntryColumn::DeletedAt.is_null())
        .apply_if(Some(!user.is_admin && !user.is_moderator), |query, v| {
            if v {
                query.filter(EntryColumn::UserId.eq(user.id))
            } else {
                query
            }
        })
        .one(db)
        .await
        .map_err(|_| Error::InternalError("Girdi bulunamadı.".to_string()))
        .and_then(|entry| entry.ok_or(Error::NotFound("Girdi bulunamadı.".to_string())))?;

    let mut entry: EntryActiveModel = entry.into();
    entry.content = Set(request.content);
    entry.updated_at = Set(chrono::Utc::now().naive_utc());

    entry
        .save(db)
        .await
        .map_err(|_| Error::InternalError("Girdi güncellenemedi.".to_string()))?;

    Ok(())
}

pub async fn migrate_entry(db: &DbConn, cookie: &str, id: i32, title_id: i32) -> Result<()> {
    let token = extract_cookie_value(cookie, "token")
        .ok_or_else(|| Error::Unauthorized("Yetkisi istek.".to_string()))?;

    let user = user_by_token(db, token).await?;
    if !user.is_admin && !user.is_moderator {
        return Err(Error::Unauthorized("Yetkisiz istek.".to_string()));
    }

    let entry = Entry::find()
        .filter(EntryColumn::Id.eq(id))
        .one(db)
        .await
        .map_err(|_| Error::InternalError("Girdi bulunamadı.".to_string()))
        .and_then(|entry| entry.ok_or(Error::NotFound("Girdi bulunamadı.".to_string())))?;

    // TODO: Use exists instead of find
    Title::find()
        .filter(TitleColumn::Id.eq(title_id))
        .one(db)
        .await
        .map_err(|_| Error::InternalError("Başlık bulunamadı.".to_string()))
        .and_then(|title| title.ok_or(Error::NotFound("Başlık bulunamadı.".to_string())))?;

    let mut entry: EntryActiveModel = entry.into();
    entry.title_id = Set(title_id);

    entry
        .save(db)
        .await
        .map_err(|_| Error::InternalError("Girdi taşınamadı.".to_string()))?;

    Ok(())
}

pub async fn get_title_entries(
    db: &DbConn,
    cookie: Option<&str>,
    title_id: i32,
    filter: GetTitleEntriesFilter,
) -> Result<PaginationResponse<EntryDto>> {
    filter.validate().map_err(|_| {
        Error::InvalidRequest("Geçersiz istek. Lütfen girilen bilgileri kontrol edin.".to_string())
    })?;

    // TODO: Stop getting the key from the environment variable again and again
    let key = match std::env::var("JWT_SECRET") {
        Ok(key) => key,
        Err(_) => return Err(Error::InternalError("JWT anahtarı bulunamadı.".to_string())),
    };

    let token = match cookie {
        Some(cookie) => extract_cookie_value(cookie, "token"),
        None => None,
    };

    let is_admin_or_moderator = match token {
        Some(token) => is_admin(token, &key) || is_moderator(token, &key),
        None => false,
    };

    match filter.deleted {
        None | Some(Deleted::Only) if !is_admin_or_moderator => {
            return Err(Error::Unauthorized("Yetkisiz istek.".to_string()));
        }
        _ => {}
    }

    let entry_pages = Entry::find()
        .filter(EntryColumn::TitleId.eq(title_id))
        .apply_if(Some(&filter.deleted), |query, v| match v {
            Some(Deleted::Only) => query.filter(EntryColumn::DeletedAt.is_not_null()),
            Some(Deleted::None) => query.filter(EntryColumn::DeletedAt.is_null()),
            None => query,
        })
        .order_by_asc(EntryColumn::CreatedAt)
        .paginate(db, filter.per_page.into());

    let entries = entry_pages
        .fetch_page(filter.page.into())
        .await
        .map_err(|_| Error::InternalError("Girdiler getirilemedi.".to_string()))?;

    let title_name = Title::find()
        .filter(TitleColumn::Id.eq(title_id))
        .apply_if(Some(!is_admin_or_moderator), |query, v| {
            if v {
                query.filter(TitleColumn::IsVisible.eq(true))
            } else {
                query
            }
        })
        .one(db)
        .await
        .map_err(|_| Error::InternalError("Başlık adı getirilemedi.".to_string()))
        .and_then(|title| title.ok_or(Error::NotFound("Başlık adı getirilemedi.".to_string())))?
        .name;

    let entry_dto_futures = entries.into_iter().map(|entry| {
        let db = db.clone();
        let title_name = title_name.clone();

        async move {
            let user = User::find()
                .filter(UserColumn::Id.eq(entry.user_id))
                .apply_if(Some(&filter.author), |query, v| match v {
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
                    is_author_entry: user.is_author,
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
        .apply_if(Some(&filter.deleted), |query, v| match v {
            Some(Deleted::Only) => query.filter(EntryColumn::DeletedAt.is_not_null()),
            Some(Deleted::None) => query.filter(EntryColumn::DeletedAt.is_null()),
            None => query,
        })
        .inner_join(User)
        .apply_if(Some(&filter.author), |query, v| match v {
            Some(Author::Only) => query.filter(UserColumn::IsAuthor.eq(true)),
            Some(Author::None) => query.filter(UserColumn::IsAuthor.eq(false)),
            None => query,
        })
        .count(db)
        .await
        .map_err(|_| Error::InternalError("Girdi sayısı getirilemedi.".to_string()))?;

    Ok(PaginationResponse {
        total,
        page: filter.page,
        per_page: filter.per_page,
        data: entry_dtos,
    })
}

pub async fn get_user_entries(
    db: &DbConn,
    cookie: Option<&str>,
    user_id: i32,
    filter: GetUserEntriesFilter,
) -> Result<PaginationResponse<EntryDto>> {
    filter.validate().map_err(|_| {
        Error::InvalidRequest("Geçersiz istek. Lütfen girilen bilgileri kontrol edin.".to_string())
    })?;

    let key = match std::env::var("JWT_SECRET") {
        Ok(key) => key,
        Err(_) => return Err(Error::InternalError("JWT anahtarı bulunamadı.".to_string())),
    };

    let token = match cookie {
        Some(cookie) => extract_cookie_value(cookie, "token"),
        None => None,
    };

    let is_admin_or_moderator = match token {
        Some(token) => is_admin(token, &key) || is_moderator(token, &key),
        None => false,
    };

    let self_user_id = match token {
        Some(token) => get_id(token, &key),
        None => None,
    };

    match filter.deleted {
        None | Some(Deleted::Only)
            if !is_admin_or_moderator && self_user_id.is_none()
                || !is_admin_or_moderator
                    && self_user_id.is_some()
                    && user_id != self_user_id.unwrap() =>
        {
            return Err(Error::Unauthorized("Yetkisiz istek.".to_string()));
        }
        _ => {}
    }

    match filter.title_visible {
        Some(TitleVisible::None) | None if !is_admin_or_moderator => {
            return Err(Error::Unauthorized("Yetkisiz istek.".to_string()));
        }
        _ => {}
    }

    let user = User::find()
        .filter(UserColumn::Id.eq(user_id))
        .one(db)
        .await
        .map_err(|_| Error::InternalError("Kullanıcı bulunamadı.".to_string()))
        .and_then(|user| user.ok_or(Error::NotFound("Kullanıcı bulunamadı.".to_string())))?;

    let entry_pages = Entry::find()
        .filter(EntryColumn::UserId.eq(user_id))
        .apply_if(Some(&filter.deleted), |query, v| match v {
            Some(Deleted::Only) => query.filter(EntryColumn::DeletedAt.is_not_null()),
            Some(Deleted::None) => query.filter(EntryColumn::DeletedAt.is_null()),
            None => query,
        })
        .order_by_asc(EntryColumn::CreatedAt)
        .paginate(db, filter.per_page.into());

    let entries = entry_pages
        .fetch_page(filter.page.into())
        .await
        .map_err(|_| Error::InternalError("Girdiler getirilemedi.".to_string()))?;

    let entry_dto_futures = entries.into_iter().map(|entry| {
        let db = db.clone();
        let user_nickname = user.nickname.clone();
        let user_is_author = user.is_author;

        async move {
            let title = Title::find()
                .filter(TitleColumn::Id.eq(entry.title_id))
                .apply_if(Some(&filter.title_visible), |query, v| match v {
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
                    user_nickname,
                    is_author_entry: user_is_author,
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
        .apply_if(Some(&filter.deleted), |query, v| match v {
            Some(Deleted::Only) => query.filter(EntryColumn::DeletedAt.is_not_null()),
            Some(Deleted::None) => query.filter(EntryColumn::DeletedAt.is_null()),
            None => query,
        })
        .inner_join(Title)
        .apply_if(Some(&filter.title_visible), |query, v| match v {
            Some(TitleVisible::Only) => query.filter(TitleColumn::IsVisible.eq(true)),
            Some(TitleVisible::None) => query.filter(TitleColumn::IsVisible.eq(false)),
            None => query,
        })
        .count(db)
        .await
        .map_err(|_| Error::InternalError("Girdi sayısı getirilemedi.".to_string()))?;

    Ok(PaginationResponse {
        total,
        page: filter.page,
        per_page: filter.per_page,
        data: entry_dtos,
    })
}
