use ::entity::prelude::*;
use sea_orm::*;
use sea_query::{Alias, Expr};
use validator::Validate;

use crate::{
    dto::{
        pagination::{PaginationQuery, PaginationResponse},
        trends::TrendTitleDto,
    },
    Error, Result,
};

pub async fn trends(
    db: &DbConn,
    query: PaginationQuery,
) -> Result<PaginationResponse<TrendTitleDto>> {
    query.validate().map_err(|_| {
        Error::InvalidRequest("Geçersiz istek. Lütfen girilen bilgileri kontrol edin.".to_string())
    })?;

    let two_days_ago = chrono::Utc::now()
        .naive_utc()
        .checked_sub_signed(chrono::Duration::days(2))
        .unwrap();

    let base_query = Title::find()
        .filter(TitleColumn::IsVisible.eq(true))
        .select_only()
        .column(TitleColumn::Id)
        .column(TitleColumn::Name)
        .column_as(EntryColumn::Id.count(), "entry_count")
        .join_rev(
            JoinType::InnerJoin,
            Entry::belongs_to(Title)
                .from(EntryColumn::TitleId)
                .to(TitleColumn::Id)
                .into(),
        )
        .filter(EntryColumn::DeletedAt.is_null())
        .group_by(TitleColumn::Id)
        .filter(EntryColumn::CreatedAt.gt(two_days_ago))
        .order_by_desc(Expr::col(Alias::new("entry_count")))
        .into_model::<TrendTitleDto>();

    let title_pages = base_query.paginate(db, query.per_page.into());

    let titles = title_pages
        .fetch_page(query.page as u64 - 1)
        .await
        .map_err(|_| Error::InternalError("Başlıklar getirilemedi.".to_string()))?;

    let trend_dtos = titles
        .into_iter()
        .map(|title| TrendTitleDto {
            id: title.id,
            name: title.name,
            entry_count: title.entry_count,
        })
        .collect();

    let total = title_pages
        .num_items()
        .await
        .map_err(|_| Error::InternalError("Başlıklar sayılamadı.".to_string()))?;

    Ok(PaginationResponse {
        total,
        page: query.page,
        per_page: query.per_page,
        items: trend_dtos,
    })
}
