use std::env;

use crate::cookie::Cookie;
use crate::dto::auth::{LoginRequest, LoginResponse, RegisterRequest};
use crate::token::{is_admin, is_moderator, validate_token, UserClaims};
use crate::{Error, Result};
use ::entity::prelude::*;
use argon2::PasswordVerifier;
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHasher, SaltString},
    Argon2,
};
use chrono::TimeZone;
use sea_orm::*;
use validator::Validate;

#[derive(Clone, Copy)]
pub enum AuthHeader {
    Cookie,
    Authorization,
}

pub enum Role {
    Admin,
    Moderator,
    User,
}

pub fn authorize(token: &str, jwt_secret: &str, role: Role) -> bool {
    let is_valid = validate_token(token, jwt_secret);

    match role {
        Role::Admin => is_admin(token, jwt_secret) && is_valid,
        Role::Moderator => is_moderator(token, jwt_secret) && is_valid,
        Role::User => is_valid,
    }
}

pub async fn register(db: &DbConn, request: RegisterRequest) -> Result<UserActiveModel> {
    match request.validate() {
        Ok(_) => (),
        Err(_) => {
            return Err(Error::InvalidRequest(
                "Geçersiz istek. Lütfen girilen bilgileri kontrol edin.".to_string(),
            ));
        }
    }

    let argon2 = Argon2::default();
    let salt = SaltString::generate(&mut OsRng);
    let password_hash = argon2
        .hash_password(request.password.as_bytes(), &salt)
        .unwrap()
        .to_string();

    let result = UserActiveModel {
        nickname: Set(request.nickname),
        email: Set(request.email),
        password_hash: Set(password_hash),
        ..Default::default()
    }
    .save(db)
    .await;

    match result {
        Ok(user) => Ok(user),
        Err(e) => {
            if e.to_string().contains("duplicate key value") {
                return Err(Error::InvalidRequest(
                    "Bu e-posta adresi veya kullanıcı adı zaten kullanımda.".to_string(),
                ));
            }
            Err(Error::InternalError(e.to_string()))
        }
    }
}

pub async fn login(db: &DbConn, request: LoginRequest) -> Result<LoginResponse> {
    match request.validate() {
        Ok(_) => (),
        Err(_) => {
            return Err(Error::InvalidRequest(
                "Geçersiz istek. Lütfen girilen bilgileri kontrol edin.".to_string(),
            ));
        }
    }

    let user = User::find()
        .filter(UserColumn::Nickname.contains(request.nickname))
        .filter(UserColumn::DeletedAt.is_null())
        .one(db)
        .await
        .map_err(|_| Error::InternalError("Kullanıcı bulunamadı.".to_string()))?;

    let user = user.ok_or(Error::InvalidCredentials)?;
    let parsed_hash = match argon2::PasswordHash::new(&user.password_hash) {
        Ok(parsed_hash) => parsed_hash,
        Err(_) => {
            return Err(Error::InternalError(
                "Kullanıcı parolası hatalı.".to_string(),
            ))
        }
    };

    let argon2 = Argon2::default();
    if argon2
        .verify_password(request.password.as_bytes(), &parsed_hash)
        .is_err()
    {
        return Err(Error::InvalidCredentials);
    }

    let claims = UserClaims {
        id: user.id,
        nickname: user.nickname,
        email: user.email,
        is_admin: user.is_admin,
        is_moderator: user.is_moderator,
        is_faded: user.is_faded,
        iat: chrono::Utc::now().timestamp(),
        exp: chrono::Utc::now().timestamp() + 60 * 60 * 24,
    };

    let key = match env::var("JWT_SECRET") {
        Ok(key) => key.into_bytes(),
        Err(_) => return Err(Error::InternalError("JWT_SECRET is not set".to_string())),
    };

    let token = jsonwebtoken::encode(
        &jsonwebtoken::Header::default(),
        &claims,
        &jsonwebtoken::EncodingKey::from_secret(&key),
    );

    let token = match token {
        Ok(token) => token,
        Err(_) => return Err(Error::InternalError("Token oluşturulamadı.".to_string())),
    };

    let token_hash = blake3::hash(token.as_bytes()).to_hex().to_string();

    let result = TokenActiveModel {
        user_id: Set(user.id),
        hash: Set(token_hash),
        ..Default::default()
    }
    .save(db)
    .await;

    match result {
        Ok(_) => (),
        Err(e) => {
            return Err(Error::InternalError(e.to_string()));
        }
    }

    let cookie = Cookie::new("token", &token)
        .path("/")
        .http_only(true)
        .secure(true);

    let response = LoginResponse {
        cookie,
        token,
        is_admin: user.is_admin,
        is_moderator: user.is_moderator,
    };

    Ok(response)
}

pub async fn logout(db: &DbConn, token: &str) -> Result<Cookie> {
    let token_hash = blake3::hash(token.as_bytes()).to_hex().to_string();

    let token = Token::find()
        .filter(TokenColumn::Hash.eq(token_hash))
        .filter(TokenColumn::InvalidatedAt.is_null())
        .one(db)
        .await
        .map_err(|_| Error::InternalError("Token bulunamadı.".to_string()))?;

    let token = token.ok_or(Error::InvalidToken)?;

    let mut token = token.into_active_model();
    token.invalidated_at = Set(Some(chrono::Utc::now().naive_utc()));

    let result = token.save(db).await;

    let expires = Some(chrono::Utc.with_ymd_and_hms(1970, 1, 1, 0, 0, 0).unwrap());
    let cookie = Cookie::new("token", "").path("/").expires(expires);

    match result {
        Ok(_) => Ok(cookie),
        Err(e) => Err(Error::InternalError(e.to_string())),
    }
}

async fn _is_token_invalidated(db: &DbConn, token: &str) -> bool {
    let token_hash = blake3::hash(token.as_bytes()).to_hex().to_string();

    let token = Token::find()
        .filter(TokenColumn::Hash.eq(token_hash))
        .filter(TokenColumn::InvalidatedAt.is_null())
        .one(db)
        .await
        .map_err(|_| Error::InternalError("Token bulunamadı.".to_string()))
        .unwrap();

    token.is_none()
}
