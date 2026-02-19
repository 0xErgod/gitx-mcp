use rmcp::model::{CallToolResult, Content};
use schemars::JsonSchema;
use serde::Deserialize;

use crate::client::GiteaClient;
use crate::error::Result;
use crate::response;
use crate::server::resolve_owner_repo;

#[derive(Debug, Deserialize, JsonSchema)]
pub struct ReleaseListParams {
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
pub struct ReleaseGetParams {
    /// Repository owner. Optional if `directory` is provided.
    pub owner: Option<String>,
    /// Repository name. Optional if `directory` is provided.
    pub repo: Option<String>,
    /// Local directory to auto-detect owner/repo from .git/config.
    pub directory: Option<String>,
    /// Release ID (from release_list).
    pub id: i64,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct ReleaseCreateParams {
    /// Repository owner. Optional if `directory` is provided.
    pub owner: Option<String>,
    /// Repository name. Optional if `directory` is provided.
    pub repo: Option<String>,
    /// Local directory to auto-detect owner/repo from .git/config.
    pub directory: Option<String>,
    /// Tag name for the release.
    pub tag_name: String,
    /// Release title.
    pub name: Option<String>,
    /// Release notes body.
    pub body: Option<String>,
    /// Whether this is a draft release.
    pub draft: Option<bool>,
    /// Whether this is a prerelease.
    pub prerelease: Option<bool>,
    /// Branch or commit SHA to tag (if tag doesn't exist yet).
    pub target_commitish: Option<String>,
}

pub async fn release_list(
    client: &GiteaClient,
    params: ReleaseListParams,
) -> Result<CallToolResult> {
    let (owner, repo) = resolve_owner_repo(&params.owner, &params.repo, &params.directory)?;
    let mut query: Vec<(&str, String)> = Vec::new();
    query.push(("page", params.page.unwrap_or(1).to_string()));
    query.push(("limit", params.limit.unwrap_or(20).min(50).to_string()));

    let query_refs: Vec<(&str, &str)> = query.iter().map(|(k, v)| (*k, v.as_str())).collect();
    let releases: Vec<serde_json::Value> = client
        .get_with_query(&format!("/repos/{owner}/{repo}/releases"), &query_refs)
        .await?;

    if releases.is_empty() {
        return Ok(CallToolResult::success(vec![Content::text(
            "No releases found.",
        )]));
    }

    let formatted: Vec<String> = releases
        .iter()
        .map(|r| {
            let tag = r
                .get("tag_name")
                .and_then(|v| v.as_str())
                .unwrap_or("?");
            let name = r.get("name").and_then(|v| v.as_str()).unwrap_or(tag);
            let id = r.get("id").and_then(|v| v.as_i64()).unwrap_or(0);
            let draft = r.get("draft").and_then(|v| v.as_bool()).unwrap_or(false);
            let prerelease = r
                .get("prerelease")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            let mut flags = Vec::new();
            if draft {
                flags.push("draft");
            }
            if prerelease {
                flags.push("prerelease");
            }
            let flag_str = if flags.is_empty() {
                String::new()
            } else {
                format!(" [{}]", flags.join(", "))
            };
            format!("- {name} ({tag}) [id: {id}]{flag_str}")
        })
        .collect();

    Ok(CallToolResult::success(vec![Content::text(
        formatted.join("\n"),
    )]))
}

pub async fn release_get(
    client: &GiteaClient,
    params: ReleaseGetParams,
) -> Result<CallToolResult> {
    let (owner, repo) = resolve_owner_repo(&params.owner, &params.repo, &params.directory)?;
    let release: serde_json::Value = client
        .get(&format!("/repos/{owner}/{repo}/releases/{}", params.id))
        .await?;

    Ok(CallToolResult::success(vec![Content::text(
        response::format_value(&release),
    )]))
}

pub async fn release_create(
    client: &GiteaClient,
    params: ReleaseCreateParams,
) -> Result<CallToolResult> {
    let (owner, repo) = resolve_owner_repo(&params.owner, &params.repo, &params.directory)?;
    let mut body = serde_json::json!({ "tag_name": params.tag_name });

    if let Some(name) = &params.name {
        body["name"] = serde_json::Value::String(name.clone());
    }
    if let Some(b) = &params.body {
        body["body"] = serde_json::Value::String(b.clone());
    }
    if let Some(draft) = params.draft {
        body["draft"] = serde_json::Value::Bool(draft);
    }
    if let Some(pre) = params.prerelease {
        body["prerelease"] = serde_json::Value::Bool(pre);
    }
    if let Some(target) = &params.target_commitish {
        body["target_commitish"] = serde_json::Value::String(target.clone());
    }

    let release: serde_json::Value = client
        .post(&format!("/repos/{owner}/{repo}/releases"), &body)
        .await?;

    let tag = release
        .get("tag_name")
        .and_then(|v| v.as_str())
        .unwrap_or(&params.tag_name);

    Ok(CallToolResult::success(vec![Content::text(format!(
        "Release created: {tag}"
    ))]))
}
