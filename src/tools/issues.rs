use rmcp::model::{CallToolResult, Content};
use schemars::JsonSchema;
use serde::Deserialize;

use crate::client::GitClient;
use crate::error::Result;
use crate::response;
use crate::repo_resolver::RepoInfo;
use crate::server::resolve_owner_repo;

#[derive(Debug, Deserialize, JsonSchema)]
pub struct IssueListParams {
    /// Repository owner. Optional if `directory` is provided.
    pub owner: Option<String>,
    /// Repository name. Optional if `directory` is provided.
    pub repo: Option<String>,
    /// Local directory to auto-detect owner/repo from .git/config.
    pub directory: Option<String>,
    /// Filter by state: open, closed, or all. Defaults to open.
    pub state: Option<String>,
    /// Filter by comma-separated label names.
    pub labels: Option<String>,
    /// Filter by milestone name.
    pub milestone: Option<String>,
    /// Page number (1-based). Defaults to 1.
    pub page: Option<i64>,
    /// Items per page (max 50). Defaults to 20.
    pub limit: Option<i64>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct IssueGetParams {
    /// Repository owner. Optional if `directory` is provided.
    pub owner: Option<String>,
    /// Repository name. Optional if `directory` is provided.
    pub repo: Option<String>,
    /// Local directory to auto-detect owner/repo from .git/config.
    pub directory: Option<String>,
    /// Issue number.
    pub index: i64,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct IssueCreateParams {
    /// Repository owner. Optional if `directory` is provided.
    pub owner: Option<String>,
    /// Repository name. Optional if `directory` is provided.
    pub repo: Option<String>,
    /// Local directory to auto-detect owner/repo from .git/config.
    pub directory: Option<String>,
    /// Issue title.
    pub title: String,
    /// Issue body/description in markdown.
    pub body: Option<String>,
    /// Label IDs to assign (from label_list).
    pub labels: Option<Vec<i64>>,
    /// Milestone ID (from milestone_list).
    pub milestone: Option<i64>,
    /// Usernames to assign.
    pub assignees: Option<Vec<String>>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct IssueEditParams {
    /// Repository owner. Optional if `directory` is provided.
    pub owner: Option<String>,
    /// Repository name. Optional if `directory` is provided.
    pub repo: Option<String>,
    /// Local directory to auto-detect owner/repo from .git/config.
    pub directory: Option<String>,
    /// Issue number.
    pub index: i64,
    /// New title.
    pub title: Option<String>,
    /// New body.
    pub body: Option<String>,
    /// New state: open or closed.
    pub state: Option<String>,
    /// Label IDs to set, replaces existing (from label_list).
    pub labels: Option<Vec<i64>>,
    /// Milestone ID (from milestone_list).
    pub milestone: Option<i64>,
    /// Usernames to assign (replaces existing).
    pub assignees: Option<Vec<String>>,
}

pub async fn issue_list(client: &dyn GitClient, params: IssueListParams, default_repo: Option<&RepoInfo>) -> Result<CallToolResult> {
    use crate::platform::Platform;

    let (owner, repo) = resolve_owner_repo(&params.owner, &params.repo, &params.directory, default_repo)?;
    let mut query: Vec<(&str, String)> = Vec::new();

    let state = params.state.unwrap_or_else(|| "open".to_string());
    query.push(("state", state));

    // Gitea needs type=issues to exclude PRs; GitHub doesn't need this
    if client.platform() == Platform::Gitea {
        query.push(("type", "issues".to_string()));
    }

    if let Some(labels) = &params.labels {
        query.push(("labels", labels.clone()));
    }
    if let Some(milestone) = &params.milestone {
        query.push(("milestones", milestone.clone()));
    }
    query.push(("page", params.page.unwrap_or(1).to_string()));
    query.push(("limit", params.limit.unwrap_or(20).min(50).to_string()));

    let query_refs: Vec<(&str, &str)> = query.iter().map(|(k, v)| (*k, v.as_str())).collect();
    let val = client
        .get_json_with_query(&format!("/repos/{owner}/{repo}/issues"), &query_refs)
        .await?;
    let all_items = val.as_array().cloned().unwrap_or_default();

    // On GitHub, filter out pull requests (they have a "pull_request" key)
    let issues: Vec<serde_json::Value> = if client.platform() == Platform::GitHub {
        all_items.into_iter().filter(|i| i.get("pull_request").is_none()).collect()
    } else {
        all_items
    };

    Ok(CallToolResult::success(vec![Content::text(
        response::format_issue_list(&issues),
    )]))
}

pub async fn issue_get(client: &dyn GitClient, params: IssueGetParams, default_repo: Option<&RepoInfo>) -> Result<CallToolResult> {
    let (owner, repo) = resolve_owner_repo(&params.owner, &params.repo, &params.directory, default_repo)?;
    let issue = client
        .get_json(&format!("/repos/{owner}/{repo}/issues/{}", params.index))
        .await?;

    Ok(CallToolResult::success(vec![Content::text(
        response::format_issue(&issue),
    )]))
}

pub async fn issue_create(
    client: &dyn GitClient,
    params: IssueCreateParams,
    default_repo: Option<&RepoInfo>,
) -> Result<CallToolResult> {
    let (owner, repo) = resolve_owner_repo(&params.owner, &params.repo, &params.directory, default_repo)?;
    let mut body = serde_json::json!({ "title": params.title });

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

    let issue = client
        .post_json(&format!("/repos/{owner}/{repo}/issues"), &body)
        .await?;

    Ok(CallToolResult::success(vec![Content::text(
        response::format_issue(&issue),
    )]))
}

pub async fn issue_edit(
    client: &dyn GitClient,
    params: IssueEditParams,
    default_repo: Option<&RepoInfo>,
) -> Result<CallToolResult> {
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
    if let Some(milestone) = params.milestone {
        body["milestone"] = serde_json::json!(milestone);
    }
    if let Some(assignees) = &params.assignees {
        body["assignees"] = serde_json::json!(assignees);
    }

    let issue = client
        .patch_json(
            &format!("/repos/{owner}/{repo}/issues/{}", params.index),
            &body,
        )
        .await?;

    Ok(CallToolResult::success(vec![Content::text(
        response::format_issue(&issue),
    )]))
}
