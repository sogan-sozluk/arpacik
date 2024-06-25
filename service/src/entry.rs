use ::entity::prelude::*;
use sea_orm::*;
use validator::Validate;

use crate::{
    dto::{
        entry::{
            CreateEntryRequest, CreateEntryResponse, EntryAuthorDto, EntryDto, EntryTitleDto,
            GetTitleEntriesQuery, UpdateEntryRequest,
        },
        order::{self, OrderBy},
        pagination::{PaginationQuery, PaginationResponse},
    },
    title::{create_title, title_id_by_name},
    Error, Result,
};

pub async fn create_entry(
    db: &DbConn,
    user_id: i32,
    request: CreateEntryRequest,
) -> Result<CreateEntryResponse> {
    request.validate().map_err(|_| {
        Error::InvalidRequest("Geçersiz istek. Lütfen girilen bilgileri kontrol edin.".to_string())
    })?;

    let is_faded = User::find()
        .filter(UserColumn::Id.eq(user_id))
        .filter(UserColumn::DeletedAt.is_null())
        .one(db)
        .await
        .map_err(|_| Error::InternalError("Kullanıcı bulunamadı.".to_string()))
        .and_then(|user| user.ok_or(Error::NotFound("Kullanıcı bulunamadı.".to_string())))?
        .is_faded;

    let title_id = match title_id_by_name(db, &request.title).await {
        Some(title_id) => title_id,
        None => {
            if !is_faded {
                create_title(db, &request.title).await?.id.unwrap()
            } else {
                return Err(Error::InvalidRequest(
                    "Solgun kullanıcılar başlık oluşturamaz.".to_string(),
                ));
            }
        }
    };
    let title = Title::find()
        .filter(TitleColumn::Name.eq(&request.title))
        .one(db)
        .await
        .map_err(|_| Error::InternalError("Başlık bulunamadı.".to_string()))?;

    let mut title = match title {
        Some(title) => title.into(),
        None => create_title(db, &request.title).await?,
    };

    title.last_entry_at = Set(chrono::Utc::now().naive_utc());
    title
        .save(db)
        .await
        .map_err(|_| Error::InternalError("Başlık güncellenemedi.".to_string()))?;

    let entry = EntryActiveModel {
        title_id: Set(title_id),
        user_id: Set(user_id),
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

    Ok(CreateEntryResponse {
        id: entry.id.unwrap(),
    })
}

pub async fn delete_entry(db: &DbConn, user_id: i32, id: i32, soft_delete: bool) -> Result<()> {
    let entry = Entry::find()
        .filter(EntryColumn::Id.eq(id))
        .filter(EntryColumn::DeletedAt.is_null())
        .filter(EntryColumn::UserId.eq(user_id))
        .inner_join(User)
        .filter(UserColumn::DeletedAt.is_null())
        .filter(UserColumn::Id.eq(user_id))
        .inner_join(Title)
        .filter(TitleColumn::IsVisible.eq(true))
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

pub async fn recover_entry(db: &DbConn, user_id: i32, id: i32) -> Result<()> {
    let entry = Entry::find()
        .filter(EntryColumn::Id.eq(id))
        .filter(EntryColumn::UserId.eq(user_id))
        .inner_join(User)
        .filter(UserColumn::DeletedAt.is_null())
        .filter(UserColumn::Id.eq(user_id))
        .inner_join(Title)
        .filter(TitleColumn::IsVisible.eq(true))
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

pub async fn get_entry(db: &DbConn, id: i32, user_id: Option<i32>) -> Result<EntryDto> {
    let entry = Entry::find()
        .filter(EntryColumn::Id.eq(id))
        .filter(EntryColumn::DeletedAt.is_null())
        .one(db)
        .await
        .map_err(|_| Error::InternalError("Girdi bulunamadı.".to_string()))
        .and_then(|entry| entry.ok_or(Error::NotFound("Girdi bulunamadı.".to_string())))?;

    let author = User::find()
        .filter(UserColumn::Id.eq(entry.user_id))
        .one(db)
        .await
        .map_err(|_| Error::InternalError("Kullanıcı bulunamadı.".to_string()))
        .and_then(|user| user.ok_or(Error::NotFound("Kullanıcı bulunamadı.".to_string())))?;

    let user: Option<UserModel> = match user_id {
        Some(user_id) => {
            let user = User::find()
                .filter(UserColumn::Id.eq(user_id))
                .one(db)
                .await
                .map_err(|_| Error::InternalError("Kullanıcı bulunamadı.".to_string()))
                .and_then(|user| {
                    user.ok_or(Error::NotFound("Kullanıcı bulunamadı.".to_string()))
                })?;

            Some(user)
        }
        None => None,
    };

    let is_favorite: Option<bool> = match user {
        Some(user) => Some(
            Favorite::find()
                .filter(FavoriteColumn::UserId.eq(user.id))
                .filter(FavoriteColumn::EntryId.eq(entry.id))
                .one(db)
                .await
                .map_err(|_| Error::InternalError("Favori bulunamadı.".to_string()))
                .map(|favorite| favorite.is_some())?,
        ),
        None => None,
    };

    let title = Title::find()
        .filter(TitleColumn::Id.eq(entry.title_id))
        .one(db)
        .await
        .map_err(|_| Error::InternalError("Başlık bulunamadı.".to_string()))
        .and_then(|title| title.ok_or(Error::NotFound("Başlık bulunamadı.".to_string())))?;

    Ok(EntryDto {
        id: entry.id,
        title: EntryTitleDto {
            id: title.id,
            name: title.name,
        },
        content: entry.content,
        author: EntryAuthorDto {
            id: author.id,
            nickname: author.nickname,
            is_faded: author.is_faded,
        },
        is_favorite,
        vote: None,
        created_at: entry.created_at.and_utc().to_string(),
        updated_at: entry.updated_at.map(|t| t.and_utc().to_string()),
    })
}

pub async fn update_entry(
    db: &DbConn,
    user_id: i32,
    id: i32,
    request: UpdateEntryRequest,
) -> Result<()> {
    request.validate().map_err(|_| {
        Error::InvalidRequest("Geçersiz istek. Lütfen girilen bilgileri kontrol edin.".to_string())
    })?;

    let entry = Entry::find()
        .filter(EntryColumn::Id.eq(id))
        .filter(EntryColumn::DeletedAt.is_null())
        .filter(EntryColumn::UserId.eq(user_id))
        .inner_join(User)
        .filter(UserColumn::DeletedAt.is_null())
        .filter(UserColumn::Id.eq(user_id))
        .inner_join(Title)
        .filter(TitleColumn::IsVisible.eq(true))
        .one(db)
        .await
        .map_err(|_| Error::InternalError("Girdi bulunamadı.".to_string()))
        .and_then(|entry| entry.ok_or(Error::NotFound("Girdi bulunamadı.".to_string())))?;

    let mut entry: EntryActiveModel = entry.into();
    entry.content = Set(request.content);
    entry.updated_at = Set(Some(chrono::Utc::now().naive_utc()));

    entry
        .save(db)
        .await
        .map_err(|_| Error::InternalError("Girdi güncellenemedi.".to_string()))?;

    Ok(())
}

pub async fn migrate_entry(db: &DbConn, id: i32, title_id: i32) -> Result<()> {
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
        .update(db)
        .await
        .map_err(|_| Error::InternalError("Girdi taşınamadı.".to_string()))?;

    Ok(())
}

pub async fn get_title_entries(
    db: &DbConn,
    id: i32,
    user_id: Option<i32>,
    query: GetTitleEntriesQuery,
) -> Result<PaginationResponse<EntryDto>> {
    query.validate().map_err(|_| {
        Error::InvalidRequest("Geçersiz istek. Lütfen girilen bilgileri kontrol edin.".to_string())
    })?;

    let user: Option<UserModel> = match user_id {
        Some(user_id) => {
            let user = User::find()
                .filter(UserColumn::Id.eq(user_id))
                .filter(UserColumn::DeletedAt.is_null())
                .one(db)
                .await
                .map_err(|_| Error::InternalError("Kullanıcı bulunamadı.".to_string()))
                .and_then(|user| {
                    user.ok_or(Error::NotFound("Kullanıcı bulunamadı.".to_string()))
                })?;

            Some(user)
        }
        None => None,
    };

    let base_query = Entry::find()
        .filter(EntryColumn::TitleId.eq(id))
        .filter(EntryColumn::DeletedAt.is_null())
        .apply_if(Some(query.from), |q, v| match v {
            Some(v) => q.filter(EntryColumn::CreatedAt.gt(v)),
            None => q,
        })
        .apply_if(Some(query.to), |q, v| match v {
            Some(v) => q.filter(EntryColumn::CreatedAt.lt(v)),
            None => q,
        })
        .apply_if(Some(query.order_by), |q, v| match v {
            Some(OrderBy::CreatedAt) => match query.order {
                Some(order::Order::Asc) => q.order_by_asc(EntryColumn::CreatedAt),
                Some(order::Order::Desc) => q.order_by_desc(EntryColumn::CreatedAt),
                None => q,
            },
            Some(OrderBy::UpdatedAt) => match query.order {
                Some(order::Order::Asc) => q.order_by_asc(EntryColumn::UpdatedAt),
                Some(order::Order::Desc) => q.order_by_desc(EntryColumn::UpdatedAt),
                None => q,
            },
            Some(OrderBy::NetVotes) => match query.order {
                Some(order::Order::Asc) => q.order_by_asc(EntryColumn::NetVotes),
                Some(order::Order::Desc) => q.order_by_desc(EntryColumn::NetVotes),
                None => q,
            },
            None => match query.order {
                Some(order::Order::Desc) => q.order_by_desc(EntryColumn::CreatedAt),
                _ => q.order_by_asc(EntryColumn::CreatedAt),
            },
        })
        .inner_join(User)
        .filter(UserColumn::DeletedAt.is_null())
        .filter(UserColumn::IsFaded.eq(false));

    let entry_pages = base_query.clone().paginate(db, query.per_page.into());

    let entries = entry_pages
        .fetch_page(query.page as u64 - 1)
        .await
        .map_err(|_| Error::InternalError("Girdiler getirilemedi.".to_string()))?;

    let title_name: String = Title::find()
        .filter(TitleColumn::Id.eq(id))
        .filter(TitleColumn::IsVisible.eq(true))
        .select_only()
        .column(TitleColumn::Name)
        .into_tuple()
        .one(db)
        .await
        .map_err(|_| Error::InternalError("Başlık adı getirilemedi.".to_string()))
        .and_then(|title_name: Option<String>| {
            title_name.ok_or(Error::NotFound("Başlık adı getirilemedi.".to_string()))
        })?;

    let entry_dto_futures = entries.into_iter().map(|entry| {
        let db = db.clone();
        let title_name = title_name.clone();
        let user = user.clone();

        async move {
            let author = User::find()
                .filter(UserColumn::Id.eq(entry.user_id))
                .filter(UserColumn::DeletedAt.is_null())
                .filter(UserColumn::IsFaded.eq(false))
                .one(&db)
                .await;

            let is_favorite: Option<bool> = match user {
                Some(user) => Some(
                    Favorite::find()
                        .filter(FavoriteColumn::UserId.eq(user.id))
                        .filter(FavoriteColumn::EntryId.eq(entry.id))
                        .one(&db)
                        .await
                        .map_err(|_| Error::InternalError("Favori bulunamadı.".to_string()))
                        .map(|favorite| favorite.is_some())?,
                ),
                None => None,
            };

            if let Ok(Some(user)) = author {
                Ok(Some(EntryDto {
                    id: entry.id,
                    title: EntryTitleDto {
                        id,
                        name: title_name.clone(),
                    },
                    content: entry.content,
                    author: EntryAuthorDto {
                        id: user.id,
                        nickname: user.nickname,
                        is_faded: user.is_faded,
                    },
                    is_favorite,
                    vote: None,
                    created_at: entry.created_at.and_utc().to_string(),
                    updated_at: entry.updated_at.map(|t| t.and_utc().to_string()),
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

    let total = base_query
        .count(db)
        .await
        .map_err(|_| Error::InternalError("Girdi sayısı getirilemedi.".to_string()))?;

    Ok(PaginationResponse {
        total,
        page: query.page,
        per_page: query.per_page,
        items: entry_dtos,
    })
}

pub async fn get_title_entries_by_name(
    db: &DbConn,
    title_name: &str,
    query: GetTitleEntriesQuery,
    user_id: Option<i32>,
) -> Result<PaginationResponse<EntryDto>> {
    let title = Title::find()
        .filter(TitleColumn::Name.eq(title_name))
        .one(db)
        .await
        .map_err(|_| Error::InternalError("Başlık bulunamadı.".to_string()))
        .and_then(|title| title.ok_or(Error::NotFound("Başlık bulunamadı.".to_string())))?;

    get_title_entries(db, title.id, user_id, query).await
}

pub async fn get_user_entries(
    db: &DbConn,
    id: i32,
    query: PaginationQuery,
    user_id: Option<i32>,
) -> Result<PaginationResponse<EntryDto>> {
    query.validate().map_err(|_| {
        Error::InvalidRequest("Geçersiz istek. Lütfen girilen bilgileri kontrol edin.".to_string())
    })?;

    let user: Option<UserModel> = match user_id {
        Some(user_id) => {
            let user = User::find()
                .filter(UserColumn::Id.eq(user_id))
                .filter(UserColumn::DeletedAt.is_null())
                .one(db)
                .await
                .map_err(|_| Error::InternalError("Kullanıcı bulunamadı.".to_string()))
                .and_then(|user| {
                    user.ok_or(Error::NotFound("Kullanıcı bulunamadı.".to_string()))
                })?;

            Some(user)
        }
        None => None,
    };

    let (nickname, is_faded): (String, bool) = User::find()
        .filter(UserColumn::Id.eq(id))
        .filter(UserColumn::DeletedAt.is_null())
        .select_only()
        .column(UserColumn::Nickname)
        .column(UserColumn::IsFaded)
        .into_tuple()
        .one(db)
        .await
        .map_err(|_| Error::InternalError("Kullanıcı bulunamadı.".to_string()))
        .and_then(|user| user.ok_or(Error::NotFound("Kullanıcı bulunamadı.".to_string())))?;

    let base_query = Entry::find()
        .filter(EntryColumn::UserId.eq(id))
        .filter(EntryColumn::DeletedAt.is_null())
        .inner_join(Title)
        .filter(TitleColumn::IsVisible.eq(true));

    let entry_pages = base_query
        .clone()
        .order_by_desc(EntryColumn::CreatedAt)
        .paginate(db, query.per_page.into());

    let entries = entry_pages
        .fetch_page(query.page as u64 - 1)
        .await
        .map_err(|_| Error::InternalError("Girdiler getirilemedi.".to_string()))?;

    let entry_dto_futures = entries.into_iter().map(|entry| {
        let db = db.clone();
        let nickname = nickname.clone();
        let user = user.clone();

        async move {
            let title = Title::find()
                .filter(TitleColumn::Id.eq(entry.title_id))
                .one(&db)
                .await;

            let is_favorite: Option<bool> = match user {
                Some(user) => Some(
                    Favorite::find()
                        .filter(FavoriteColumn::UserId.eq(user.id))
                        .filter(FavoriteColumn::EntryId.eq(entry.id))
                        .one(&db)
                        .await
                        .map_err(|_| Error::InternalError("Favori bulunamadı.".to_string()))
                        .map(|favorite| favorite.is_some())?,
                ),
                None => None,
            };

            if let Ok(Some(title)) = title {
                Ok(Some(EntryDto {
                    id: entry.id,
                    title: EntryTitleDto {
                        id: title.id,
                        name: title.name,
                    },
                    content: entry.content,
                    author: EntryAuthorDto {
                        id,
                        nickname,
                        is_faded,
                    },
                    is_favorite,
                    vote: None,
                    created_at: entry.created_at.and_utc().to_string(),
                    updated_at: entry.updated_at.map(|t| t.and_utc().to_string()),
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

    let total = base_query
        .count(db)
        .await
        .map_err(|_| Error::InternalError("Girdi sayısı getirilemedi.".to_string()))?;

    Ok(PaginationResponse {
        total,
        page: query.page,
        per_page: query.per_page,
        items: entry_dtos,
    })
}

pub async fn favorite_entry(db: &DbConn, user_id: i32, entry_id: i32) -> Result<()> {
    let mut entry = Entry::find()
        .filter(EntryColumn::Id.eq(entry_id))
        .filter(EntryColumn::DeletedAt.is_null())
        .inner_join(Title)
        .filter(TitleColumn::IsVisible.eq(true))
        .one(db)
        .await
        .map_err(|_| Error::InternalError("Girdi bulunamadı.".to_string()))
        .and_then(|entry| entry.ok_or(Error::NotFound("Girdi bulunamadı.".to_string())))?
        .into_active_model();

    User::find()
        .filter(UserColumn::Id.eq(user_id))
        .filter(UserColumn::DeletedAt.is_null())
        .one(db)
        .await
        .map_err(|_| Error::InternalError("Kullanıcı bulunamadı.".to_string()))
        .and_then(|user| user.ok_or(Error::NotFound("Kullanıcı bulunamadı.".to_string())))?;

    // TODO: Use exists instead of find
    let favorite = Favorite::find()
        .filter(FavoriteColumn::UserId.eq(user_id))
        .filter(FavoriteColumn::EntryId.eq(entry_id))
        .one(db)
        .await
        .map_err(|_| Error::InternalError("Favori bulunamadı.".to_string()))?;

    if favorite.is_some() {
        return Err(Error::InvalidRequest(
            "Zaten favorilere eklenmiş.".to_string(),
        ));
    }

    FavoriteActiveModel {
        user_id: Set(user_id),
        entry_id: Set(entry_id),
        ..Default::default()
    }
    .save(db)
    .await
    .map_err(|_| Error::InternalError("Favori eklenemedi.".to_string()))?;

    entry.total_favorites = Set(entry.total_favorites.unwrap() + 1);
    entry
        .update(db)
        .await
        .map_err(|_| Error::InternalError("Favori eklenemedi.".to_string()))?;

    Ok(())
}

pub async fn unfavorite_entry(db: &DbConn, user_id: i32, entry_id: i32) -> Result<()> {
    let mut entry = Entry::find()
        .filter(EntryColumn::Id.eq(entry_id))
        .filter(EntryColumn::DeletedAt.is_null())
        .inner_join(Title)
        .filter(TitleColumn::IsVisible.eq(true))
        .one(db)
        .await
        .map_err(|_| Error::InternalError("Girdi bulunamadı.".to_string()))
        .and_then(|entry| entry.ok_or(Error::NotFound("Girdi bulunamadı.".to_string())))?
        .into_active_model();

    User::find()
        .filter(UserColumn::Id.eq(user_id))
        .filter(UserColumn::DeletedAt.is_null())
        .one(db)
        .await
        .map_err(|_| Error::InternalError("Kullanıcı bulunamadı.".to_string()))
        .and_then(|user| user.ok_or(Error::NotFound("Kullanıcı bulunamadı.".to_string())))?;

    Favorite::find()
        .filter(FavoriteColumn::UserId.eq(user_id))
        .filter(FavoriteColumn::EntryId.eq(entry_id))
        .one(db)
        .await
        .map_err(|_| Error::InternalError("Favori bulunamadı.".to_string()))
        .and_then(|favorite| favorite.ok_or(Error::NotFound("Favori bulunamadı.".to_string())))?
        .delete(db)
        .await
        .map_err(|_| Error::InternalError("Favori silinemedi.".to_string()))?;

    entry.total_favorites = Set(entry.total_favorites.unwrap() - 1);
    entry
        .update(db)
        .await
        .map_err(|_| Error::InternalError("Favori silinemedi.".to_string()))?;

    Ok(())
}
