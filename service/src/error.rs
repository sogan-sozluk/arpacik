use thiserror::Error;

pub type Result<T> = core::result::Result<T, Error>;

#[derive(Error, Debug, Clone)]
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
    Unauthorized(String),
    #[error("{0}")]
    NotFound(String),
}
