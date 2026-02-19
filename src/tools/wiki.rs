use rmcp::model::{CallToolResult, Content};
use schemars::JsonSchema;
use serde::Deserialize;

use crate::client::GiteaClient;
use crate::error::Result;
use crate::repo_resolver::RepoInfo;
use crate::server::resolve_owner_repo;

#[derive(Debug, Deserialize, JsonSchema)]
pub struct WikiListParams {
    /// Repository owner. Optional if `directory` is provided.
    pub owner: Option<String>,
    /// Repository name. Optional if `directory` is provided.
    pub repo: Option<String>,
    /// Local directory to auto-detect owner/repo from .git/config.
    pub directory: Option<String>,
    /// Page number (1-based). Defaults to 1.
    pub page: Option<i64>,
    /// Items per page (max 50). Defaults to 20.
    pub limit: Option<i64>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct WikiGetParams {
    /// Repository owner. Optional if `directory` is provided.
    pub owner: Option<String>,
    /// Repository name. Optional if `directory` is provided.
    pub repo: Option<String>,
    /// Local directory to auto-detect owner/repo from .git/config.
    pub directory: Option<String>,
    /// Wiki page slug (URL-encoded page name).
    pub slug: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct WikiCreateParams {
    /// Repository owner. Optional if `directory` is provided.
    pub owner: Option<String>,
    /// Repository name. Optional if `directory` is provided.
    pub repo: Option<String>,
    /// Local directory to auto-detect owner/repo from .git/config.
    pub directory: Option<String>,
    /// Wiki page title.
    pub title: String,
    /// Wiki page content in markdown.
    pub content: String,
}

pub async fn wiki_list(client: &GiteaClient, params: WikiListParams, default_repo: Option<&RepoInfo>) -> Result<CallToolResult> {
    let (owner, repo) = resolve_owner_repo(&params.owner, &params.repo, &params.directory, default_repo)?;
    let mut query: Vec<(&str, String)> = Vec::new();
    query.push(("page", params.page.unwrap_or(1).to_string()));
    query.push(("limit", params.limit.unwrap_or(20).min(50).to_string()));

    let query_refs: Vec<(&str, &str)> = query.iter().map(|(k, v)| (*k, v.as_str())).collect();
    let pages: Vec<serde_json::Value> = match client
        .get_with_query(&format!("/repos/{owner}/{repo}/wiki/pages"), &query_refs)
        .await
    {
        Ok(p) => p,
        Err(crate::error::GiteaError::NotFound(_)) => {
            return Ok(CallToolResult::success(vec![Content::text(
                "No wiki pages found (wiki may be disabled for this repository).",
            )]));
        }
        Err(e) => return Err(e),
    };

    if pages.is_empty() {
        return Ok(CallToolResult::success(vec![Content::text(
            "No wiki pages found.",
        )]));
    }

    let formatted: Vec<String> = pages
        .iter()
        .map(|p| {
            let title = p.get("title").and_then(|v| v.as_str()).unwrap_or("?");
            let sub_url = p.get("sub_url").and_then(|v| v.as_str()).unwrap_or("?");
            format!("- {title} (slug: {sub_url})")
        })
        .collect();

    Ok(CallToolResult::success(vec![Content::text(
        formatted.join("\n"),
    )]))
}

pub async fn wiki_get(client: &GiteaClient, params: WikiGetParams, default_repo: Option<&RepoInfo>) -> Result<CallToolResult> {
    let (owner, repo) = resolve_owner_repo(&params.owner, &params.repo, &params.directory, default_repo)?;
    let page: serde_json::Value = client
        .get(&format!(
            "/repos/{owner}/{repo}/wiki/page/{}",
            params.slug
        ))
        .await?;

    let title = page
        .get("title")
        .and_then(|v| v.as_str())
        .unwrap_or("(untitled)");
    let content = page
        .get("content_base64")
        .and_then(|v| v.as_str())
        .unwrap_or("");

    let decoded = if !content.is_empty() {
        use base64::Engine;
        let clean = content.replace('\n', "");
        base64::engine::general_purpose::STANDARD
            .decode(&clean)
            .ok()
            .and_then(|bytes| String::from_utf8(bytes).ok())
            .unwrap_or_else(|| "(failed to decode content)".to_string())
    } else {
        "(empty page)".to_string()
    };

    Ok(CallToolResult::success(vec![Content::text(format!(
        "## {title}\n\n{decoded}"
    ))]))
}

pub async fn wiki_create(
    client: &GiteaClient,
    params: WikiCreateParams,
    default_repo: Option<&RepoInfo>,
) -> Result<CallToolResult> {
    let (owner, repo) = resolve_owner_repo(&params.owner, &params.repo, &params.directory, default_repo)?;
    use base64::Engine;
    let encoded = base64::engine::general_purpose::STANDARD.encode(params.content.as_bytes());
    let body = serde_json::json!({
        "title": params.title,
        "content_base64": encoded,
    });

    let _page: serde_json::Value = client
        .post(&format!("/repos/{owner}/{repo}/wiki/new"), &body)
        .await?;

    Ok(CallToolResult::success(vec![Content::text(format!(
        "Wiki page created: {}",
        params.title
    ))]))
}
