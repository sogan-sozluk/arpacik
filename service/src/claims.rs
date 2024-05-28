use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct UserClaims {
    pub id: i32,
    pub nickname: String,
    pub email: String,
    pub is_admin: bool,
    pub is_moderator: bool,
    pub is_author: bool,
    pub iat: i64,
    pub exp: i64,
}
