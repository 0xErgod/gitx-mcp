use crate::error::{GiteaError, Result};

/// Server configuration loaded from environment variables.
#[derive(Debug, Clone)]
pub struct Config {
    /// Base URL of the Gitea/Forgejo instance (e.g. `https://git.example.com`)
    pub base_url: String,
    /// API token for authentication
    pub token: String,
}

impl Config {
    /// Load configuration from environment variables.
    ///
    /// Checks `GITEA_URL` / `GITEA_TOKEN` first, then falls back to
    /// `FORGEJO_REMOTE_URL` / `FORGEJO_AUTH_TOKEN` for backward compatibility.
    pub fn from_env() -> Result<Self> {
        let _ = dotenvy::dotenv(); // ignore missing .env

        let base_url = std::env::var("GITEA_URL")
            .or_else(|_| std::env::var("FORGEJO_REMOTE_URL"))
            .map_err(|_| {
                GiteaError::MissingParam(
                    "GITEA_URL (or FORGEJO_REMOTE_URL) environment variable is required"
                        .to_string(),
                )
            })?;

        let token = std::env::var("GITEA_TOKEN")
            .or_else(|_| std::env::var("FORGEJO_AUTH_TOKEN"))
            .map_err(|_| {
                GiteaError::MissingParam(
                    "GITEA_TOKEN (or FORGEJO_AUTH_TOKEN) environment variable is required"
                        .to_string(),
                )
            })?;

        // Strip trailing slash from base URL
        let base_url = base_url.trim_end_matches('/').to_string();

        Ok(Config { base_url, token })
    }
}
