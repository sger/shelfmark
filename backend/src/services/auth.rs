use crate::{
    error::{AppError, AppResult},
    models::{Claims, User},
    state::AppState,
};
use argon2::{
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use rand_core::OsRng;
use uuid::Uuid;

pub fn hash_password(password: &str) -> AppResult<String> {
    let salt = SaltString::generate(&mut OsRng);
    Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .map(|hash| hash.to_string())
        .map_err(|err| AppError::BadRequest(format!("invalid password: {err}")))
}

pub fn verify_password(password: &str, hash: &str) -> AppResult<bool> {
    let parsed = PasswordHash::new(hash)
        .map_err(|_| AppError::Unauthorized)?;
    Ok(Argon2::default().verify_password(password.as_bytes(), &parsed).is_ok())
}

pub fn issue_token(state: &AppState, user: &User) -> AppResult<String> {
    let claims = Claims {
        sub: user.id,
        email: user.email.clone(),
        role: user.role.clone(),
        exp: (Utc::now() + Duration::days(14)).timestamp() as usize,
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(state.config.jwt_secret.as_bytes()),
    )
    .map_err(|err| AppError::Internal(anyhow::anyhow!(err)))
}

pub fn decode_token(state: &AppState, token: &str) -> AppResult<Claims> {
    decode::<Claims>(
        token,
        &DecodingKey::from_secret(state.config.jwt_secret.as_bytes()),
        &Validation::default(),
    )
    .map(|data| data.claims)
    .map_err(|_| AppError::Unauthorized)
}

pub async fn get_user(db: &sqlx::PgPool, id: Uuid) -> AppResult<User> {
    sqlx::query_as!(
        User,
        "SELECT id, email, display_name, role, created_at FROM users WHERE id = $1",
        id,
    )
    .fetch_optional(db)
    .await?
    .ok_or(AppError::Unauthorized)
}

pub fn normalize_email(email: &str) -> String {
    email.trim().to_lowercase()
}
