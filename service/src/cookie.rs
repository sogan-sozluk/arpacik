use crate::token::{is_admin, is_author, is_moderator, is_token_valid};

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Cookie {
    pub name: String,
    pub value: String,
    pub domain: Option<String>,
    pub path: Option<String>,
    pub expires: Option<chrono::DateTime<chrono::Utc>>,
    pub http_only: bool,
    pub secure: bool,
    pub same_site: Option<String>,
}

impl Cookie {
    pub fn new(name: &str, value: &str) -> Self {
        Self {
            name: name.to_string(),
            value: value.to_string(),
            domain: None,
            path: None,
            expires: None,
            http_only: true,
            secure: true,
            same_site: Some("Strict".to_string()),
        }
    }

    pub fn domain(mut self, domain: &str) -> Self {
        self.domain = Some(domain.to_string());
        self
    }

    pub fn path(mut self, path: &str) -> Self {
        self.path = Some(path.to_string());
        self
    }

    pub fn expires(mut self, expires: Option<chrono::DateTime<chrono::Utc>>) -> Self {
        self.expires = expires;
        self
    }

    pub fn http_only(mut self, http_only: bool) -> Self {
        self.http_only = http_only;
        self
    }

    pub fn secure(mut self, secure: bool) -> Self {
        self.secure = secure;
        self
    }

    pub fn same_site(mut self, same_site: &str) -> Self {
        self.same_site = Some(same_site.to_string());
        self
    }
}

impl std::fmt::Display for Cookie {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}={}; ", self.name, self.value)?;
        if let Some(domain) = &self.domain {
            write!(f, "Domain={}; ", domain)?;
        }
        if let Some(path) = &self.path {
            write!(f, "Path={}; ", path)?;
        }
        if let Some(expires) = &self.expires {
            write!(f, "Expires={}; ", expires.to_rfc2822())?;
        }
        if self.http_only {
            write!(f, "HttpOnly; ")?;
        }
        if self.secure {
            write!(f, "Secure; ")?;
        }
        if let Some(same_site) = &self.same_site {
            write!(f, "SameSite={}; ", same_site)?;
        }

        Ok(())
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct CookieJar {
    cookies: Vec<Cookie>,
}

impl CookieJar {
    pub fn new() -> Self {
        Self { cookies: vec![] }
    }

    pub fn add(&mut self, cookie: Cookie) {
        self.cookies.push(cookie);
    }

    pub fn get(&self, name: &str) -> Option<&Cookie> {
        self.cookies.iter().find(|c| c.name == name)
    }

    pub fn remove(&mut self, name: &str) {
        self.cookies.retain(|c| c.name != name);
    }

    pub fn clear(&mut self) {
        self.cookies.clear();
    }
}

impl std::fmt::Display for CookieJar {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for cookie in &self.cookies {
            write!(f, "{}", cookie)?;
        }
        Ok(())
    }
}

pub fn extract_cookie_value<'a>(cookie: &'a str, name: &str) -> Option<&'a str> {
    let name = format!("{}=", name);
    let start = cookie.find(&name)?;
    let start = start + name.len();
    let end = cookie[start..].find(';').unwrap_or(cookie.len() - start);

    Some(&cookie[start..start + end])
}

pub fn authorize(cookie: &str) -> bool {
    let key = match std::env::var("JWT_SECRET") {
        Ok(key) => key,
        Err(_) => return false,
    };

    let token = match extract_cookie_value(cookie, "token") {
        Some(token) => token,
        None => return false,
    };

    is_token_valid(token, &key)
}

pub fn authorize_admin(cookie: &str) -> bool {
    let key = match std::env::var("JWT_SECRET") {
        Ok(key) => key,
        Err(_) => return false,
    };

    let token = match extract_cookie_value(cookie, "token") {
        Some(token) => token,
        None => return false,
    };

    if !is_token_valid(token, &key) {
        return false;
    }

    is_admin(token, &key)
}

pub fn authorize_moderator(cookie: &str) -> bool {
    let key = match std::env::var("JWT_SECRET") {
        Ok(key) => key,
        Err(_) => return false,
    };

    let token = match extract_cookie_value(cookie, "token") {
        Some(token) => token,
        None => return false,
    };

    if !is_token_valid(token, &key) {
        return false;
    }

    is_moderator(token, &key)
}

pub fn authorize_author(cookie: &str) -> bool {
    let key = match std::env::var("JWT_SECRET") {
        Ok(key) => key,
        Err(_) => return false,
    };

    let token = match extract_cookie_value(cookie, "token") {
        Some(token) => token,
        None => return false,
    };

    if !is_token_valid(token, &key) {
        return false;
    }

    is_author(token, &key)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cookie() {
        let cookie = Cookie::new("name", "value");
        assert_eq!(cookie.name, "name");
        assert_eq!(cookie.value, "value");
        assert_eq!(cookie.domain, None);
        assert_eq!(cookie.path, None);
        assert_eq!(cookie.expires, None);
        assert!(cookie.http_only);
        assert!(cookie.secure);
        assert_eq!(cookie.same_site, Some("Strict".to_string()));
    }

    #[test]
    fn test_cookie_display() {
        let now = chrono::Utc::now();
        let cookie = Cookie {
            name: "name".to_string(),
            value: "value".to_string(),
            domain: Some("example.com".to_string()),
            path: Some("/".to_string()),
            expires: Some(now),
            http_only: true,
            secure: true,
            same_site: Some("Strict".to_string()),
        };
        assert_eq!(
            cookie.to_string(),
            format!(
                "name=value; Domain=example.com; Path=/; Expires={}; HttpOnly; Secure; SameSite=Strict; ",
                now.to_rfc2822()
            )
        );
    }

    #[test]
    fn test_cookie_jar() {
        let mut jar = CookieJar::new();
        assert_eq!(jar.cookies.len(), 0);

        let cookie = Cookie::new("name", "value");
        jar.add(cookie.clone());
        assert_eq!(jar.cookies.len(), 1);
        assert_eq!(jar.get("name"), Some(&cookie));

        jar.remove("name");
        assert_eq!(jar.cookies.len(), 0);

        jar.add(cookie.clone());
        jar.clear();
        assert_eq!(jar.cookies.len(), 0);
    }

    #[test]
    fn test_cookie_jar_display() {
        let mut jar = CookieJar::new();
        let now = chrono::Utc::now();
        jar.add(Cookie {
            name: "name".to_string(),
            value: "value".to_string(),
            domain: Some("example.com".to_string()),
            path: Some("/".to_string()),
            expires: Some(now),
            http_only: true,
            secure: true,
            same_site: Some("Strict".to_string()),
        });
        assert_eq!(
            jar.to_string(),
            format!(
                "name=value; Domain=example.com; Path=/; Expires={}; HttpOnly; Secure; SameSite=Strict; ",
                now.to_rfc2822()
            )
        );
    }

    #[test]
    fn test_extract_value() {
        let cookie_request = "name=value; location=istanbul; theme=dark; lang=en";
        assert_eq!(extract_cookie_value(cookie_request, "name"), Some("value"));
        assert_eq!(
            extract_cookie_value(cookie_request, "location"),
            Some("istanbul")
        );
        assert_eq!(extract_cookie_value(cookie_request, "theme"), Some("dark"));
        assert_eq!(extract_cookie_value(cookie_request, "lang"), Some("en"));

        let cookie_request_no_whitespace = "name=value;location=istanbul;theme=dark;lang=en";
        assert_eq!(
            extract_cookie_value(cookie_request_no_whitespace, "name"),
            Some("value")
        );
        assert_eq!(
            extract_cookie_value(cookie_request_no_whitespace, "location"),
            Some("istanbul")
        );
        assert_eq!(
            extract_cookie_value(cookie_request_no_whitespace, "theme"),
            Some("dark")
        );
        assert_eq!(
            extract_cookie_value(cookie_request_no_whitespace, "lang"),
            Some("en")
        );
    }
}
