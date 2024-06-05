use ::entity::prelude::*;
use sea_orm::*;
use sea_query::{Func, SimpleExpr};

use crate::{
    dto::entry::{EntryAuthorDto, EntryDto, EntryTitleDto},
    Error, Result,
};

pub async fn feed(db: &DbConn) -> Result<Vec<EntryDto>> {
    let select = Entry::find()
        .filter(EntryColumn::DeletedAt.is_null())
        .filter(EntryColumn::NetVotes.gt(0))
        .inner_join(Title)
        .filter(TitleColumn::IsVisible.eq(true))
        .inner_join(User)
        .filter(UserColumn::IsFaded.eq(false))
        .as_query()
        .to_owned()
        .order_by_expr(SimpleExpr::FunctionCall(Func::random()), Order::Asc)
        .offset(0)
        .limit(10)
        .to_owned();

    let statement = db.get_database_backend().build(&select);
    let entries = EntryModel::find_by_statement(statement)
        .all(db)
        .await
        .map_err(|_| Error::InternalError("Gönderiler getirilemedi.".to_string()))?;

    let entry_dto_futures = entries.into_iter().map(|entry| {
        let db = db.clone();

        async move {
            let title = Title::find()
                .filter(TitleColumn::Id.eq(entry.title_id))
                .one(&db)
                .await;

            let (author_id, author_nickname, author_is_faded): (i32, String, bool) = User::find()
                .select_only()
                .column(UserColumn::Id)
                .column(UserColumn::Nickname)
                .column(UserColumn::IsFaded)
                .filter(UserColumn::Id.eq(entry.user_id))
                .into_tuple()
                .one(&db)
                .await
                .map_err(|_| Error::InternalError("Kullanıcı bulunamadı.".to_string()))
                .and_then(|user| {
                    user.ok_or(Error::NotFound("Kullanıcı bulunamadı.".to_string()))
                })?;

            if let Ok(Some(title)) = title {
                Ok(Some(EntryDto {
                    id: entry.id,
                    title: EntryTitleDto {
                        id: title.id,
                        name: title.name,
                    },
                    content: entry.content,
                    author: EntryAuthorDto {
                        id: author_id,
                        nickname: author_nickname,
                        is_faded: author_is_faded,
                    },
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

    Ok(entry_dtos)
}
