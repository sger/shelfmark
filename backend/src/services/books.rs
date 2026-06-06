use std::path::{Path, PathBuf};

use crate::{
    error::{AppError, AppResult},
    models::{Book, BookFile, ImportJob, Library},
    state::AppState,
};
use sanitize_filename::sanitize;
use sqlx::PgPool;
use tokio::{fs, io::AsyncWriteExt};
use uuid::Uuid;

pub fn detect_format(filename: &str, content_type: Option<&str>) -> AppResult<&'static str> {
    let lower = filename.to_lowercase();
    let by_ext = if lower.ends_with(".pdf") {
        Some("pdf")
    } else if lower.ends_with(".epub") {
        Some("epub")
    } else {
        None
    };

    let by_mime = match content_type.unwrap_or_default() {
        "application/pdf" => Some("pdf"),
        "application/epub+zip" => Some("epub"),
        _ => None,
    };

    by_ext.or(by_mime)
        .ok_or_else(|| AppError::BadRequest("only PDF and EPUB files are supported".to_string()))
}

pub async fn create_job(db: &PgPool, kind: &str, library_id: Option<Uuid>, message: Option<&str>) -> AppResult<ImportJob> {
    let job = sqlx::query_as!(
        ImportJob,
        "INSERT INTO import_jobs (kind, status, library_id, message)
         VALUES ($1, 'queued', $2, $3)
         RETURNING id, kind, status, message, library_id, book_id, created_at, updated_at",
        kind,
        library_id,
        message,
    )
    .fetch_one(db)
    .await?;
    Ok(job)
}

pub async fn finish_job(db: &PgPool, job_id: Uuid, status: &str, message: Option<&str>, book_id: Option<Uuid>) -> AppResult<()> {
    sqlx::query!(
        "UPDATE import_jobs SET status = $1, message = $2, book_id = $3, updated_at = now() WHERE id = $4",
        status,
        message,
        book_id,
        job_id,
    )
    .execute(db)
    .await?;
    Ok(())
}

pub async fn import_file(
    state: &AppState,
    library_id: Option<Uuid>,
    original_name: &str,
    content_type: Option<String>,
    bytes: &[u8],
) -> AppResult<(Book, BookFile)> {
    let format = detect_format(original_name, content_type.as_deref())?;
    let book_id = Uuid::new_v4();
    let safe_name = sanitize(original_name);
    let relative_path = format!("books/{book_id}/{safe_name}");
    let absolute_path = state.config.storage_dir.join(&relative_path);

    if let Some(parent) = absolute_path.parent() {
        fs::create_dir_all(parent).await.map_err(|err| AppError::Internal(anyhow::anyhow!(err)))?;
    }

    let mut file = fs::File::create(&absolute_path).await.map_err(|err| AppError::Internal(anyhow::anyhow!(err)))?;
    file.write_all(bytes).await.map_err(|err| AppError::Internal(anyhow::anyhow!(err)))?;

    let title = title_from_filename(original_name);
    let mut tx = state.db.begin().await?;

    let book = sqlx::query_as!(
        Book,
        "INSERT INTO books (id, library_id, title)
         VALUES ($1, $2, $3)
         RETURNING id, library_id, title, authors, description, publisher, published_date, language, page_count, cover_path, created_at, updated_at",
        book_id,
        library_id,
        title,
    )
    .fetch_one(&mut *tx)
    .await?;

    let book_file = sqlx::query_as!(
        BookFile,
        "INSERT INTO book_files (book_id, original_name, stored_path, format, size_bytes, content_type)
         VALUES ($1, $2, $3, $4, $5, $6)
         RETURNING id, book_id, original_name, stored_path, format, size_bytes, content_type, created_at",
        book.id,
        original_name,
        relative_path,
        format,
        bytes.len() as i64,
        content_type,
    )
    .fetch_one(&mut *tx)
    .await?;

    tx.commit().await?;
    Ok((book, book_file))
}

pub async fn scan_library(state: AppState, library: Library, job_id: Uuid) {
    let result = scan_library_inner(&state, &library).await;
    match result {
        Ok(count) => {
            let _ = finish_job(&state.db, job_id, "completed", Some(&format!("Imported {count} files")), None).await;
        }
        Err(err) => {
            let _ = finish_job(&state.db, job_id, "failed", Some(&err.to_string()), None).await;
        }
    }
}

async fn scan_library_inner(state: &AppState, library: &Library) -> AppResult<usize> {
    let mut count = 0;
    let paths = collect_supported_files(Path::new(&library.path))?;

    for path in paths {
        let bytes = fs::read(&path).await.map_err(|err| AppError::Internal(anyhow::anyhow!(err)))?;
        let name = path.file_name().and_then(|v| v.to_str()).unwrap_or("book").to_string();
        let content_type = mime_guess::from_path(&path).first_raw().map(str::to_string);
        if import_file(state, Some(library.id), &name, content_type, &bytes).await.is_ok() {
            count += 1;
        }
    }

    Ok(count)
}

fn collect_supported_files(root: &Path) -> AppResult<Vec<PathBuf>> {
    let mut out = Vec::new();
    if !root.exists() {
        return Err(AppError::BadRequest(format!("library path does not exist: {}", root.display())));
    }

    for entry in std::fs::read_dir(root).map_err(|err| AppError::Internal(anyhow::anyhow!(err)))? {
        let entry = entry.map_err(|err| AppError::Internal(anyhow::anyhow!(err)))?;
        let path = entry.path();
        if path.is_dir() {
            out.extend(collect_supported_files(&path)?);
        } else if let Some(name) = path.file_name().and_then(|v| v.to_str()) {
            if detect_format(name, None).is_ok() {
                out.push(path);
            }
        }
    }
    Ok(out)
}

pub fn title_from_filename(filename: &str) -> String {
    Path::new(filename)
        .file_stem()
        .and_then(|v| v.to_str())
        .unwrap_or(filename)
        .replace(['_', '-'], " ")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_pdf_and_epub() {
        assert_eq!(detect_format("book.pdf", None).unwrap(), "pdf");
        assert_eq!(detect_format("book.epub", None).unwrap(), "epub");
        assert!(detect_format("book.txt", None).is_err());
    }

    #[test]
    fn normalizes_title_from_filename() {
        assert_eq!(title_from_filename("my-book_name.pdf"), "my book name");
    }
}

