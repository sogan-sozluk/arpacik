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
}
