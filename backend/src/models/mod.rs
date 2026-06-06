use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::{IntoParams, ToSchema};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct Claims {
    pub sub: Uuid,
    pub email: String,
    pub role: String,
    pub exp: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, ToSchema)]
pub struct User {
    pub id: Uuid,
    pub email: String,
    pub display_name: String,
    pub role: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, ToSchema)]
pub struct Library {
    pub id: Uuid,
    pub name: String,
    pub path: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, ToSchema)]
pub struct Book {
    pub id: Uuid,
    pub library_id: Option<Uuid>,
    pub title: String,
    pub authors: Vec<String>,
    pub description: Option<String>,
    pub publisher: Option<String>,
    pub published_date: Option<String>,
    pub language: Option<String>,
    pub page_count: Option<i32>,
    pub cover_path: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, ToSchema)]
pub struct BookListItem {
    pub id: Uuid,
    pub library_id: Option<Uuid>,
    pub title: String,
    pub authors: Vec<String>,
    pub description: Option<String>,
    pub publisher: Option<String>,
    pub published_date: Option<String>,
    pub language: Option<String>,
    pub page_count: Option<i32>,
    pub cover_path: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub favorite: bool,
    pub reading_status: String,
    pub tags: Vec<String>,
    pub progress_percent: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, ToSchema)]
pub struct BookFile {
    pub id: Uuid,
    pub book_id: Uuid,
    pub original_name: String,
    pub stored_path: String,
    pub format: String,
    pub size_bytes: i64,
    pub content_type: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, ToSchema)]
pub struct ImportJob {
    pub id: Uuid,
    pub kind: String,
    pub status: String,
    pub message: Option<String>,
    pub library_id: Option<Uuid>,
    pub book_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, ToSchema)]
pub struct ReadingProgress {
    pub id: Uuid,
    pub user_id: Uuid,
    pub book_id: Uuid,
    pub position: String,
    pub progress_percent: f64,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, ToSchema)]
pub struct BookUserState {
    pub user_id: Uuid,
    pub book_id: Uuid,
    pub favorite: bool,
    pub reading_status: String,
    pub tags: Vec<String>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, ToSchema)]
pub struct SmartShelf {
    pub id: Uuid,
    pub user_id: Uuid,
    pub name: String,
    pub match_mode: String,
    pub rules: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ShelfSummary {
    pub id: String,
    pub name: String,
    pub kind: String,
    pub count: i64,
    pub group: Option<String>,
    pub rules: Option<Vec<ShelfRule>>,
    pub match_mode: Option<String>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct RegisterRequest {
    pub email: String,
    pub password: String,
    pub display_name: String,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct AuthResponse {
    pub token: String,
    pub user: User,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateLibraryRequest {
    pub name: String,
    pub path: String,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateLibraryRequest {
    pub name: Option<String>,
    pub path: Option<String>,
}

#[derive(Debug, Deserialize, IntoParams, ToSchema)]
pub struct BookQuery {
    pub q: Option<String>,
    pub library_id: Option<Uuid>,
    pub status: Option<String>,
    pub favorite: Option<bool>,
    pub tag: Option<String>,
    pub author: Option<String>,
    pub language: Option<String>,
    pub year: Option<String>,
    pub shelf_id: Option<String>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateBookRequest {
    pub title: Option<String>,
    pub authors: Option<Vec<String>>,
    pub description: Option<String>,
    pub publisher: Option<String>,
    pub published_date: Option<String>,
    pub language: Option<String>,
    pub page_count: Option<i32>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct ProgressRequest {
    pub position: String,
    pub progress_percent: f64,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateBookStateRequest {
    pub favorite: Option<bool>,
    pub reading_status: Option<String>,
    pub tags: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ShelfRule {
    pub field: String,
    pub operator: String,
    pub value: serde_json::Value,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateShelfRequest {
    pub name: String,
    pub match_mode: String,
    pub rules: Vec<ShelfRule>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateShelfRequest {
    pub name: Option<String>,
    pub match_mode: Option<String>,
    pub rules: Option<Vec<ShelfRule>>,
}

#[derive(Debug, Deserialize, IntoParams, ToSchema)]
pub struct MetadataSearchQuery {
    pub q: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct MetadataResult {
    pub title: String,
    pub authors: Vec<String>,
    pub description: Option<String>,
    pub publisher: Option<String>,
    pub published_date: Option<String>,
    pub language: Option<String>,
    pub page_count: Option<i32>,
    pub cover_url: Option<String>,
}
