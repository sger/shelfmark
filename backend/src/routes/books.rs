use axum::{
    body::Body,
    extract::{Multipart, Path, Query, State},
    http::{header, StatusCode},
    response::Response,
    routing::{get, post},
    Json, Router,
};
use tokio::fs;
use uuid::Uuid;

use crate::{
    error::{AppError, AppResult},
    models::{Book, BookFile, BookListItem, BookQuery, BookUserState, ProgressRequest, ReadingProgress, UpdateBookRequest, UpdateBookStateRequest},
    routes::auth::CurrentUser,
    services::{books, shelves},
    state::AppState,
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(list_books))
        .route("/upload", post(upload_book))
        .route("/{id}", get(get_book).patch(update_book).delete(delete_book))
        .route("/{id}/file", get(get_book_file))
        .route("/{id}/cover", get(get_book_cover))
        .route("/{id}/progress", get(get_progress).patch(update_progress))
        .route("/{id}/state", get(get_book_state).patch(update_book_state))
}

#[utoipa::path(get, path = "/api/v1/books", params(BookQuery), responses((status = 200, body = [BookListItem])), tag = "books")]
pub async fn list_books(CurrentUser(user): CurrentUser, State(state): State<AppState>, Query(query): Query<BookQuery>) -> AppResult<Json<Vec<BookListItem>>> {
    let books = shelves::list_books(&state.db, user.id, shelves::BookFilters {
        q: query.q,
        library_id: query.library_id,
        status: query.status,
        favorite: query.favorite,
        tag: query.tag,
        author: query.author,
        language: query.language,
        year: query.year,
        shelf_id: query.shelf_id,
    }).await?;
    Ok(Json(books))
}

#[utoipa::path(post, path = "/api/v1/books/upload", responses((status = 200, body = Book)), tag = "books")]
pub async fn upload_book(_user: CurrentUser, State(state): State<AppState>, mut multipart: Multipart) -> AppResult<Json<Book>> {
    let mut library_id = None;
    let mut uploaded = None;

    while let Some(field) = multipart.next_field().await.map_err(|err| AppError::BadRequest(err.to_string()))? {
        let name = field.name().unwrap_or_default().to_string();
        if name == "library_id" {
            let value = field.text().await.map_err(|err| AppError::BadRequest(err.to_string()))?;
            if !value.trim().is_empty() {
                library_id = Some(value.parse::<Uuid>().map_err(|_| AppError::BadRequest("invalid library id".to_string()))?);
            }
        } else if name == "file" {
            let file_name = field.file_name().unwrap_or("book").to_string();
            let content_type = field.content_type().map(str::to_string);
            let bytes = field.bytes().await.map_err(|err| AppError::BadRequest(err.to_string()))?;
            uploaded = Some((file_name, content_type, bytes));
        }
    }

    let (file_name, content_type, bytes) = uploaded.ok_or_else(|| AppError::BadRequest("file field is required".to_string()))?;
    let job = books::create_job(&state.db, "upload", library_id, Some("Upload received")).await?;
    let result = books::import_file(&state, library_id, &file_name, content_type, &bytes).await;
    match result {
        Ok((book, _file)) => {
            books::finish_job(&state.db, job.id, "completed", Some("Upload imported"), Some(book.id)).await?;
            Ok(Json(book))
        }
        Err(err) => {
            books::finish_job(&state.db, job.id, "failed", Some(&err.to_string()), None).await?;
            Err(err)
        }
    }
}

#[utoipa::path(get, path = "/api/v1/books/{id}", responses((status = 200, body = Book)), tag = "books")]
pub async fn get_book(_user: CurrentUser, State(state): State<AppState>, Path(id): Path<Uuid>) -> AppResult<Json<Book>> {
    Ok(Json(find_book(&state, id).await?))
}

#[utoipa::path(patch, path = "/api/v1/books/{id}", request_body = UpdateBookRequest, responses((status = 200, body = Book)), tag = "books")]
pub async fn update_book(_user: CurrentUser, State(state): State<AppState>, Path(id): Path<Uuid>, Json(input): Json<UpdateBookRequest>) -> AppResult<Json<Book>> {
    let existing = find_book(&state, id).await?;
    let book = sqlx::query_as!(
        Book,
        "UPDATE books
         SET title = $1, authors = $2, description = $3, publisher = $4, published_date = $5,
             language = $6, page_count = $7, updated_at = now()
         WHERE id = $8
         RETURNING id, library_id, title, authors, description, publisher, published_date, language, page_count, cover_path, created_at, updated_at",
        input.title.unwrap_or(existing.title),
        &input.authors.unwrap_or(existing.authors),
        input.description.or(existing.description),
        input.publisher.or(existing.publisher),
        input.published_date.or(existing.published_date),
        input.language.or(existing.language),
        input.page_count.or(existing.page_count),
        id,
    )
    .fetch_one(&state.db)
    .await?;
    Ok(Json(book))
}

#[utoipa::path(delete, path = "/api/v1/books/{id}", responses((status = 200)), tag = "books")]
pub async fn delete_book(_user: CurrentUser, State(state): State<AppState>, Path(id): Path<Uuid>) -> AppResult<Json<serde_json::Value>> {
    sqlx::query!("DELETE FROM books WHERE id = $1", id).execute(&state.db).await?;
    Ok(Json(serde_json::json!({ "ok": true })))
}

#[utoipa::path(get, path = "/api/v1/books/{id}/file", responses((status = 200)), tag = "books")]
pub async fn get_book_file(_user: CurrentUser, State(state): State<AppState>, Path(id): Path<Uuid>) -> AppResult<Response> {
    let file = find_book_file(&state, id).await?;
    let path = state.config.storage_dir.join(&file.stored_path);
    let bytes = fs::read(path).await.map_err(|err| AppError::Internal(anyhow::anyhow!(err)))?;
    let mime = file.content_type.unwrap_or_else(|| if file.format == "pdf" { "application/pdf" } else { "application/epub+zip" }.to_string());

    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, mime)
        .header(header::CONTENT_DISPOSITION, format!("inline; filename=\"{}\"", file.original_name))
        .body(Body::from(bytes))
        .map_err(|err| AppError::Internal(anyhow::anyhow!(err)))
}

#[utoipa::path(get, path = "/api/v1/books/{id}/cover", responses((status = 200)), tag = "books")]
pub async fn get_book_cover(_user: CurrentUser, State(state): State<AppState>, Path(id): Path<Uuid>) -> AppResult<Response> {
    let book = find_book(&state, id).await?;
    let cover_path = book.cover_path.ok_or(AppError::NotFound)?;
    let bytes = fs::read(state.config.storage_dir.join(cover_path)).await.map_err(|err| AppError::Internal(anyhow::anyhow!(err)))?;
    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "image/jpeg")
        .body(Body::from(bytes))
        .map_err(|err| AppError::Internal(anyhow::anyhow!(err)))
}

#[utoipa::path(get, path = "/api/v1/books/{id}/progress", responses((status = 200, body = ReadingProgress)), tag = "books")]
pub async fn get_progress(CurrentUser(user): CurrentUser, State(state): State<AppState>, Path(id): Path<Uuid>) -> AppResult<Json<Option<ReadingProgress>>> {
    let progress = sqlx::query_as!(
        ReadingProgress,
        "SELECT id, user_id, book_id, position, progress_percent, updated_at
         FROM reading_progress WHERE user_id = $1 AND book_id = $2",
        user.id,
        id,
    )
    .fetch_optional(&state.db)
    .await?;
    Ok(Json(progress))
}

#[utoipa::path(patch, path = "/api/v1/books/{id}/progress", request_body = ProgressRequest, responses((status = 200, body = ReadingProgress)), tag = "books")]
pub async fn update_progress(CurrentUser(user): CurrentUser, State(state): State<AppState>, Path(id): Path<Uuid>, Json(input): Json<ProgressRequest>) -> AppResult<Json<ReadingProgress>> {
    if !(0.0..=100.0).contains(&input.progress_percent) {
        return Err(AppError::BadRequest("progress_percent must be between 0 and 100".to_string()));
    }
    let progress = sqlx::query_as!(
        ReadingProgress,
        "INSERT INTO reading_progress (user_id, book_id, position, progress_percent)
         VALUES ($1, $2, $3, $4)
         ON CONFLICT (user_id, book_id)
         DO UPDATE SET position = excluded.position, progress_percent = excluded.progress_percent, updated_at = now()
         RETURNING id, user_id, book_id, position, progress_percent, updated_at",
        user.id,
        id,
        input.position,
        input.progress_percent,
    )
    .fetch_one(&state.db)
    .await?;
    shelves::derive_reading_status(&state.db, user.id, id, input.progress_percent).await?;
    Ok(Json(progress))
}


#[utoipa::path(get, path = "/api/v1/books/{id}/state", responses((status = 200, body = BookUserState)), tag = "books")]
pub async fn get_book_state(CurrentUser(user): CurrentUser, State(state): State<AppState>, Path(id): Path<Uuid>) -> AppResult<Json<BookUserState>> {
    let _ = find_book(&state, id).await?;
    Ok(Json(shelves::get_book_state(&state.db, user.id, id).await?))
}

#[utoipa::path(patch, path = "/api/v1/books/{id}/state", request_body = UpdateBookStateRequest, responses((status = 200, body = BookUserState)), tag = "books")]
pub async fn update_book_state(CurrentUser(user): CurrentUser, State(state): State<AppState>, Path(id): Path<Uuid>, Json(input): Json<UpdateBookStateRequest>) -> AppResult<Json<BookUserState>> {
    let _ = find_book(&state, id).await?;
    Ok(Json(shelves::update_book_state(&state.db, user.id, id, input).await?))
}

async fn find_book(state: &AppState, id: Uuid) -> AppResult<Book> {
    sqlx::query_as!(
        Book,
        "SELECT id, library_id, title, authors, description, publisher, published_date, language, page_count, cover_path, created_at, updated_at
         FROM books WHERE id = $1",
        id,
    )
    .fetch_optional(&state.db)
    .await?
    .ok_or(AppError::NotFound)
}

async fn find_book_file(state: &AppState, book_id: Uuid) -> AppResult<BookFile> {
    sqlx::query_as!(
        BookFile,
        "SELECT id, book_id, original_name, stored_path, format, size_bytes, content_type, created_at
         FROM book_files WHERE book_id = $1 ORDER BY created_at DESC LIMIT 1",
        book_id,
    )
    .fetch_optional(&state.db)
    .await?
    .ok_or(AppError::NotFound)
}
