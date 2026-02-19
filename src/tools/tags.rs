use rmcp::model::{CallToolResult, Content};
use schemars::JsonSchema;
use serde::Deserialize;

use crate::client::GiteaClient;
use crate::error::Result;
use crate::server::resolve_owner_repo;

#[derive(Debug, Deserialize, JsonSchema)]
pub struct TagListParams {
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
pub struct TagCreateParams {
    /// Repository owner. Optional if `directory` is provided.
    pub owner: Option<String>,
    /// Repository name. Optional if `directory` is provided.
    pub repo: Option<String>,
    /// Local directory to auto-detect owner/repo from .git/config.
    pub directory: Option<String>,
    /// Tag name.
    pub tag_name: String,
    /// Commit SHA or branch to tag.
    pub target: Option<String>,
    /// Tag message (creates annotated tag if provided).
    pub message: Option<String>,
}

pub async fn tag_list(client: &GiteaClient, params: TagListParams) -> Result<CallToolResult> {
    let (owner, repo) = resolve_owner_repo(&params.owner, &params.repo, &params.directory)?;
    let mut query: Vec<(&str, String)> = Vec::new();
    query.push(("page", params.page.unwrap_or(1).to_string()));
    query.push(("limit", params.limit.unwrap_or(20).min(50).to_string()));

    let query_refs: Vec<(&str, &str)> = query.iter().map(|(k, v)| (*k, v.as_str())).collect();
    let tags: Vec<serde_json::Value> = client
        .get_with_query(&format!("/repos/{owner}/{repo}/tags"), &query_refs)
        .await?;

    if tags.is_empty() {
        return Ok(CallToolResult::success(vec![Content::text(
            "No tags found.",
        )]));
    }

    let formatted: Vec<String> = tags
        .iter()
        .map(|t| {
            let name = t.get("name").and_then(|v| v.as_str()).unwrap_or("?");
            let sha = t
                .get("commit")
                .and_then(|v| v.get("sha"))
                .and_then(|v| v.as_str())
                .map(|s| &s[..7.min(s.len())])
                .unwrap_or("???????");
            format!("- {name} (`{sha}`)")
        })
        .collect();

    Ok(CallToolResult::success(vec![Content::text(
        formatted.join("\n"),
    )]))
}

pub async fn tag_create(client: &GiteaClient, params: TagCreateParams) -> Result<CallToolResult> {
    let (owner, repo) = resolve_owner_repo(&params.owner, &params.repo, &params.directory)?;
    let mut body = serde_json::json!({ "tag_name": params.tag_name });

    if let Some(target) = &params.target {
        body["target"] = serde_json::Value::String(target.clone());
    }
    if let Some(msg) = &params.message {
        body["message"] = serde_json::Value::String(msg.clone());
    }

    let tag: serde_json::Value = client
        .post(&format!("/repos/{owner}/{repo}/tags"), &body)
        .await?;

    let name = tag
        .get("name")
        .and_then(|v| v.as_str())
        .unwrap_or(&params.tag_name);

    Ok(CallToolResult::success(vec![Content::text(format!(
        "Tag created: {name}"
    ))]))
}
