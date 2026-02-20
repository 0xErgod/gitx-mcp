use rmcp::model::{CallToolResult, Content};
use schemars::JsonSchema;
use serde::Deserialize;

use crate::client::GitClient;
use crate::error::Result;
use crate::repo_resolver::RepoInfo;
use crate::server::resolve_owner_repo;

#[derive(Debug, Deserialize, JsonSchema)]
pub struct PrReviewListParams {
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
pub struct PrReviewCreateParams {
    /// Repository owner. Optional if `directory` is provided.
    pub owner: Option<String>,
    /// Repository name. Optional if `directory` is provided.
    pub repo: Option<String>,
    /// Local directory to auto-detect owner/repo from .git/config.
    pub directory: Option<String>,
    /// Pull request number.
    pub index: i64,
    /// Review event type: APPROVED, REQUEST_CHANGES, or COMMENT.
    pub event: String,
    /// Review body/comment.
    pub body: Option<String>,
}

pub async fn pr_review_list(
    client: &dyn GitClient,
    params: PrReviewListParams,
    default_repo: Option<&RepoInfo>,
) -> Result<CallToolResult> {
    let (owner, repo) = resolve_owner_repo(&params.owner, &params.repo, &params.directory, default_repo)?;
    let val = client
        .get_json(&format!(
            "/repos/{owner}/{repo}/pulls/{}/reviews",
            params.index
        ))
        .await?;
    let reviews = val.as_array().cloned().unwrap_or_default();

    if reviews.is_empty() {
        return Ok(CallToolResult::success(vec![Content::text(
            "No reviews found.",
        )]));
    }

    let formatted: Vec<String> = reviews
        .iter()
        .map(|r| {
            let user = r
                .get("user")
                .and_then(|v| v.get("login"))
                .and_then(|v| v.as_str())
                .unwrap_or("unknown");
            let state = r
                .get("state")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown");
            let body = r
                .get("body")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let id = r.get("id").and_then(|v| v.as_i64()).unwrap_or(0);
            if body.is_empty() {
                format!("- Review #{id} by {user}: {state}")
            } else {
                format!("- Review #{id} by {user}: {state}\n  {body}")
            }
        })
        .collect();

    Ok(CallToolResult::success(vec![Content::text(
        formatted.join("\n"),
    )]))
}

pub async fn pr_review_create(
    client: &dyn GitClient,
    params: PrReviewCreateParams,
    default_repo: Option<&RepoInfo>,
) -> Result<CallToolResult> {
    let (owner, repo) = resolve_owner_repo(&params.owner, &params.repo, &params.directory, default_repo)?;
    let mut body = serde_json::json!({ "event": params.event });

    if let Some(b) = &params.body {
        body["body"] = serde_json::Value::String(b.clone());
    }

    let review = client
        .post_json(
            &format!("/repos/{owner}/{repo}/pulls/{}/reviews", params.index),
            &body,
        )
        .await?;

    let state = review
        .get("state")
        .and_then(|v| v.as_str())
        .unwrap_or("submitted");

    Ok(CallToolResult::success(vec![Content::text(format!(
        "Review submitted: {state}"
    ))]))
}
