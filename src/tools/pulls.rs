use rmcp::model::{CallToolResult, Content};
use schemars::JsonSchema;
use serde::Deserialize;

use crate::client::GiteaClient;
use crate::error::Result;
use crate::response;
use crate::repo_resolver::RepoInfo;
use crate::server::resolve_owner_repo;

#[derive(Debug, Deserialize, JsonSchema)]
pub struct PrListParams {
    /// Repository owner. Optional if `directory` is provided.
    pub owner: Option<String>,
    /// Repository name. Optional if `directory` is provided.
    pub repo: Option<String>,
    /// Local directory to auto-detect owner/repo from .git/config.
    pub directory: Option<String>,
    /// Filter by state: open, closed, or all. Defaults to open.
    pub state: Option<String>,
    /// Page number (1-based). Defaults to 1.
    pub page: Option<i64>,
    /// Items per page (max 50). Defaults to 20.
    pub limit: Option<i64>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct PrGetParams {
    /// Repository owner. Optional if `directory` is provided.
    pub owner: Option<String>,
    /// Repository name. Optional if `directory` is provided.
    pub repo: Option<String>,
    /// Local directory to auto-detect owner/repo from .git/config.
    pub directory: Option<String>,
    /// Pull request number.
    pub index: i64,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct PrCreateParams {
    /// Repository owner. Optional if `directory` is provided.
    pub owner: Option<String>,
    /// Repository name. Optional if `directory` is provided.
    pub repo: Option<String>,
    /// Local directory to auto-detect owner/repo from .git/config.
    pub directory: Option<String>,
    /// PR title.
    pub title: String,
    /// Head branch (source).
    pub head: String,
    /// Base branch (target).
    pub base: String,
    /// PR body/description.
    pub body: Option<String>,
    /// Label IDs (from label_list).
    pub labels: Option<Vec<i64>>,
    /// Milestone ID (from milestone_list).
    pub milestone: Option<i64>,
    /// Assignee usernames.
    pub assignees: Option<Vec<String>>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct PrEditParams {
    /// Repository owner. Optional if `directory` is provided.
    pub owner: Option<String>,
    /// Repository name. Optional if `directory` is provided.
    pub repo: Option<String>,
    /// Local directory to auto-detect owner/repo from .git/config.
    pub directory: Option<String>,
    /// Pull request number.
    pub index: i64,
    /// New title.
    pub title: Option<String>,
    /// New body.
    pub body: Option<String>,
    /// New state: open or closed.
    pub state: Option<String>,
    /// Label IDs, replaces existing (from label_list).
    pub labels: Option<Vec<i64>>,
    /// Assignee usernames, replaces existing.
    pub assignees: Option<Vec<String>>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct PrMergeParams {
    /// Repository owner. Optional if `directory` is provided.
    pub owner: Option<String>,
    /// Repository name. Optional if `directory` is provided.
    pub repo: Option<String>,
    /// Local directory to auto-detect owner/repo from .git/config.
    pub directory: Option<String>,
    /// Pull request number.
    pub index: i64,
    /// Merge strategy: merge, rebase, or squash. Defaults to merge.
    pub merge_style: Option<String>,
    /// Custom merge commit message.
    pub merge_message: Option<String>,
    /// Delete head branch after merge.
    pub delete_branch_after_merge: Option<bool>,
}

pub async fn pr_list(client: &GiteaClient, params: PrListParams, default_repo: Option<&RepoInfo>) -> Result<CallToolResult> {
    let (owner, repo) = resolve_owner_repo(&params.owner, &params.repo, &params.directory, default_repo)?;
    let mut query: Vec<(&str, String)> = Vec::new();

    let state = params.state.unwrap_or_else(|| "open".to_string());
    query.push(("state", state));
    query.push(("page", params.page.unwrap_or(1).to_string()));
    query.push(("limit", params.limit.unwrap_or(20).min(50).to_string()));

    let query_refs: Vec<(&str, &str)> = query.iter().map(|(k, v)| (*k, v.as_str())).collect();
    let prs: Vec<serde_json::Value> = client
        .get_with_query(&format!("/repos/{owner}/{repo}/pulls"), &query_refs)
        .await?;

    Ok(CallToolResult::success(vec![Content::text(
        response::format_pr_list(&prs),
    )]))
}

pub async fn pr_get(client: &GiteaClient, params: PrGetParams, default_repo: Option<&RepoInfo>) -> Result<CallToolResult> {
    let (owner, repo) = resolve_owner_repo(&params.owner, &params.repo, &params.directory, default_repo)?;
    let pr: serde_json::Value = client
        .get(&format!("/repos/{owner}/{repo}/pulls/{}", params.index))
        .await?;

    Ok(CallToolResult::success(vec![Content::text(
        response::format_pull_request(&pr),
    )]))
}

pub async fn pr_create(client: &GiteaClient, params: PrCreateParams, default_repo: Option<&RepoInfo>) -> Result<CallToolResult> {
    let (owner, repo) = resolve_owner_repo(&params.owner, &params.repo, &params.directory, default_repo)?;
    let mut body = serde_json::json!({
        "title": params.title,
        "head": params.head,
        "base": params.base,
    });

    if let Some(b) = &params.body {
        body["body"] = serde_json::Value::String(b.clone());
    }
    if let Some(labels) = &params.labels {
        body["labels"] = serde_json::json!(labels);
    }
    if let Some(milestone) = params.milestone {
        body["milestone"] = serde_json::json!(milestone);
    }
    if let Some(assignees) = &params.assignees {
        body["assignees"] = serde_json::json!(assignees);
    }

    let pr: serde_json::Value = client
        .post(&format!("/repos/{owner}/{repo}/pulls"), &body)
        .await?;

    Ok(CallToolResult::success(vec![Content::text(
        response::format_pull_request(&pr),
    )]))
}

pub async fn pr_edit(client: &GiteaClient, params: PrEditParams, default_repo: Option<&RepoInfo>) -> Result<CallToolResult> {
    let (owner, repo) = resolve_owner_repo(&params.owner, &params.repo, &params.directory, default_repo)?;
    let mut body = serde_json::json!({});

    if let Some(title) = &params.title {
        body["title"] = serde_json::Value::String(title.clone());
    }
    if let Some(b) = &params.body {
        body["body"] = serde_json::Value::String(b.clone());
    }
    if let Some(state) = &params.state {
        body["state"] = serde_json::Value::String(state.clone());
    }
    if let Some(labels) = &params.labels {
        body["labels"] = serde_json::json!(labels);
    }
    if let Some(assignees) = &params.assignees {
        body["assignees"] = serde_json::json!(assignees);
    }

    let pr: serde_json::Value = client
        .patch(
            &format!("/repos/{owner}/{repo}/pulls/{}", params.index),
            &body,
        )
        .await?;

    Ok(CallToolResult::success(vec![Content::text(
        response::format_pull_request(&pr),
    )]))
}

pub async fn pr_merge(client: &GiteaClient, params: PrMergeParams, default_repo: Option<&RepoInfo>) -> Result<CallToolResult> {
    let (owner, repo) = resolve_owner_repo(&params.owner, &params.repo, &params.directory, default_repo)?;
    let mut body = serde_json::json!({
        "Do": params.merge_style.unwrap_or_else(|| "merge".to_string()),
    });

    if let Some(msg) = &params.merge_message {
        body["merge_message_field"] = serde_json::Value::String(msg.clone());
    }
    if let Some(delete) = params.delete_branch_after_merge {
        body["delete_branch_after_merge"] = serde_json::Value::Bool(delete);
    }

    client
        .post_no_content(
            &format!("/repos/{owner}/{repo}/pulls/{}/merge", params.index),
            &body,
        )
        .await?;

    Ok(CallToolResult::success(vec![Content::text(format!(
        "Pull request #{} merged successfully.",
        params.index
    ))]))
}
