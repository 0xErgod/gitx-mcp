use async_trait::async_trait;
use reqwest::header::{HeaderMap, HeaderValue, ACCEPT, AUTHORIZATION};
use serde_json::Value;

use crate::config::Config;
use crate::error::{GitxError, Result};
use crate::platform::Platform;

use super::GitClient;

/// HTTP client wrapper for the GitHub REST API.
#[derive(Debug, Clone)]
pub struct GitHubClient {
    http: reqwest::Client,
    base_api: String,
}

impl GitHubClient {
    /// Create a new GitHub client from configuration.
    ///
    /// For github.com the base API is `https://api.github.com`.
    /// For GitHub Enterprise, it is `{base_url}/api/v3`.
    pub fn new(config: &Config) -> Result<Self> {
        let mut headers = HeaderMap::new();
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {}", config.token))
                .map_err(|e| GitxError::Api(format!("Invalid token header: {e}")))?,
        );
        headers.insert(
            ACCEPT,
            HeaderValue::from_static("application/vnd.github+json"),
        );
        headers.insert(
            "X-GitHub-Api-Version",
            HeaderValue::from_static("2022-11-28"),
        );

        let http = reqwest::Client::builder()
            .default_headers(headers)
            .user_agent("gitx-mcp")
            .build()
            .map_err(|e| GitxError::Api(format!("Failed to build HTTP client: {e}")))?;

        // github.com → https://api.github.com
        // Enterprise → {base_url}/api/v3
        let base_api = if config.base_url == "https://github.com" {
            "https://api.github.com".to_string()
        } else {
            format!("{}/api/v3", config.base_url)
        };

        Ok(Self { http, base_api })
    }

    /// Build the full API URL for a given path.
    fn url(&self, path: &str) -> String {
        format!("{}{}", self.base_api, path)
    }

    /// Handle a response: check status, deserialize JSON to Value.
    async fn handle_response(&self, resp: reqwest::Response) -> Result<Value> {
        let status = resp.status();
        if status == reqwest::StatusCode::UNAUTHORIZED
            || status == reqwest::StatusCode::FORBIDDEN
        {
            return Err(GitxError::Auth);
        }
        if status == reqwest::StatusCode::NOT_FOUND {
            let url = resp.url().to_string();
            return Err(GitxError::NotFound(url));
        }
        if !status.is_success() {
            let body = resp.text().await.unwrap_or_default();
            return Err(GitxError::Api(format!("HTTP {status}: {body}")));
        }
        let body = resp.json::<Value>().await?;
        Ok(body)
    }
}

#[async_trait]
impl GitClient for GitHubClient {
    fn platform(&self) -> Platform {
        Platform::GitHub
    }

    async fn get_json(&self, path: &str) -> Result<Value> {
        let resp = self.http.get(self.url(path)).send().await?;
        self.handle_response(resp).await
    }

    async fn get_json_with_query(&self, path: &str, query: &[(&str, &str)]) -> Result<Value> {
        let resp = self
            .http
            .get(self.url(path))
            .query(query)
            .send()
            .await?;
        self.handle_response(resp).await
    }

    async fn get_raw(&self, path: &str) -> Result<String> {
        let url = self.url(path);
        let resp = self
            .http
            .get(&url)
            .header(ACCEPT, "application/vnd.github.diff")
            .send()
            .await?;

        let status = resp.status();
        if status == reqwest::StatusCode::UNAUTHORIZED
            || status == reqwest::StatusCode::FORBIDDEN
        {
            return Err(GitxError::Auth);
        }
        if status == reqwest::StatusCode::NOT_FOUND {
            return Err(GitxError::NotFound(url));
        }
        if !status.is_success() {
            let body = resp.text().await.unwrap_or_default();
            return Err(GitxError::Api(format!("HTTP {status}: {body}")));
        }
        Ok(resp.text().await?)
    }

    async fn post_json(&self, path: &str, body: &Value) -> Result<Value> {
        let resp = self.http.post(self.url(path)).json(body).send().await?;
        self.handle_response(resp).await
    }

    async fn post_no_content(&self, path: &str, body: &Value) -> Result<()> {
        let resp = self.http.put(self.url(path)).json(body).send().await?;
        let status = resp.status();
        if status == reqwest::StatusCode::UNAUTHORIZED
            || status == reqwest::StatusCode::FORBIDDEN
        {
            return Err(GitxError::Auth);
        }
        if status == reqwest::StatusCode::NOT_FOUND {
            return Err(GitxError::NotFound(self.url(path)));
        }
        if !status.is_success() {
            let body = resp.text().await.unwrap_or_default();
            return Err(GitxError::Api(format!("HTTP {status}: {body}")));
        }
        Ok(())
    }

    async fn put_json(&self, path: &str, body: &Value) -> Result<Value> {
        let resp = self.http.put(self.url(path)).json(body).send().await?;
        self.handle_response(resp).await
    }

    async fn patch_json(&self, path: &str, body: &Value) -> Result<Value> {
        let resp = self.http.patch(self.url(path)).json(body).send().await?;
        self.handle_response(resp).await
    }

    async fn delete(&self, path: &str) -> Result<()> {
        let resp = self.http.delete(self.url(path)).send().await?;
        let status = resp.status();
        if status == reqwest::StatusCode::UNAUTHORIZED
            || status == reqwest::StatusCode::FORBIDDEN
        {
            return Err(GitxError::Auth);
        }
        if status == reqwest::StatusCode::NOT_FOUND {
            return Err(GitxError::NotFound(self.url(path)));
        }
        if !status.is_success() {
            let body = resp.text().await.unwrap_or_default();
            return Err(GitxError::Api(format!("HTTP {status}: {body}")));
        }
        Ok(())
    }

    async fn delete_with_body(&self, path: &str, body: &Value) -> Result<()> {
        let resp = self
            .http
            .delete(self.url(path))
            .json(body)
            .send()
            .await?;
        let status = resp.status();
        if status == reqwest::StatusCode::UNAUTHORIZED
            || status == reqwest::StatusCode::FORBIDDEN
        {
            return Err(GitxError::Auth);
        }
        if status == reqwest::StatusCode::NOT_FOUND {
            return Err(GitxError::NotFound(self.url(path)));
        }
        if !status.is_success() {
            let text = resp.text().await.unwrap_or_default();
            return Err(GitxError::Api(format!("HTTP {status}: {text}")));
        }
        Ok(())
    }
}
