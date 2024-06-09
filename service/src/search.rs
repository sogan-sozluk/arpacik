use ::entity::prelude::*;
use sea_orm::*;

use crate::{
    dto::search::{SearchItem, SearchQuery},
    Error, Result,
};

pub async fn search(db: &DbConn, sq: SearchQuery) -> Result<Vec<SearchItem>> {
    let query = sq.query;
    if query.starts_with('@') {
        let users: Vec<SearchItem> = User::find()
            .filter(UserColumn::Nickname.contains(query.trim_start_matches('@')))
            .filter(UserColumn::DeletedAt.is_null())
            .offset(0)
            .limit(10)
            .all(db)
            .await
            .map_err(|_| Error::InternalError("Kullanıcı bulunamadı.".to_string()))
            .map(|users| {
                users
                    .into_iter()
                    .map(|user| SearchItem {
                        id: user.id,
                        name: user.nickname,
                    })
                    .collect()
            })?;

        Ok(users)
    } else {
        let titles: Vec<SearchItem> = Title::find()
            .filter(TitleColumn::Name.contains(query))
            .filter(TitleColumn::IsVisible.eq(true))
            .offset(0)
            .limit(10)
            .all(db)
            .await
            .map_err(|_| Error::InternalError("Başlık bulunamadı.".to_string()))
            .map(|tags| {
                tags.into_iter()
                    .map(|tag| SearchItem {
                        id: tag.id,
                        name: tag.name,
                    })
                    .collect()
            })?;

        Ok(titles)
    }
}
