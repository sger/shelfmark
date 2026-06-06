use axum::{extract::{Path, State}, routing::{get, patch, post}, Json, Router};
use uuid::Uuid;

use crate::{
    error::{AppError, AppResult},
    models::{CreateLibraryRequest, ImportJob, Library, UpdateLibraryRequest},
    routes::auth::CurrentUser,
    services::books,
    state::AppState,
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(list_libraries).post(create_library))
        .route("/{id}", patch(update_library).delete(delete_library))
        .route("/{id}/scan", post(scan_library))
}

fn require_admin(user: &CurrentUser) -> AppResult<()> {
    if user.0.role == "admin" { Ok(()) } else { Err(AppError::Forbidden) }
}

#[utoipa::path(get, path = "/api/v1/libraries", responses((status = 200, body = [Library])), tag = "libraries")]
pub async fn list_libraries(_user: CurrentUser, State(state): State<AppState>) -> AppResult<Json<Vec<Library>>> {
    let libraries = sqlx::query_as!(
        Library,
        "SELECT id, name, path, created_at, updated_at FROM libraries ORDER BY name",
    )
    .fetch_all(&state.db)
    .await?;
    Ok(Json(libraries))
}

#[utoipa::path(post, path = "/api/v1/libraries", request_body = CreateLibraryRequest, responses((status = 200, body = Library)), tag = "libraries")]
pub async fn create_library(user: CurrentUser, State(state): State<AppState>, Json(input): Json<CreateLibraryRequest>) -> AppResult<Json<Library>> {
    require_admin(&user)?;
    if input.name.trim().is_empty() || input.path.trim().is_empty() {
        return Err(AppError::BadRequest("name and path are required".to_string()));
    }
    let library = sqlx::query_as!(
        Library,
        "INSERT INTO libraries (name, path) VALUES ($1, $2)
         RETURNING id, name, path, created_at, updated_at",
        input.name.trim(),
        input.path.trim(),
    )
    .fetch_one(&state.db)
    .await?;
    Ok(Json(library))
}

#[utoipa::path(patch, path = "/api/v1/libraries/{id}", request_body = UpdateLibraryRequest, responses((status = 200, body = Library)), tag = "libraries")]
pub async fn update_library(user: CurrentUser, State(state): State<AppState>, Path(id): Path<Uuid>, Json(input): Json<UpdateLibraryRequest>) -> AppResult<Json<Library>> {
    require_admin(&user)?;
    let existing = sqlx::query_as!(
        Library,
        "SELECT id, name, path, created_at, updated_at FROM libraries WHERE id = $1",
        id,
    )
    .fetch_optional(&state.db)
    .await?
    .ok_or(AppError::NotFound)?;

    let library = sqlx::query_as!(
        Library,
        "UPDATE libraries SET name = $1, path = $2, updated_at = now() WHERE id = $3
         RETURNING id, name, path, created_at, updated_at",
        input.name.unwrap_or(existing.name),
        input.path.unwrap_or(existing.path),
        id,
    )
    .fetch_one(&state.db)
    .await?;
    Ok(Json(library))
}

#[utoipa::path(delete, path = "/api/v1/libraries/{id}", responses((status = 200)), tag = "libraries")]
pub async fn delete_library(user: CurrentUser, State(state): State<AppState>, Path(id): Path<Uuid>) -> AppResult<Json<serde_json::Value>> {
    require_admin(&user)?;
    sqlx::query!("DELETE FROM libraries WHERE id = $1", id).execute(&state.db).await?;
    Ok(Json(serde_json::json!({ "ok": true })))
}

#[utoipa::path(post, path = "/api/v1/libraries/{id}/scan", responses((status = 200, body = ImportJob)), tag = "libraries")]
pub async fn scan_library(user: CurrentUser, State(state): State<AppState>, Path(id): Path<Uuid>) -> AppResult<Json<ImportJob>> {
    require_admin(&user)?;
    let library = sqlx::query_as!(
        Library,
        "SELECT id, name, path, created_at, updated_at FROM libraries WHERE id = $1",
        id,
    )
    .fetch_optional(&state.db)
    .await?
    .ok_or(AppError::NotFound)?;

    let job = books::create_job(&state.db, "scan", Some(id), Some("Library scan queued")).await?;
    let scan_state = state.clone();
    let job_id = job.id;
    tokio::spawn(async move {
        let _ = books::finish_job(&scan_state.db, job_id, "running", Some("Scanning library"), None).await;
        books::scan_library(scan_state, library, job_id).await;
    });
    Ok(Json(job))
}
