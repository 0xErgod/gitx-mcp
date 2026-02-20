use rmcp::model::{CallToolResult, Content};
use schemars::JsonSchema;
use serde::Deserialize;

use crate::client::GitClient;
use crate::error::Result;
use crate::response;
use crate::repo_resolver::RepoInfo;
use crate::server::resolve_owner_repo;

#[derive(Debug, Deserialize, JsonSchema)]
pub struct CommitListParams {
    /// Repository owner. Optional if `directory` is provided.
    pub owner: Option<String>,
    /// Repository name. Optional if `directory` is provided.
    pub repo: Option<String>,
    /// Local directory to auto-detect owner/repo from .git/config.
    pub directory: Option<String>,
    /// Branch name, tag name, or commit SHA to list commits from. Defaults to the default branch.
    pub sha: Option<String>,
    /// Filter commits by file path.
    pub path: Option<String>,
    /// Page number (1-based). Defaults to 1.
    pub page: Option<i64>,
    /// Items per page (max 50). Defaults to 20.
    pub limit: Option<i64>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct CommitGetParams {
    /// Repository owner. Optional if `directory` is provided.
    pub owner: Option<String>,
    /// Repository name. Optional if `directory` is provided.
    pub repo: Option<String>,
    /// Local directory to auto-detect owner/repo from .git/config.
    pub directory: Option<String>,
    /// Commit SHA.
    pub sha: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct CommitDiffParams {
    /// Repository owner. Optional if `directory` is provided.
    pub owner: Option<String>,
    /// Repository name. Optional if `directory` is provided.
    pub repo: Option<String>,
    /// Local directory to auto-detect owner/repo from .git/config.
    pub directory: Option<String>,
    /// Commit SHA.
    pub sha: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct CommitCompareParams {
    /// Repository owner. Optional if `directory` is provided.
    pub owner: Option<String>,
    /// Repository name. Optional if `directory` is provided.
    pub repo: Option<String>,
    /// Local directory to auto-detect owner/repo from .git/config.
    pub directory: Option<String>,
    /// Base ref (branch, tag, or SHA).
    pub base: String,
    /// Head ref (branch, tag, or SHA).
    pub head: String,
}

pub async fn commit_list(
    client: &dyn GitClient,
    params: CommitListParams,
    default_repo: Option<&RepoInfo>,
) -> Result<CallToolResult> {
    let (owner, repo) = resolve_owner_repo(&params.owner, &params.repo, &params.directory, default_repo)?;
    let mut query: Vec<(&str, String)> = Vec::new();

    if let Some(sha) = &params.sha {
        query.push(("sha", sha.clone()));
    }
    if let Some(path) = &params.path {
        query.push(("path", path.clone()));
    }
    query.push(("page", params.page.unwrap_or(1).to_string()));
    query.push(("limit", params.limit.unwrap_or(20).min(50).to_string()));

    let query_refs: Vec<(&str, &str)> = query.iter().map(|(k, v)| (*k, v.as_str())).collect();
    let val = client
        .get_json_with_query(&format!("/repos/{owner}/{repo}/commits"), &query_refs)
        .await?;
    let commits = val.as_array().cloned().unwrap_or_default();

    Ok(CallToolResult::success(vec![Content::text(
        response::format_commit_list(&commits),
    )]))
}

pub async fn commit_get(client: &dyn GitClient, params: CommitGetParams, default_repo: Option<&RepoInfo>) -> Result<CallToolResult> {
    let (owner, repo) = resolve_owner_repo(&params.owner, &params.repo, &params.directory, default_repo)?;
    let commit = client
        .get_json(&format!(
            "/repos/{owner}/{repo}/git/commits/{}",
            params.sha
        ))
        .await?;

    Ok(CallToolResult::success(vec![Content::text(
        response::format_commit(&commit),
    )]))
}

pub async fn commit_diff(
    client: &dyn GitClient,
    params: CommitDiffParams,
    default_repo: Option<&RepoInfo>,
) -> Result<CallToolResult> {
    let (owner, repo) = resolve_owner_repo(&params.owner, &params.repo, &params.directory, default_repo)?;
    let diff = client
        .get_raw(&format!(
            "/repos/{owner}/{repo}/git/commits/{}.diff",
            params.sha
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

pub async fn commit_compare(
    client: &dyn GitClient,
    params: CommitCompareParams,
    default_repo: Option<&RepoInfo>,
) -> Result<CallToolResult> {
    let (owner, repo) = resolve_owner_repo(&params.owner, &params.repo, &params.directory, default_repo)?;
    let result = client
        .get_json(&format!(
            "/repos/{owner}/{repo}/compare/{}...{}",
            params.base, params.head
        ))
        .await?;

    let mut output = Vec::new();

    // Show commits between the two refs
    if let Some(commits) = result.get("commits").and_then(|v| v.as_array()) {
        output.push(format!("**Commits:** {}", commits.len()));
        for c in commits {
            let sha = c
                .get("sha")
                .and_then(|v| v.as_str())
                .map(|s| &s[..7.min(s.len())])
                .unwrap_or("???????");
            let msg = c
                .get("commit")
                .and_then(|v| v.get("message"))
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .lines()
                .next()
                .unwrap_or("");
            output.push(format!("- `{sha}` {msg}"));
        }
    }

    // Show changed files count
    if let Some(files) = result.get("files").and_then(|v| v.as_array()) {
        output.push(format!("\n**Changed files:** {}", files.len()));
        for f in files.iter().take(50) {
            let filename = f
                .get("filename")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown");
            let status = f
                .get("status")
                .and_then(|v| v.as_str())
                .unwrap_or("modified");
            output.push(format!("- {filename} ({status})"));
        }
    }

    if output.is_empty() {
        output.push("No differences found.".to_string());
    }

    Ok(CallToolResult::success(vec![Content::text(
        output.join("\n"),
    )]))
}
