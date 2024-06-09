use axum::http::HeaderMap;

pub trait HeadersCookie {
    fn cookie(&self) -> Option<String>;
}

impl HeadersCookie for HeaderMap {
    fn cookie(&self) -> Option<String> {
        let cookie = self.get("cookie")?;
        let cookie = cookie.to_str().ok()?;

        Some(cookie.to_string())
    }
}
