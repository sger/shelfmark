use axum::{extract::{Path, State}, routing::get, Json, Router};
use uuid::Uuid;

use crate::{
    error::{AppError, AppResult},
    models::ImportJob,
    routes::auth::CurrentUser,
    state::AppState,
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(list_jobs))
        .route("/{id}", get(get_job))
}

#[utoipa::path(get, path = "/api/v1/jobs", responses((status = 200, body = [ImportJob])), tag = "jobs")]
pub async fn list_jobs(_user: CurrentUser, State(state): State<AppState>) -> AppResult<Json<Vec<ImportJob>>> {
    let jobs = sqlx::query_as!(
        ImportJob,
        "SELECT id, kind, status, message, library_id, book_id, created_at, updated_at
         FROM import_jobs ORDER BY created_at DESC LIMIT 100",
    )
    .fetch_all(&state.db)
    .await?;
    Ok(Json(jobs))
}

#[utoipa::path(get, path = "/api/v1/jobs/{id}", responses((status = 200, body = ImportJob)), tag = "jobs")]
pub async fn get_job(_user: CurrentUser, State(state): State<AppState>, Path(id): Path<Uuid>) -> AppResult<Json<ImportJob>> {
    let job = sqlx::query_as!(
        ImportJob,
        "SELECT id, kind, status, message, library_id, book_id, created_at, updated_at
         FROM import_jobs WHERE id = $1",
        id,
    )
    .fetch_optional(&state.db)
    .await?
    .ok_or(AppError::NotFound)?;
    Ok(Json(job))
}

