use rmcp::model::{CallToolResult, Content};
use schemars::JsonSchema;
use serde::Deserialize;

use crate::client::GitClient;
use crate::error::Result;
use crate::repo_resolver::RepoInfo;
use crate::server::resolve_owner_repo;

#[derive(Debug, Deserialize, JsonSchema)]
pub struct PrFilesParams {
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
pub struct PrDiffParams {
    /// Repository owner. Optional if `directory` is provided.
    pub owner: Option<String>,
    /// Repository name. Optional if `directory` is provided.
    pub repo: Option<String>,
    /// Local directory to auto-detect owner/repo from .git/config.
    pub directory: Option<String>,
    /// Pull request number.
    pub index: i64,
}

pub async fn pr_files(client: &dyn GitClient, params: PrFilesParams, default_repo: Option<&RepoInfo>) -> Result<CallToolResult> {
    let (owner, repo) = resolve_owner_repo(&params.owner, &params.repo, &params.directory, default_repo)?;
    let val = client
        .get_json(&format!(
            "/repos/{owner}/{repo}/pulls/{}/files",
            params.index
        ))
        .await?;
    let files = val.as_array().cloned().unwrap_or_default();

    if files.is_empty() {
        return Ok(CallToolResult::success(vec![Content::text(
            "No changed files.",
        )]));
    }

    let formatted: Vec<String> = files
        .iter()
        .map(|f| {
            let filename = f
                .get("filename")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown");
            let status = f
                .get("status")
                .and_then(|v| v.as_str())
                .unwrap_or("modified");
            let additions = f.get("additions").and_then(|v| v.as_i64()).unwrap_or(0);
            let deletions = f.get("deletions").and_then(|v| v.as_i64()).unwrap_or(0);
            format!("- {filename} ({status}) +{additions} -{deletions}")
        })
        .collect();

    Ok(CallToolResult::success(vec![Content::text(
        formatted.join("\n"),
    )]))
}

pub async fn pr_diff(client: &dyn GitClient, params: PrDiffParams, default_repo: Option<&RepoInfo>) -> Result<CallToolResult> {
    let (owner, repo) = resolve_owner_repo(&params.owner, &params.repo, &params.directory, default_repo)?;
    let diff = client
        .get_raw(&format!(
            "/repos/{owner}/{repo}/pulls/{}.diff",
            params.index
        ))
        .await?;

    if diff.is_empty() {
        return Ok(CallToolResult::success(vec![Content::text(
            "No diff content.",
        )]));
    }

    Ok(CallToolResult::success(vec![Content::text(format!(
        "```diff\n{diff}\n```"
    ))]))
}
