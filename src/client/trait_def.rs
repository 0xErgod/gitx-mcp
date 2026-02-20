use std::fmt::Debug;

use async_trait::async_trait;
use serde_json::Value;

use crate::error::Result;
use crate::platform::Platform;

/// Trait abstracting HTTP client operations for Git platform APIs.
///
/// All JSON methods return `serde_json::Value` to maintain object safety
/// (generic `DeserializeOwned` methods are not object-safe).
#[async_trait]
pub trait GitClient: Send + Sync + Debug {
    /// Which platform this client connects to.
    fn platform(&self) -> Platform;

    /// GET request, returning parsed JSON.
    async fn get_json(&self, path: &str) -> Result<Value>;

    /// GET request with query parameters, returning parsed JSON.
    async fn get_json_with_query(&self, path: &str, query: &[(&str, &str)]) -> Result<Value>;

    /// GET request returning raw text (e.g. diffs).
    async fn get_raw(&self, path: &str) -> Result<String>;

    /// POST request with JSON body, returning parsed JSON.
    async fn post_json(&self, path: &str, body: &Value) -> Result<Value>;

    /// POST request that returns no meaningful body (e.g. merge).
    async fn post_no_content(&self, path: &str, body: &Value) -> Result<()>;

    /// PUT request with JSON body, returning parsed JSON.
    async fn put_json(&self, path: &str, body: &Value) -> Result<Value>;

    /// PATCH request with JSON body, returning parsed JSON.
    async fn patch_json(&self, path: &str, body: &Value) -> Result<Value>;

    /// DELETE request.
    async fn delete(&self, path: &str) -> Result<()>;

    /// DELETE request with a JSON body (e.g. file_delete).
    async fn delete_with_body(&self, path: &str, body: &Value) -> Result<()>;
}
