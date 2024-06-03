use axum::{http::StatusCode, Json};
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct ErrorBody {
    pub error: String,
    pub details: Option<String>,
}

impl ErrorBody {
    fn new(error: String, details: Option<String>) -> Self {
        ErrorBody { error, details }
    }
}

pub trait IntoErrorResponse {
    fn into_error_response(self) -> (StatusCode, Json<ErrorBody>);
}

impl IntoErrorResponse for service::Error {
    fn into_error_response(self) -> (StatusCode, Json<ErrorBody>) {
        let (status, error, details) = match self {
            service::Error::InternalError(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Sunucu hatası".to_string(),
                None,
            ),
            service::Error::InvalidRequest(e) => (
                StatusCode::BAD_REQUEST,
                "Geçersiz istek".to_string(),
                Some(e.to_string()),
            ),
            service::Error::InvalidCredentials => (
                StatusCode::UNAUTHORIZED,
                "Geçersiz kullanıcı adı veya parola".to_string(),
                Some("Kullanıcı adı veya parola hatalı. Lütfen kontrol edin.".to_string()),
            ),
            service::Error::InvalidToken => (
                StatusCode::UNAUTHORIZED,
                "Geçersiz token".to_string(),
                Some("Token geçerli değil veya süresi dolmuş olabilir.".to_string()),
            ),
            service::Error::Unauthorized(e) => (
                StatusCode::UNAUTHORIZED,
                "Yetkisiz erişim".to_string(),
                Some(e.to_string()),
            ),
            service::Error::NotFound(e) => (
                StatusCode::NOT_FOUND,
                "Bulunamadı".to_string(),
                Some(e.to_string()),
            ),
        };

        (status, Json(ErrorBody::new(error, details)))
    }
}
