use ::entity::prelude::*;
use sea_orm::*;
use validator::Validate;

use crate::{
    dto::{
        pagination::{PaginationQuery, PaginationResponse},
        today::TodayTitleDto,
    },
    Error, Result,
};

pub async fn today(
    db: &DbConn,
    query: PaginationQuery,
) -> Result<PaginationResponse<TodayTitleDto>> {
    query.validate().map_err(|_| {
        Error::InvalidRequest("Geçersiz istek. Lütfen girilen bilgileri kontrol edin.".to_string())
    })?;

    let last_midnight = chrono::Utc::now()
        .date_naive()
        .and_hms_opt(0, 0, 0)
        .unwrap();

    let base_query = Title::find()
        .filter(TitleColumn::IsVisible.eq(true))
        .inner_join(Entry)
        .filter(EntryColumn::CreatedAt.gt(last_midnight))
        .group_by(TitleColumn::Id)
        .order_by_desc(TitleColumn::LastEntryAt);

    let title_pages = base_query.clone().paginate(db, query.per_page.into());

    let titles = title_pages
        .fetch_page(query.page as u64 - 1)
        .await
        .map_err(|_| Error::InternalError("Başlıklar getirilemedi.".to_string()))?;

    let today_dto_futures = titles.into_iter().map(|title| async move {
        let entry_count = Entry::find()
            .filter(EntryColumn::TitleId.eq(title.id))
            .filter(EntryColumn::CreatedAt.gt(last_midnight))
            .count(db)
            .await;

        if let Ok(entry_count) = entry_count {
            Ok(Some(TodayTitleDto {
                id: title.id,
                name: title.name,
                entry_count,
            }))
        } else {
            Ok(None)
        }
    });

    let today_dtos: Result<Vec<Option<TodayTitleDto>>> =
        futures::future::join_all(today_dto_futures)
            .await
            .into_iter()
            .collect();

    let today_dtos: Vec<TodayTitleDto> = today_dtos?.into_iter().flatten().collect();

    let total = base_query
        .count(db)
        .await
        .map_err(|_| Error::InternalError("Başlık sayısı getirilemedi.".to_string()))?;

    Ok(PaginationResponse {
        total,
        per_page: query.per_page,
        page: query.page,
        items: today_dtos,
    })
}
