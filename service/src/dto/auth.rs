use crate::{cookie::Cookie, validation::validate_password};
use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct LoginRequest {
    #[validate(length(min = 2, max = 30))]
    pub nickname: String,
    #[validate(length(min = 8, max = 32))]
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LoginResponse {
    pub cookie: Cookie,
    pub token: String,
    pub is_admin: bool,
    pub is_moderator: bool,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct RegisterRequest {
    #[validate(length(min = 2, max = 30))]
    pub nickname: String,
    #[validate(email)]
    pub email: String,
    #[validate(custom(function = validate_password))]
    pub password: String,
}
