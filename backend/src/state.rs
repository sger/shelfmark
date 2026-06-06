use crate::config::Config;
use reqwest::Client;
use sqlx::PgPool;

#[derive(Clone)]
pub struct AppState {
    pub config: Config,
    pub db: PgPool,
    pub http: Client,
}

