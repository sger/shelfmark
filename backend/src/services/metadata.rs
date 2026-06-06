use crate::{error::{AppError, AppResult}, models::MetadataResult, state::AppState};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct OpenLibraryResponse {
    docs: Vec<OpenLibraryDoc>,
}

#[derive(Debug, Deserialize)]
struct OpenLibraryDoc {
    title: Option<String>,
    author_name: Option<Vec<String>>,
    first_publish_year: Option<i32>,
    publisher: Option<Vec<String>>,
    language: Option<Vec<String>>,
    number_of_pages_median: Option<i32>,
    cover_i: Option<i64>,
}

pub async fn search_open_library(state: &AppState, query: &str) -> AppResult<Vec<MetadataResult>> {
    if query.trim().is_empty() {
        return Err(AppError::BadRequest("query is required".to_string()));
    }

    let response = state
        .http
        .get("https://openlibrary.org/search.json")
        .query(&[("q", query), ("limit", "10")])
        .send()
        .await
        .map_err(|err| AppError::Internal(anyhow::anyhow!(err)))?
        .error_for_status()
        .map_err(|err| AppError::Internal(anyhow::anyhow!(err)))?
        .json::<OpenLibraryResponse>()
        .await
        .map_err(|err| AppError::Internal(anyhow::anyhow!(err)))?;

    Ok(response.docs.into_iter().filter_map(map_doc).collect())
}

fn map_doc(doc: OpenLibraryDoc) -> Option<MetadataResult> {
    let title = doc.title?;
    Some(MetadataResult {
        title,
        authors: doc.author_name.unwrap_or_default(),
        description: None,
        publisher: doc.publisher.and_then(|v| v.into_iter().next()),
        published_date: doc.first_publish_year.map(|year| year.to_string()),
        language: doc.language.and_then(|v| v.into_iter().next()),
        page_count: doc.number_of_pages_median,
        cover_url: doc.cover_i.map(|id| format!("https://covers.openlibrary.org/b/id/{id}-L.jpg")),
    })
}

