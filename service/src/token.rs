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

pub fn is_token_valid(token: &str, key: &str) -> bool {
    let key = jsonwebtoken::DecodingKey::from_secret(key.as_bytes());
    let validation = jsonwebtoken::Validation::default();
    let token = jsonwebtoken::decode::<UserClaims>(token, &key, &validation);

    token.is_ok()
}

pub enum TokenField {
    Id,
    Nickname,
    Email,
    IsAdmin,
    IsModerator,
    IsAuthor,
    Iat,
    Exp,
}

pub fn get_field_from_token(token: &str, field: TokenField, key: &str) -> Option<String> {
    let key = jsonwebtoken::DecodingKey::from_secret(key.as_bytes());
    let validation = jsonwebtoken::Validation::default();
    let token = jsonwebtoken::decode::<UserClaims>(token, &key, &validation);

    match token {
        Ok(token) => match field {
            TokenField::Id => Some(token.claims.id.to_string()),
            TokenField::Nickname => Some(token.claims.nickname),
            TokenField::Email => Some(token.claims.email),
            TokenField::IsAdmin => Some(token.claims.is_admin.to_string()),
            TokenField::IsModerator => Some(token.claims.is_moderator.to_string()),
            TokenField::IsAuthor => Some(token.claims.is_author.to_string()),
            TokenField::Iat => Some(token.claims.iat.to_string()),
            TokenField::Exp => Some(token.claims.exp.to_string()),
        },
        Err(_) => None,
    }
}

pub fn get_id(token: &str, key: &str) -> Option<i32> {
    let id = get_field_from_token(token, TokenField::Id, key);

    match id {
        Some(id) => id.parse::<i32>().ok(),
        None => None,
    }
}

pub fn is_admin(token: &str, key: &str) -> bool {
    let is_admin = get_field_from_token(token, TokenField::IsAdmin, key);

    match is_admin {
        Some(is_admin) => is_admin == "true",
        None => false,
    }
}

pub fn is_moderator(token: &str, key: &str) -> bool {
    let is_moderator = get_field_from_token(token, TokenField::IsModerator, key);

    match is_moderator {
        Some(is_moderator) => is_moderator == "true",
        None => false,
    }
}

pub fn is_author(token: &str, key: &str) -> bool {
    let is_author = get_field_from_token(token, TokenField::IsAuthor, key);

    match is_author {
        Some(is_author) => is_author == "true",
        None => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_token_valid_with_valid_token() {
        let key = "test123";
        let claims = UserClaims {
            id: 1,
            nickname: "test".to_string(),
            email: "john.doe@example.com".to_string(),
            is_admin: false,
            is_moderator: false,
            is_author: false,
            iat: chrono::Utc::now().timestamp(),
            exp: chrono::Utc::now().timestamp() + 60 * 60 * 24,
        };

        let token = jsonwebtoken::encode(
            &jsonwebtoken::Header::default(),
            &claims,
            &jsonwebtoken::EncodingKey::from_secret(key.as_bytes()),
        )
        .unwrap();

        assert!(is_token_valid(&token, key));
    }

    #[test]
    fn test_is_token_valid_with_invalid_token() {
        let key = "test123";
        let claims = UserClaims {
            id: 1,
            nickname: "test".to_string(),
            email: "john.doe@example.com".to_string(),
            is_admin: false,
            is_moderator: false,
            is_author: false,
            iat: chrono::Utc::now().timestamp(),
            exp: chrono::Utc::now().timestamp() + 60 * 60 * 24,
        };

        let token = jsonwebtoken::encode(
            &jsonwebtoken::Header::default(),
            &claims,
            &jsonwebtoken::EncodingKey::from_secret(key.as_bytes()),
        )
        .unwrap();

        assert!(!is_token_valid(&token, "invalid_key"));
    }

    #[test]
    fn test_get_field_from_token() {
        let key = "test123";
        let claims = UserClaims {
            id: 1,
            nickname: "test".to_string(),
            email: "jane.doe@example.com".to_string(),
            is_admin: false,
            is_moderator: true,
            is_author: true,
            iat: chrono::Utc::now().timestamp(),
            exp: chrono::Utc::now().timestamp() + 60 * 60 * 24,
        };

        let token = jsonwebtoken::encode(
            &jsonwebtoken::Header::default(),
            &claims,
            &jsonwebtoken::EncodingKey::from_secret(key.as_bytes()),
        )
        .unwrap();

        assert_eq!(
            get_field_from_token(&token, TokenField::Id, key),
            Some(claims.id.to_string())
        );
        assert_eq!(
            get_field_from_token(&token, TokenField::Nickname, key),
            Some(claims.nickname)
        );
        assert_eq!(
            get_field_from_token(&token, TokenField::Email, key),
            Some(claims.email)
        );
        assert_eq!(
            get_field_from_token(&token, TokenField::IsAdmin, key),
            Some(claims.is_admin.to_string())
        );
        assert_eq!(
            get_field_from_token(&token, TokenField::IsModerator, key),
            Some(claims.is_moderator.to_string())
        );
        assert_eq!(
            get_field_from_token(&token, TokenField::IsAuthor, key),
            Some(claims.is_author.to_string())
        );
        assert_eq!(
            get_field_from_token(&token, TokenField::Iat, key),
            Some(claims.iat.to_string())
        );
        assert_eq!(
            get_field_from_token(&token, TokenField::Exp, key),
            Some(claims.exp.to_string())
        );

        assert_eq!(
            get_field_from_token(&token, TokenField::Id, "invalid_key"),
            None
        );
    }

    #[test]
    fn test_is_admin() {
        let key = "test123";
        let claims = UserClaims {
            id: 1,
            nickname: "test".to_string(),
            email: "john.doe@example.com".to_string(),
            is_admin: true,
            is_moderator: false,
            is_author: false,
            iat: chrono::Utc::now().timestamp(),
            exp: chrono::Utc::now().timestamp() + 60 * 60 * 24,
        };

        let token = jsonwebtoken::encode(
            &jsonwebtoken::Header::default(),
            &claims,
            &jsonwebtoken::EncodingKey::from_secret(key.as_bytes()),
        )
        .unwrap();

        assert!(is_admin(&token, key));
    }
}
