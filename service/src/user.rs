use crate::error::{Error, Result};
use ::entity::prelude::*;
use sea_orm::*;

pub async fn user_by_token(db: &DbConn, token: &str) -> Result<UserModel> {
    let token_hash = blake3::hash(token.as_bytes()).to_hex().to_string();

    let user_id = Token::find()
        .filter(TokenColumn::Hash.eq(token_hash))
        .one(db)
        .await
        .map_err(|_| Error::InvalidToken)
        .and_then(|token| token.ok_or(Error::InvalidToken))?
        .user_id;

    User::find()
        .filter(UserColumn::Id.eq(user_id))
        .filter(UserColumn::DeletedAt.is_null())
        .one(db)
        .await
        .map_err(|_| Error::InvalidToken)
        .and_then(|user| user.ok_or(Error::InvalidToken))
}
