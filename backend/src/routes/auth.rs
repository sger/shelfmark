use axum::{
    extract::{FromRef, FromRequestParts, State},
    http::{request::Parts, HeaderMap},
    routing::{get, post},
    Json, Router,
};

use crate::{
    error::{AppError, AppResult},
    models::{AuthResponse, Claims, LoginRequest, RegisterRequest, User},
    services::auth as auth_service,
    state::AppState,
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/register", post(register))
        .route("/login", post(login))
        .route("/logout", post(logout))
        .route("/me", get(me))
}

#[derive(Clone, Debug)]
pub struct CurrentUser(pub User);

impl<S> FromRequestParts<S> for CurrentUser
where
    AppState: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let state = AppState::from_ref(state);
        let token = bearer(&parts.headers).ok_or(AppError::Unauthorized)?;
        let claims: Claims = auth_service::decode_token(&state, token)?;
        let user = auth_service::get_user(&state.db, claims.sub).await?;
        Ok(CurrentUser(user))
    }
}

fn bearer(headers: &HeaderMap) -> Option<&str> {
    headers
        .get("Authorization")?
        .to_str()
        .ok()?
        .strip_prefix("Bearer ")
}

#[utoipa::path(post, path = "/api/v1/auth/register", request_body = RegisterRequest, responses((status = 200, body = AuthResponse)), tag = "auth")]
pub async fn register(State(state): State<AppState>, Json(input): Json<RegisterRequest>) -> AppResult<Json<AuthResponse>> {
    let email = auth_service::normalize_email(&input.email);
    if email.is_empty() || input.password.len() < 8 || input.display_name.trim().is_empty() {
        return Err(AppError::BadRequest("email, display name, and an 8+ character password are required".to_string()));
    }

    let password_hash = auth_service::hash_password(&input.password)?;
    let mut tx = state.db.begin().await?;
    let user_count: i64 = sqlx::query_scalar!(r#"SELECT COUNT(*) AS "count!" FROM users"#)
        .fetch_one(&mut *tx)
        .await?;
    let role = if user_count == 0 { "admin" } else { "user" };

    let user = sqlx::query_as!(
        User,
        "INSERT INTO users (email, password_hash, display_name, role)
         VALUES ($1, $2, $3, $4)
         RETURNING id, email, display_name, role, created_at",
        email,
        password_hash,
        input.display_name.trim(),
        role,
    )
    .fetch_one(&mut *tx)
    .await
    .map_err(map_registration_error)?;
    tx.commit().await?;

    let token = auth_service::issue_token(&state, &user)?;
    Ok(Json(AuthResponse { token, user }))
}

fn map_registration_error(error: sqlx::Error) -> AppError {
    if let sqlx::Error::Database(database_error) = &error {
        if database_error.constraint() == Some("users_email_key") {
            return AppError::BadRequest("an account with this email already exists".to_string());
        }
    }
    AppError::Database(error)
}

#[utoipa::path(post, path = "/api/v1/auth/login", request_body = LoginRequest, responses((status = 200, body = AuthResponse)), tag = "auth")]
pub async fn login(State(state): State<AppState>, Json(input): Json<LoginRequest>) -> AppResult<Json<AuthResponse>> {
    let email = auth_service::normalize_email(&input.email);
    let row = sqlx::query!(
        "SELECT id, email, password_hash, display_name, role, created_at FROM users WHERE email = $1",
        email,
    )
    .fetch_optional(&state.db)
    .await?
    .ok_or(AppError::Unauthorized)?;

    if !auth_service::verify_password(&input.password, &row.password_hash)? {
        return Err(AppError::Unauthorized);
    }

    let user = User {
        id: row.id,
        email: row.email,
        display_name: row.display_name,
        role: row.role,
        created_at: row.created_at,
    };
    let token = auth_service::issue_token(&state, &user)?;
    Ok(Json(AuthResponse { token, user }))
}

#[utoipa::path(post, path = "/api/v1/auth/logout", responses((status = 200)), tag = "auth")]
pub async fn logout(_user: CurrentUser) -> Json<serde_json::Value> {
    Json(serde_json::json!({ "ok": true }))
}

#[utoipa::path(get, path = "/api/v1/auth/me", responses((status = 200, body = User)), tag = "auth")]
pub async fn me(CurrentUser(user): CurrentUser) -> Json<User> {
    Json(user)
}
