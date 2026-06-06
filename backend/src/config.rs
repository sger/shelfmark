use std::{env, net::SocketAddr, path::PathBuf};

#[derive(Clone, Debug)]
pub struct Config {
    pub bind_addr: SocketAddr,
    pub database_url: String,
    pub jwt_secret: String,
    pub storage_dir: PathBuf,
    pub allowed_origins: String,
}

impl Config {
    pub fn from_env() -> anyhow::Result<Self> {
        let bind_addr = env::var("BIND_ADDR")
            .unwrap_or_else(|_| "0.0.0.0:6060".to_string())
            .parse()?;

        Ok(Self {
            bind_addr,
            database_url: required("DATABASE_URL")?,
            jwt_secret: env::var("JWT_SECRET").unwrap_or_else(|_| "change-me-in-production".to_string()),
            storage_dir: PathBuf::from(env::var("STORAGE_DIR").unwrap_or_else(|_| "./storage/data".to_string())),
            allowed_origins: env::var("ALLOWED_ORIGINS").unwrap_or_else(|_| "http://localhost:5173".to_string()),
        })
    }
}

fn required(key: &str) -> anyhow::Result<String> {
    env::var(key).map_err(|_| anyhow::anyhow!("{key} is required"))
}

