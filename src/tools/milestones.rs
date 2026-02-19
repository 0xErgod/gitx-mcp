use rmcp::model::{CallToolResult, Content};
use schemars::JsonSchema;
use serde::Deserialize;

use crate::client::GiteaClient;
use crate::error::Result;
use crate::response;
use crate::repo_resolver::RepoInfo;
use crate::server::resolve_owner_repo;

#[derive(Debug, Deserialize, JsonSchema)]
pub struct MilestoneListParams {
    /// Repository owner. Optional if `directory` is provided.
    pub owner: Option<String>,
    /// Repository name. Optional if `directory` is provided.
    pub repo: Option<String>,
    /// Local directory to auto-detect owner/repo from .git/config.
    pub directory: Option<String>,
    /// Filter by state: open, closed, or all. Defaults to open.
    pub state: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct MilestoneGetParams {
    /// Repository owner. Optional if `directory` is provided.
    pub owner: Option<String>,
    /// Repository name. Optional if `directory` is provided.
    pub repo: Option<String>,
    /// Local directory to auto-detect owner/repo from .git/config.
    pub directory: Option<String>,
    /// Milestone ID (from milestone_list).
    pub id: i64,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct MilestoneCreateParams {
    /// Repository owner. Optional if `directory` is provided.
    pub owner: Option<String>,
    /// Repository name. Optional if `directory` is provided.
    pub repo: Option<String>,
    /// Local directory to auto-detect owner/repo from .git/config.
    pub directory: Option<String>,
    /// Milestone title.
    pub title: String,
    /// Milestone description.
    pub description: Option<String>,
    /// Due date in ISO 8601 format (e.g. "2025-12-31T00:00:00Z").
    pub due_on: Option<String>,
}

pub async fn milestone_list(
    client: &GiteaClient,
    params: MilestoneListParams,
    default_repo: Option<&RepoInfo>,
) -> Result<CallToolResult> {
    let (owner, repo) = resolve_owner_repo(&params.owner, &params.repo, &params.directory, default_repo)?;
    let state = params.state.unwrap_or_else(|| "open".to_string());
    let query: Vec<(&str, &str)> = vec![("state", state.as_str())];

    let milestones: Vec<serde_json::Value> = client
        .get_with_query(&format!("/repos/{owner}/{repo}/milestones"), &query)
        .await?;

    if milestones.is_empty() {
        return Ok(CallToolResult::success(vec![Content::text(
            "No milestones found.",
        )]));
    }

    let formatted: Vec<String> = milestones
        .iter()
        .map(|m| {
            let title = m.get("title").and_then(|v| v.as_str()).unwrap_or("?");
            let state = m.get("state").and_then(|v| v.as_str()).unwrap_or("?");
            let id = m.get("id").and_then(|v| v.as_i64()).unwrap_or(0);
            let open = m
                .get("open_issues")
                .and_then(|v| v.as_i64())
                .unwrap_or(0);
            let closed = m
                .get("closed_issues")
                .and_then(|v| v.as_i64())
                .unwrap_or(0);
            format!("- {title} ({state}) [id: {id}] - {open} open, {closed} closed")
        })
        .collect();

    Ok(CallToolResult::success(vec![Content::text(
        formatted.join("\n"),
    )]))
}

pub async fn milestone_get(
    client: &GiteaClient,
    params: MilestoneGetParams,
    default_repo: Option<&RepoInfo>,
) -> Result<CallToolResult> {
    let (owner, repo) = resolve_owner_repo(&params.owner, &params.repo, &params.directory, default_repo)?;
    let milestone: serde_json::Value = client
        .get(&format!(
            "/repos/{owner}/{repo}/milestones/{}",
            params.id
        ))
        .await?;

    Ok(CallToolResult::success(vec![Content::text(
        response::format_value(&milestone),
    )]))
}

pub async fn milestone_create(
    client: &GiteaClient,
    params: MilestoneCreateParams,
    default_repo: Option<&RepoInfo>,
) -> Result<CallToolResult> {
    let (owner, repo) = resolve_owner_repo(&params.owner, &params.repo, &params.directory, default_repo)?;
    let mut body = serde_json::json!({ "title": params.title });

    if let Some(desc) = &params.description {
        body["description"] = serde_json::Value::String(desc.clone());
    }
    if let Some(due) = &params.due_on {
        body["due_on"] = serde_json::Value::String(due.clone());
    }

    let milestone: serde_json::Value = client
        .post(&format!("/repos/{owner}/{repo}/milestones"), &body)
        .await?;

    let title = milestone
        .get("title")
        .and_then(|v| v.as_str())
        .unwrap_or(&params.title);

    Ok(CallToolResult::success(vec![Content::text(format!(
        "Milestone created: {title}"
    ))]))
}
