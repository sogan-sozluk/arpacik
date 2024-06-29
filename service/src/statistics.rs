use ::entity::prelude::*;
use sea_orm::*;

use crate::{dto::statistics::StatisticsResponse, Error, Result};

pub async fn statistics(db: &DbConn) -> Result<StatisticsResponse> {
    let title_count = Title::find()
        .count(db)
        .await
        .map_err(|_| Error::InternalError("Başlık sayısı getirilemedi.".to_string()))?;
    let entry_count = Entry::find()
        .count(db)
        .await
        .map_err(|_| Error::InternalError("Girdi sayısı getirilemedi.".to_string()))?;
    let user_count = User::find()
        .count(db)
        .await
        .map_err(|_| Error::InternalError("Kullanıcı sayısı getirilemedi.".to_string()))?;
    let crew_count = User::find()
        .filter(
            UserColumn::IsModerator
                .eq(true)
                .or(UserColumn::IsAdmin.eq(true)),
        )
        .count(db)
        .await
        .map_err(|_| Error::InternalError("Ekip sayısı getirilemedi.".to_string()))?;

    Ok(StatisticsResponse {
        title_count,
        entry_count,
        user_count,
        crew_count,
    })
}
