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
            jwt_secret: required_secret("JWT_SECRET")?,
            storage_dir: PathBuf::from(env::var("STORAGE_DIR").unwrap_or_else(|_| "./storage/data".to_string())),
            allowed_origins: env::var("ALLOWED_ORIGINS").unwrap_or_else(|_| "http://localhost:5173".to_string()),
        })
    }
}

fn required(key: &str) -> anyhow::Result<String> {
    env::var(key).map_err(|_| anyhow::anyhow!("{key} is required"))
}

/// Like `required`, but additionally rejects known placeholder/weak values so a
/// misconfigured deployment fails loudly instead of signing tokens with a public secret.
fn required_secret(key: &str) -> anyhow::Result<String> {
    let value = required(key)?;
    const WEAK: [&str; 2] = ["change-me-in-production", "change-me"];
    if value.trim().is_empty() || WEAK.contains(&value.as_str()) {
        anyhow::bail!("{key} must be set to a strong, non-default value");
    }
    Ok(value)
}

