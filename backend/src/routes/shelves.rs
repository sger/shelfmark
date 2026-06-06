use axum::{extract::{Path, State}, routing::get, Json, Router};
use uuid::Uuid;

use crate::{
    error::AppResult,
    models::{BookListItem, CreateShelfRequest, ShelfSummary, UpdateShelfRequest},
    routes::auth::CurrentUser,
    services::shelves,
    state::AppState,
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(list_shelves).post(create_shelf))
        .route("/{id}", get(get_shelf_books).patch(update_shelf).delete(delete_shelf))
        .route("/{id}/books", get(get_shelf_books))
}

#[utoipa::path(get, path = "/api/v1/shelves", responses((status = 200, body = [ShelfSummary])), tag = "shelves")]
pub async fn list_shelves(CurrentUser(user): CurrentUser, State(state): State<AppState>) -> AppResult<Json<Vec<ShelfSummary>>> {
    Ok(Json(shelves::list_shelves(&state.db, user.id).await?))
}

#[utoipa::path(post, path = "/api/v1/shelves", request_body = CreateShelfRequest, responses((status = 200, body = ShelfSummary)), tag = "shelves")]
pub async fn create_shelf(CurrentUser(user): CurrentUser, State(state): State<AppState>, Json(input): Json<CreateShelfRequest>) -> AppResult<Json<ShelfSummary>> {
    Ok(Json(shelves::create_shelf(&state.db, user.id, input).await?))
}

#[utoipa::path(patch, path = "/api/v1/shelves/{id}", request_body = UpdateShelfRequest, responses((status = 200, body = ShelfSummary)), tag = "shelves")]
pub async fn update_shelf(CurrentUser(user): CurrentUser, State(state): State<AppState>, Path(id): Path<Uuid>, Json(input): Json<UpdateShelfRequest>) -> AppResult<Json<ShelfSummary>> {
    Ok(Json(shelves::update_shelf(&state.db, user.id, id, input).await?))
}

#[utoipa::path(delete, path = "/api/v1/shelves/{id}", responses((status = 200)), tag = "shelves")]
pub async fn delete_shelf(CurrentUser(user): CurrentUser, State(state): State<AppState>, Path(id): Path<Uuid>) -> AppResult<Json<serde_json::Value>> {
    shelves::delete_shelf(&state.db, user.id, id).await?;
    Ok(Json(serde_json::json!({ "ok": true })))
}

#[utoipa::path(get, path = "/api/v1/shelves/{id}/books", responses((status = 200, body = [BookListItem])), tag = "shelves")]
pub async fn get_shelf_books(CurrentUser(user): CurrentUser, State(state): State<AppState>, Path(id): Path<String>) -> AppResult<Json<Vec<BookListItem>>> {
    Ok(Json(shelves::list_books(&state.db, user.id, shelves::BookFilters { shelf_id: Some(id), ..Default::default() }).await?))
}
