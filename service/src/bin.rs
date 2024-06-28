use ::entity::prelude::*;
use sea_orm::*;
use validator::Validate;

use crate::{
    dto::{
        entry::{EntryAuthorDto, EntryDto, EntryTitleDto},
        pagination::{PaginationQuery, PaginationResponse},
    },
    Error, Result,
};

pub async fn get_user_bin(
    db: &DatabaseConnection,
    user_id: i32,
    query: PaginationQuery,
) -> Result<PaginationResponse<EntryDto>> {
    query.validate().map_err(|_| {
        Error::InvalidRequest("Geçersiz istek. Lütfen girilen bilgileri kontrol edin.".to_string())
    })?;

    let user = User::find()
        .filter(UserColumn::Id.eq(user_id))
        .filter(UserColumn::DeletedAt.is_null())
        .one(db)
        .await
        .map_err(|_| Error::InternalError("Kullanıcı bulunamadı.".to_string()))
        .and_then(|user| user.ok_or(Error::NotFound("Kullanıcı bulunamadı.".to_string())))?;

    let base_query = Entry::find()
        .filter(EntryColumn::UserId.eq(user_id))
        .filter(EntryColumn::DeletedAt.is_not_null())
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
        let user = user.clone();

        async move {
            let title = Title::find()
                .filter(TitleColumn::Id.eq(entry.title_id))
                .one(&db)
                .await;

            let is_favorite: Option<bool> = Some(
                Favorite::find()
                    .filter(FavoriteColumn::UserId.eq(user.id))
                    .filter(FavoriteColumn::EntryId.eq(entry.id))
                    .one(&db)
                    .await
                    .map_err(|_| Error::InternalError("Favori bulunamadı.".to_string()))
                    .map(|favorite| favorite.is_some())?,
            );

            let vote: Option<Rating> = Vote::find()
                .filter(VoteColumn::UserId.eq(user.id))
                .filter(VoteColumn::EntryId.eq(entry.id))
                .one(&db)
                .await
                .map_err(|_| Error::InternalError("Oy bulunamadı.".to_string()))
                .map(|vote| vote.map(|vote| vote.rating))?;

            if let Ok(Some(title)) = title {
                Ok(Some(EntryDto {
                    id: entry.id,
                    title: EntryTitleDto {
                        id: title.id,
                        name: title.name,
                    },
                    content: entry.content,
                    author: EntryAuthorDto {
                        id: user.id,
                        nickname: user.nickname,
                        is_faded: user.is_faded,
                    },
                    is_favorite,
                    vote,
                    created_at: entry.created_at.and_utc().to_string(),
                    updated_at: entry.updated_at.map(|t| t.and_utc().to_string()),
                    deleted_at: entry.deleted_at.map(|t| t.and_utc().to_string()),
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

pub async fn empty_user_bin(db: &DatabaseConnection, user_id: i32) -> Result<()> {
    User::find()
        .filter(UserColumn::Id.eq(user_id))
        .filter(UserColumn::DeletedAt.is_null())
        .one(db)
        .await
        .map_err(|_| Error::InternalError("Kullanıcı bulunamadı.".to_string()))
        .and_then(|user| user.ok_or(Error::NotFound("Kullanıcı bulunamadı.".to_string())))?;

    Entry::delete_many()
        .filter(EntryColumn::UserId.eq(user_id))
        .filter(EntryColumn::DeletedAt.is_not_null())
        .exec(db)
        .await
        .map_err(|_| Error::InternalError("Girdiler silinemedi.".to_string()))?;

    Ok(())
}
