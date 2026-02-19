use rmcp::model::{CallToolResult, Content};
use schemars::JsonSchema;
use serde::Deserialize;

use crate::client::GiteaClient;
use crate::error::Result;
use crate::repo_resolver::RepoInfo;
use crate::server::resolve_owner_repo;

#[derive(Debug, Deserialize, JsonSchema)]
pub struct RepoGetParams {
    /// Repository owner. Optional if `directory` is provided.
    pub owner: Option<String>,
    /// Repository name. Optional if `directory` is provided.
    pub repo: Option<String>,
    /// Local directory to auto-detect owner/repo from .git/config.
    pub directory: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct RepoSearchParams {
    /// Search keyword.
    pub q: String,
    /// Page number (1-based). Defaults to 1.
    pub page: Option<i64>,
    /// Items per page (max 50). Defaults to 20.
    pub limit: Option<i64>,
}

pub async fn repo_get(client: &GiteaClient, params: RepoGetParams, default_repo: Option<&RepoInfo>) -> Result<CallToolResult> {
    let (owner, repo) = resolve_owner_repo(&params.owner, &params.repo, &params.directory, default_repo)?;
    let repo_info: serde_json::Value = client
        .get(&format!("/repos/{owner}/{repo}"))
        .await?;

    let mut parts = Vec::new();

    let full_name = repo_info
        .get("full_name")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown");
    parts.push(format!("## {full_name}"));

    if let Some(desc) = repo_info.get("description").and_then(|v| v.as_str()) {
        if !desc.is_empty() {
            parts.push(format!("**Description:** {desc}"));
        }
    }

    if let Some(branch) = repo_info
        .get("default_branch")
        .and_then(|v| v.as_str())
    {
        parts.push(format!("**Default branch:** {branch}"));
    }

    let stars = repo_info
        .get("stars_count")
        .and_then(|v| v.as_i64())
        .unwrap_or(0);
    let forks = repo_info
        .get("forks_count")
        .and_then(|v| v.as_i64())
        .unwrap_or(0);
    parts.push(format!("**Stars:** {stars} | **Forks:** {forks}"));

    let private = repo_info
        .get("private")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    parts.push(format!(
        "**Visibility:** {}",
        if private { "private" } else { "public" }
    ));

    if let Some(lang) = repo_info.get("language").and_then(|v| v.as_str()) {
        if !lang.is_empty() {
            parts.push(format!("**Language:** {lang}"));
        }
    }

    Ok(CallToolResult::success(vec![Content::text(
        parts.join("\n"),
    )]))
}

pub async fn repo_search(
    client: &GiteaClient,
    params: RepoSearchParams,
) -> Result<CallToolResult> {
    let mut query: Vec<(&str, String)> = Vec::new();
    query.push(("q", params.q.clone()));
    query.push(("page", params.page.unwrap_or(1).to_string()));
    query.push(("limit", params.limit.unwrap_or(20).min(50).to_string()));

    let query_refs: Vec<(&str, &str)> = query.iter().map(|(k, v)| (*k, v.as_str())).collect();
    let result: serde_json::Value = client
        .get_with_query("/repos/search", &query_refs)
        .await?;

    let repos = result
        .get("data")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();

    if repos.is_empty() {
        return Ok(CallToolResult::success(vec![Content::text(
            "No repositories found.",
        )]));
    }

    let formatted: Vec<String> = repos
        .iter()
        .map(|r| {
            let full_name = r
                .get("full_name")
                .and_then(|v| v.as_str())
                .unwrap_or("?");
            let desc = r
                .get("description")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let stars = r
                .get("stars_count")
                .and_then(|v| v.as_i64())
                .unwrap_or(0);
            if desc.is_empty() {
                format!("- {full_name} ({stars} stars)")
            } else {
                format!("- {full_name} ({stars} stars) - {desc}")
            }
        })
        .collect();

    Ok(CallToolResult::success(vec![Content::text(
        formatted.join("\n"),
    )]))
}
