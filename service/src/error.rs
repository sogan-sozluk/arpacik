use serde::{Deserialize, Serialize};
use thiserror::Error;

pub type Result<T> = core::result::Result<T, Error>;

#[derive(Serialize, Deserialize)]
pub struct ErrorResponse {
    pub code: u16,
    pub error: String,
    pub details: Option<String>,
}

#[derive(Error, Debug)]
pub enum Error {
    #[error("Sunucu hatası: {0}")]
    InternalError(String),
    #[error("Geçersiz istek: {0}")]
    InvalidRequest(String),
    #[error("Geçersiz kullanıcı adı veya parola")]
    InvalidCredentials,
    #[error("Geçersiz token")]
    InvalidToken,
    #[error("Kimlik doğrulama hatası: {0}")]
    AuthError(String),
}

pub trait IntoErrorResponse {
    fn into_error_response(self) -> ErrorResponse;
}

impl IntoErrorResponse for Error {
    fn into_error_response(self) -> ErrorResponse {
        match self {
            Error::InternalError(e) => ErrorResponse {
                code: 500,
                error: "Sunucu hatası".to_string(),
                details: Some(e.to_string()),
            },
            Error::InvalidRequest(e) => ErrorResponse {
                code: 400,
                error: "Geçersiz istek".to_string(),
                details: Some(e.to_string()),
            },
            Error::InvalidCredentials => ErrorResponse {
                code: 401,
                error: "Geçersiz kullanıcı adı veya parola".to_string(),
                details: None,
            },
            Error::InvalidToken => ErrorResponse {
                code: 401,
                error: "Geçersiz token".to_string(),
                details: Some("Token geçerli değil veya süresi dolmuş olabilir.".to_string()),
            },
            Error::AuthError(e) => ErrorResponse {
                code: 401,
                error: e.split(':').next().unwrap().to_string(),
                details: Some(e.to_string()),
            },
        }
    }
}
