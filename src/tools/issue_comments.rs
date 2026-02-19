use rmcp::model::{CallToolResult, Content};
use schemars::JsonSchema;
use serde::Deserialize;

use crate::client::GiteaClient;
use crate::error::Result;
use crate::response;
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
    client: &GiteaClient,
    params: IssueCommentListParams,
) -> Result<CallToolResult> {
    let (owner, repo) = resolve_owner_repo(&params.owner, &params.repo, &params.directory)?;
    let comments: Vec<serde_json::Value> = client
        .get(&format!(
            "/repos/{owner}/{repo}/issues/{}/comments",
            params.index
        ))
        .await?;

    Ok(CallToolResult::success(vec![Content::text(
        response::format_comment_list(&comments),
    )]))
}

pub async fn issue_comment_create(
    client: &GiteaClient,
    params: IssueCommentCreateParams,
) -> Result<CallToolResult> {
    let (owner, repo) = resolve_owner_repo(&params.owner, &params.repo, &params.directory)?;
    let body = serde_json::json!({ "body": params.body });
    let comment: serde_json::Value = client
        .post(
            &format!("/repos/{owner}/{repo}/issues/{}/comments", params.index),
            &body,
        )
        .await?;

    Ok(CallToolResult::success(vec![Content::text(
        response::format_comment(&comment),
    )]))
}
