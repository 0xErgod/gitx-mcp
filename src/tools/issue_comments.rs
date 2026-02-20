use rmcp::model::{CallToolResult, Content};
use schemars::JsonSchema;
use serde::Deserialize;

use crate::client::GitClient;
use crate::error::Result;
use crate::response;
use crate::repo_resolver::RepoInfo;
use crate::server::resolve_owner_repo;

#[derive(Debug, Deserialize, JsonSchema)]
pub struct IssueCommentListParams {
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
pub struct IssueCommentCreateParams {
    /// Repository owner. Optional if `directory` is provided.
    pub owner: Option<String>,
    /// Repository name. Optional if `directory` is provided.
    pub repo: Option<String>,
    /// Local directory to auto-detect owner/repo from .git/config.
    pub directory: Option<String>,
    /// Issue number.
    pub index: i64,
    /// Comment body in markdown.
    pub body: String,
}

pub async fn issue_comment_list(
    client: &dyn GitClient,
    params: IssueCommentListParams,
    default_repo: Option<&RepoInfo>,
) -> Result<CallToolResult> {
    let (owner, repo) = resolve_owner_repo(&params.owner, &params.repo, &params.directory, default_repo)?;
    let val = client
        .get_json(&format!(
            "/repos/{owner}/{repo}/issues/{}/comments",
            params.index
        ))
        .await?;
    let comments = val.as_array().cloned().unwrap_or_default();

    Ok(CallToolResult::success(vec![Content::text(
        response::format_comment_list(&comments),
    )]))
}

pub async fn issue_comment_create(
    client: &dyn GitClient,
    params: IssueCommentCreateParams,
    default_repo: Option<&RepoInfo>,
) -> Result<CallToolResult> {
    let (owner, repo) = resolve_owner_repo(&params.owner, &params.repo, &params.directory, default_repo)?;
    let body = serde_json::json!({ "body": params.body });
    let comment = client
        .post_json(
            &format!("/repos/{owner}/{repo}/issues/{}/comments", params.index),
            &body,
        )
        .await?;

    Ok(CallToolResult::success(vec![Content::text(
        response::format_comment(&comment),
    )]))
}
