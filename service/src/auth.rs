use crate::dto::auth::RegisterRequest;
use crate::error::{Error, Result};
use ::entity::prelude::*;
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHasher, SaltString},
    Argon2,
};
use sea_orm::*;
use validator::Validate;

pub async fn register(db: &DbConn, register_request: RegisterRequest) -> Result<UserActiveModel> {
    match register_request.validate() {
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
        .hash_password(register_request.password.as_bytes(), &salt)
        .unwrap()
        .to_string();

    let result = UserActiveModel {
        nickname: Set(register_request.nickname),
        email: Set(register_request.email),
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
