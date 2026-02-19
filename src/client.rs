use reqwest::header::{HeaderMap, HeaderValue, ACCEPT, AUTHORIZATION};
use serde::de::DeserializeOwned;
use serde_json::Value;

use crate::config::Config;
use crate::error::{GiteaError, Result};

/// HTTP client wrapper for the Gitea/Forgejo REST API v1.
#[derive(Debug, Clone)]
pub struct GiteaClient {
    http: reqwest::Client,
    base_api: String,
}

impl GiteaClient {
    /// Create a new client from configuration.
    pub fn new(config: &Config) -> Result<Self> {
        let mut headers = HeaderMap::new();
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str(&format!("token {}", config.token))
                .map_err(|e| GiteaError::Api(format!("Invalid token header: {e}")))?,
        );
        headers.insert(ACCEPT, HeaderValue::from_static("application/json"));

        let http = reqwest::Client::builder()
            .default_headers(headers)
            .build()
            .map_err(|e| GiteaError::Api(format!("Failed to build HTTP client: {e}")))?;

        Ok(Self {
            http,
            base_api: format!("{}/api/v1", config.base_url),
        })
    }

    /// Build the full API URL for a given path.
    fn url(&self, path: &str) -> String {
        format!("{}{}", self.base_api, path)
    }

    /// Send a GET request and deserialize the JSON response.
    pub async fn get<T: DeserializeOwned>(&self, path: &str) -> Result<T> {
        let resp = self.http.get(self.url(path)).send().await?;
        self.handle_response(resp).await
    }

    /// Send a GET request with query parameters.
    pub async fn get_with_query<T: DeserializeOwned>(
        &self,
        path: &str,
        query: &[(&str, &str)],
    ) -> Result<T> {
        let resp = self
            .http
            .get(self.url(path))
            .query(query)
            .send()
            .await?;
        self.handle_response(resp).await
    }

    /// Send a GET request and return the raw text response (for diffs).
    pub async fn get_raw(&self, path: &str) -> Result<String> {
        let url = self.url(path);
        let resp = self
            .http
            .get(&url)
            .header(ACCEPT, "text/plain")
            .send()
            .await?;

        let status = resp.status();
        if status == reqwest::StatusCode::UNAUTHORIZED
            || status == reqwest::StatusCode::FORBIDDEN
        {
            return Err(GiteaError::Auth);
        }
        if status == reqwest::StatusCode::NOT_FOUND {
            return Err(GiteaError::NotFound(url));
        }
        if !status.is_success() {
            let body = resp.text().await.unwrap_or_default();
            return Err(GiteaError::Api(format!("HTTP {status}: {body}")));
        }
        Ok(resp.text().await?)
    }

    /// Send a POST request with a JSON body.
    pub async fn post<T: DeserializeOwned>(&self, path: &str, body: &Value) -> Result<T> {
        let resp = self.http.post(self.url(path)).json(body).send().await?;
        self.handle_response(resp).await
    }

    /// Send a POST request that returns no meaningful body (e.g. merge).
    pub async fn post_no_content(&self, path: &str, body: &Value) -> Result<()> {
        let resp = self.http.post(self.url(path)).json(body).send().await?;
        let status = resp.status();
        if status == reqwest::StatusCode::UNAUTHORIZED
            || status == reqwest::StatusCode::FORBIDDEN
        {
            return Err(GiteaError::Auth);
        }
        if status == reqwest::StatusCode::NOT_FOUND {
            return Err(GiteaError::NotFound(self.url(path)));
        }
        if !status.is_success() {
            let body = resp.text().await.unwrap_or_default();
            return Err(GiteaError::Api(format!("HTTP {status}: {body}")));
        }
        Ok(())
    }

    /// Send a PUT request with a JSON body.
    pub async fn put<T: DeserializeOwned>(&self, path: &str, body: &Value) -> Result<T> {
        let resp = self.http.put(self.url(path)).json(body).send().await?;
        self.handle_response(resp).await
    }

    /// Send a PATCH request with a JSON body.
    pub async fn patch<T: DeserializeOwned>(&self, path: &str, body: &Value) -> Result<T> {
        let resp = self.http.patch(self.url(path)).json(body).send().await?;
        self.handle_response(resp).await
    }

    /// Send a DELETE request.
    pub async fn delete(&self, path: &str) -> Result<()> {
        let resp = self.http.delete(self.url(path)).send().await?;
        let status = resp.status();
        if status == reqwest::StatusCode::UNAUTHORIZED
            || status == reqwest::StatusCode::FORBIDDEN
        {
            return Err(GiteaError::Auth);
        }
        if status == reqwest::StatusCode::NOT_FOUND {
            return Err(GiteaError::NotFound(self.url(path)));
        }
        if !status.is_success() {
            let body = resp.text().await.unwrap_or_default();
            return Err(GiteaError::Api(format!("HTTP {status}: {body}")));
        }
        Ok(())
    }

    /// Send a DELETE request with a JSON body (e.g. file_delete).
    pub async fn delete_with_body(&self, path: &str, body: &Value) -> Result<()> {
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
            return Err(GiteaError::Auth);
        }
        if status == reqwest::StatusCode::NOT_FOUND {
            return Err(GiteaError::NotFound(self.url(path)));
        }
        if !status.is_success() {
            let text = resp.text().await.unwrap_or_default();
            return Err(GiteaError::Api(format!("HTTP {status}: {text}")));
        }
        Ok(())
    }

    /// Handle a response: check status, deserialize JSON.
    async fn handle_response<T: DeserializeOwned>(
        &self,
        resp: reqwest::Response,
    ) -> Result<T> {
        let status = resp.status();
        if status == reqwest::StatusCode::UNAUTHORIZED
            || status == reqwest::StatusCode::FORBIDDEN
        {
            return Err(GiteaError::Auth);
        }
        if status == reqwest::StatusCode::NOT_FOUND {
            let url = resp.url().to_string();
            return Err(GiteaError::NotFound(url));
        }
        if !status.is_success() {
            let body = resp.text().await.unwrap_or_default();
            return Err(GiteaError::Api(format!("HTTP {status}: {body}")));
        }
        let body = resp.json::<T>().await?;
        Ok(body)
    }
}
