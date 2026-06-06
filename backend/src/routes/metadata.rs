use axum::{extract::{Query, State}, routing::get, Json, Router};

use crate::{
    error::AppResult,
    models::{MetadataResult, MetadataSearchQuery},
    routes::auth::CurrentUser,
    services::metadata,
    state::AppState,
};

pub fn router() -> Router<AppState> {
    Router::new().route("/open-library/search", get(search_open_library))
}

#[utoipa::path(get, path = "/api/v1/metadata/open-library/search", params(MetadataSearchQuery), responses((status = 200, body = [MetadataResult])), tag = "metadata")]
pub async fn search_open_library(_user: CurrentUser, State(state): State<AppState>, Query(query): Query<MetadataSearchQuery>) -> AppResult<Json<Vec<MetadataResult>>> {
    Ok(Json(metadata::search_open_library(&state, &query.q).await?))
}

