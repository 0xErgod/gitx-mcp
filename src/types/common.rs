use schemars::JsonSchema;
use serde::Deserialize;

/// Parameters identifying a repository, shared by most tools.
/// Either provide `owner` + `repo`, or `directory` to auto-detect from `.git/config`.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct RepoParams {
    /// Repository owner (user or organization). Optional if `directory` is provided.
    pub owner: Option<String>,
    /// Repository name. Optional if `directory` is provided.
    pub repo: Option<String>,
    /// Local directory path containing a `.git/config` to auto-detect owner/repo.
    pub directory: Option<String>,
}

/// Pagination parameters.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct PaginationParams {
    /// Page number (1-based). Defaults to 1.
    pub page: Option<i64>,
    /// Number of items per page. Defaults to 20, max 50.
    pub limit: Option<i64>,
}

impl PaginationParams {
    pub fn to_query(&self) -> Vec<(&str, String)> {
        let mut q = Vec::new();
        if let Some(page) = self.page {
            q.push(("page", page.to_string()));
        }
        let limit = self.limit.unwrap_or(20).min(50);
        q.push(("limit", limit.to_string()));
        q
    }
}
