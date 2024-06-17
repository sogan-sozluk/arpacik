use axum::http::HeaderMap;
use service::{auth::AuthHeader, cookie::cookie_value};

pub trait HeadersAuth {
    fn cookie(&self) -> Option<String>;
    fn authorization(&self) -> Option<String>;
}

impl HeadersAuth for HeaderMap {
    fn cookie(&self) -> Option<String> {
        let cookie = self.get("cookie")?;
        let cookie = cookie.to_str().ok()?;

        Some(cookie.to_string())
    }

    fn authorization(&self) -> Option<String> {
        let authorization = self.get("authorization")?;
        let authorization = authorization.to_str().ok()?;

        Some(authorization.to_string())
    }
}

pub trait HeaderToken {
    fn token(&self, from: AuthHeader) -> Option<String>;
}

impl HeaderToken for HeaderMap {
    fn token(&self, from: AuthHeader) -> Option<String> {
        match from {
            AuthHeader::Cookie => {
                let cookie = self.cookie()?;
                Some(cookie_value(&cookie, "token")?.to_string())
            }
            AuthHeader::Authorization => {
                let authorization = self.authorization()?;
                let token = authorization.split_whitespace().last()?;

                Some(token.to_string())
            }
        }
    }
}
