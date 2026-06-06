mod config;
mod db;
mod error;
mod models;
mod routes;
mod services;
mod state;

use axum::{
    http::{HeaderValue, Method},
    routing::get,
    Json, Router,
};
use config::Config;
use models::*;
use reqwest::Client;
use routes::{auth, books, jobs, libraries, metadata, shelves};
use state::AppState;
use tower_http::{cors::CorsLayer, services::{ServeDir, ServeFile}, trace::TraceLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

#[derive(OpenApi)]
#[openapi(
    paths(
        auth::register,
        auth::login,
        auth::logout,
        auth::me,
        libraries::list_libraries,
        libraries::create_library,
        libraries::update_library,
        libraries::delete_library,
        libraries::scan_library,
        books::list_books,
        books::upload_book,
        books::get_book,
        books::update_book,
        books::delete_book,
        books::get_book_file,
        books::get_book_cover,
        books::get_progress,
        books::update_progress,
        books::get_book_state,
        books::update_book_state,
        shelves::list_shelves,
        shelves::create_shelf,
        shelves::update_shelf,
        shelves::delete_shelf,
        shelves::get_shelf_books,
        metadata::search_open_library,
        jobs::list_jobs,
        jobs::get_job
    ),
    components(schemas(
        AuthResponse,
        Book,
        BookFile,
        BookListItem,
        BookQuery,
        BookUserState,
        CreateLibraryRequest,
        CreateShelfRequest,
        ImportJob,
        Library,
        LoginRequest,
        MetadataResult,
        MetadataSearchQuery,
        ProgressRequest,
        ReadingProgress,
        RegisterRequest,
        ShelfRule,
        ShelfSummary,
        SmartShelf,
        UpdateBookRequest,
        UpdateBookStateRequest,
        UpdateLibraryRequest,
        UpdateShelfRequest,
        User
    )),
    tags(
        (name = "auth", description = "Local account authentication"),
        (name = "libraries", description = "Library folder management"),
        (name = "books", description = "Book import, metadata, files, and progress"),
        (name = "shelves", description = "Smart shelves and book organization"),
        (name = "metadata", description = "External metadata lookup"),
        (name = "jobs", description = "Import and scan jobs")
    )
)]
struct ApiDoc;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info,tower_http=debug".into()))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let config = Config::from_env()?;
    tokio::fs::create_dir_all(&config.storage_dir).await?;
    let db = db::connect(&config.database_url).await?;
    let state = AppState {
        config: config.clone(),
        db,
        http: Client::new(),
    };

    let cors = CorsLayer::new()
        .allow_methods([Method::GET, Method::POST, Method::PATCH, Method::DELETE])
        .allow_headers(tower_http::cors::Any)
        .allow_origin(config.allowed_origins.parse::<HeaderValue>()?);

    let api = Router::new()
        .nest("/auth", auth::router())
        .nest("/libraries", libraries::router())
        .nest("/books", books::router())
        .nest("/shelves", shelves::router())
        .nest("/metadata", metadata::router())
        .nest("/jobs", jobs::router());
    let static_dir = std::env::var("STATIC_DIR").unwrap_or_else(|_| "/app/static".to_string());

    let app = Router::new()
        .route("/health", get(health))
        .nest("/api/v1", api)
        .merge(SwaggerUi::new("/api/docs").url("/api/openapi.json", ApiDoc::openapi()))
        .fallback_service(ServeDir::new(&static_dir).not_found_service(ServeFile::new(format!("{static_dir}/index.html"))))
        .layer(cors)
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    tracing::info!("listening on {}", config.bind_addr);
    let listener = tokio::net::TcpListener::bind(config.bind_addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}

async fn health() -> Json<serde_json::Value> {
    Json(serde_json::json!({ "status": "ok" }))
}
