use rmcp::model::{CallToolResult, Content};
use schemars::JsonSchema;
use serde::Deserialize;

use crate::client::GitClient;
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

pub async fn pr_list(client: &dyn GitClient, params: PrListParams, default_repo: Option<&RepoInfo>) -> Result<CallToolResult> {
    let (owner, repo) = resolve_owner_repo(&params.owner, &params.repo, &params.directory, default_repo)?;
    let mut query: Vec<(&str, String)> = Vec::new();

    let state = params.state.unwrap_or_else(|| "open".to_string());
    query.push(("state", state));
    query.push(("page", params.page.unwrap_or(1).to_string()));
    query.push(("limit", params.limit.unwrap_or(20).min(50).to_string()));

    let query_refs: Vec<(&str, &str)> = query.iter().map(|(k, v)| (*k, v.as_str())).collect();
    let val = client
        .get_json_with_query(&format!("/repos/{owner}/{repo}/pulls"), &query_refs)
        .await?;
    let prs = val.as_array().cloned().unwrap_or_default();

    Ok(CallToolResult::success(vec![Content::text(
        response::format_pr_list(&prs),
    )]))
}

pub async fn pr_get(client: &dyn GitClient, params: PrGetParams, default_repo: Option<&RepoInfo>) -> Result<CallToolResult> {
    let (owner, repo) = resolve_owner_repo(&params.owner, &params.repo, &params.directory, default_repo)?;
    let pr = client
        .get_json(&format!("/repos/{owner}/{repo}/pulls/{}", params.index))
        .await?;

    Ok(CallToolResult::success(vec![Content::text(
        response::format_pull_request(&pr),
    )]))
}

pub async fn pr_create(client: &dyn GitClient, params: PrCreateParams, default_repo: Option<&RepoInfo>) -> Result<CallToolResult> {
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

    let pr = client
        .post_json(&format!("/repos/{owner}/{repo}/pulls"), &body)
        .await?;

    Ok(CallToolResult::success(vec![Content::text(
        response::format_pull_request(&pr),
    )]))
}

pub async fn pr_edit(client: &dyn GitClient, params: PrEditParams, default_repo: Option<&RepoInfo>) -> Result<CallToolResult> {
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

    let pr = client
        .patch_json(
            &format!("/repos/{owner}/{repo}/pulls/{}", params.index),
            &body,
        )
        .await?;

    Ok(CallToolResult::success(vec![Content::text(
        response::format_pull_request(&pr),
    )]))
}

pub async fn pr_merge(client: &dyn GitClient, params: PrMergeParams, default_repo: Option<&RepoInfo>) -> Result<CallToolResult> {
    use crate::platform::Platform;

    let (owner, repo) = resolve_owner_repo(&params.owner, &params.repo, &params.directory, default_repo)?;
    let style = params.merge_style.unwrap_or_else(|| "merge".to_string());

    let mut body = match client.platform() {
        Platform::Gitea => {
            let mut b = serde_json::json!({ "Do": style });
            if let Some(msg) = &params.merge_message {
                b["merge_message_field"] = serde_json::Value::String(msg.clone());
            }
            b
        }
        Platform::GitHub => {
            let mut b = serde_json::json!({ "merge_method": style });
            if let Some(msg) = &params.merge_message {
                b["commit_message"] = serde_json::Value::String(msg.clone());
            }
            b
        }
    };

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
