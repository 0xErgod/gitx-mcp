use rmcp::model::{CallToolResult, Content};
use schemars::JsonSchema;
use serde::Deserialize;

use crate::client::GitClient;
use crate::error::Result;
use crate::response;
use crate::repo_resolver::RepoInfo;
use crate::server::resolve_owner_repo;

#[derive(Debug, Deserialize, JsonSchema)]
pub struct FileReadParams {
    /// Repository owner. Optional if `directory` is provided.
    pub owner: Option<String>,
    /// Repository name. Optional if `directory` is provided.
    pub repo: Option<String>,
    /// Local directory to auto-detect owner/repo from .git/config.
    pub directory: Option<String>,
    /// File path within the repository.
    pub path: String,
    /// Git ref (branch, tag, or commit SHA). Defaults to the default branch.
    #[serde(rename = "ref")]
    pub git_ref: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct FileListParams {
    /// Repository owner. Optional if `directory` is provided.
    pub owner: Option<String>,
    /// Repository name. Optional if `directory` is provided.
    pub repo: Option<String>,
    /// Local directory to auto-detect owner/repo from .git/config.
    pub directory: Option<String>,
    /// Directory path within the repository. Empty or "/" for root.
    pub path: Option<String>,
    /// Git ref (branch, tag, or commit SHA). Defaults to the default branch.
    #[serde(rename = "ref")]
    pub git_ref: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct FileCreateParams {
    /// Repository owner. Optional if `directory` is provided.
    pub owner: Option<String>,
    /// Repository name. Optional if `directory` is provided.
    pub repo: Option<String>,
    /// Local directory to auto-detect owner/repo from .git/config.
    pub directory: Option<String>,
    /// File path to create.
    pub path: String,
    /// File content (plain text, will be base64-encoded automatically).
    pub content: String,
    /// Commit message.
    pub message: String,
    /// Branch to commit to. Defaults to the default branch.
    pub branch: Option<String>,
    /// New branch to create from `branch`.
    pub new_branch: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct FileUpdateParams {
    /// Repository owner. Optional if `directory` is provided.
    pub owner: Option<String>,
    /// Repository name. Optional if `directory` is provided.
    pub repo: Option<String>,
    /// Local directory to auto-detect owner/repo from .git/config.
    pub directory: Option<String>,
    /// File path to update.
    pub path: String,
    /// New file content (plain text, will be base64-encoded automatically).
    pub content: String,
    /// SHA of the file being replaced (from file_read).
    pub sha: String,
    /// Commit message.
    pub message: String,
    /// Branch to commit to.
    pub branch: Option<String>,
    /// New branch to create from `branch`.
    pub new_branch: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct FileDeleteParams {
    /// Repository owner. Optional if `directory` is provided.
    pub owner: Option<String>,
    /// Repository name. Optional if `directory` is provided.
    pub repo: Option<String>,
    /// Local directory to auto-detect owner/repo from .git/config.
    pub directory: Option<String>,
    /// File path to delete.
    pub path: String,
    /// SHA of the file being deleted (from file_read).
    pub sha: String,
    /// Commit message.
    pub message: String,
    /// Branch to commit to.
    pub branch: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct TreeGetParams {
    /// Repository owner. Optional if `directory` is provided.
    pub owner: Option<String>,
    /// Repository name. Optional if `directory` is provided.
    pub repo: Option<String>,
    /// Local directory to auto-detect owner/repo from .git/config.
    pub directory: Option<String>,
    /// Git ref (branch, tag, or SHA). Defaults to the default branch.
    #[serde(rename = "ref")]
    pub git_ref: Option<String>,
}

pub async fn file_read(client: &dyn GitClient, params: FileReadParams, default_repo: Option<&RepoInfo>) -> Result<CallToolResult> {
    let (owner, repo) = resolve_owner_repo(&params.owner, &params.repo, &params.directory, default_repo)?;
    let path = params.path.trim_start_matches('/');
    let mut url = format!("/repos/{owner}/{repo}/contents/{path}");

    if let Some(git_ref) = &params.git_ref {
        url = format!("{url}?ref={git_ref}");
    }

    let file = client.get_json(&url).await?;

    Ok(CallToolResult::success(vec![Content::text(
        response::format_file_content(&file),
    )]))
}

pub async fn file_list(client: &dyn GitClient, params: FileListParams, default_repo: Option<&RepoInfo>) -> Result<CallToolResult> {
    let (owner, repo) = resolve_owner_repo(&params.owner, &params.repo, &params.directory, default_repo)?;
    let path = params
        .path
        .as_deref()
        .unwrap_or("")
        .trim_start_matches('/');
    let mut url = format!("/repos/{owner}/{repo}/contents/{path}");

    if let Some(git_ref) = &params.git_ref {
        url = format!("{url}?ref={git_ref}");
    }

    let val = client.get_json(&url).await?;
    let entries = val.as_array().cloned().unwrap_or_default();

    Ok(CallToolResult::success(vec![Content::text(
        response::format_file_list(&entries),
    )]))
}

pub async fn file_create(
    client: &dyn GitClient,
    params: FileCreateParams,
    default_repo: Option<&RepoInfo>,
) -> Result<CallToolResult> {
    let (owner, repo) = resolve_owner_repo(&params.owner, &params.repo, &params.directory, default_repo)?;
    let path = params.path.trim_start_matches('/');

    use base64::Engine;
    let encoded = base64::engine::general_purpose::STANDARD.encode(params.content.as_bytes());

    let mut body = serde_json::json!({
        "content": encoded,
        "message": params.message,
    });

    if let Some(branch) = &params.branch {
        body["branch"] = serde_json::Value::String(branch.clone());
    }
    if let Some(new_branch) = &params.new_branch {
        body["new_branch"] = serde_json::Value::String(new_branch.clone());
    }

    let result = client
        .post_json(&format!("/repos/{owner}/{repo}/contents/{path}"), &body)
        .await?;

    let file_path = result
        .get("content")
        .and_then(|v| v.get("path"))
        .and_then(|v| v.as_str())
        .unwrap_or(path);

    Ok(CallToolResult::success(vec![Content::text(format!(
        "File created: {file_path}"
    ))]))
}

pub async fn file_update(
    client: &dyn GitClient,
    params: FileUpdateParams,
    default_repo: Option<&RepoInfo>,
) -> Result<CallToolResult> {
    let (owner, repo) = resolve_owner_repo(&params.owner, &params.repo, &params.directory, default_repo)?;
    let path = params.path.trim_start_matches('/');

    use base64::Engine;
    let encoded = base64::engine::general_purpose::STANDARD.encode(params.content.as_bytes());

    let mut body = serde_json::json!({
        "content": encoded,
        "sha": params.sha,
        "message": params.message,
    });

    if let Some(branch) = &params.branch {
        body["branch"] = serde_json::Value::String(branch.clone());
    }
    if let Some(new_branch) = &params.new_branch {
        body["new_branch"] = serde_json::Value::String(new_branch.clone());
    }

    let _result = client
        .put_json(&format!("/repos/{owner}/{repo}/contents/{path}"), &body)
        .await?;

    Ok(CallToolResult::success(vec![Content::text(format!(
        "File updated: {path}"
    ))]))
}

pub async fn file_delete(
    client: &dyn GitClient,
    params: FileDeleteParams,
    default_repo: Option<&RepoInfo>,
) -> Result<CallToolResult> {
    let (owner, repo) = resolve_owner_repo(&params.owner, &params.repo, &params.directory, default_repo)?;
    let path = params.path.trim_start_matches('/');

    let mut body = serde_json::json!({
        "sha": params.sha,
        "message": params.message,
    });

    if let Some(branch) = &params.branch {
        body["branch"] = serde_json::Value::String(branch.clone());
    }

    client
        .delete_with_body(&format!("/repos/{owner}/{repo}/contents/{path}"), &body)
        .await?;

    Ok(CallToolResult::success(vec![Content::text(format!(
        "File deleted: {path}"
    ))]))
}

pub async fn tree_get(client: &dyn GitClient, params: TreeGetParams, default_repo: Option<&RepoInfo>) -> Result<CallToolResult> {
    let (owner, repo) = resolve_owner_repo(&params.owner, &params.repo, &params.directory, default_repo)?;
    let git_ref = params.git_ref.as_deref().unwrap_or("HEAD");

    let tree = client
        .get_json(&format!(
            "/repos/{owner}/{repo}/git/trees/{git_ref}?recursive=true"
        ))
        .await?;

    let entries = tree
        .get("tree")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();

    if entries.is_empty() {
        return Ok(CallToolResult::success(vec![Content::text(
            "No files found in tree.",
        )]));
    }

    let formatted: Vec<String> = entries
        .iter()
        .map(|e| {
            let path = e.get("path").and_then(|v| v.as_str()).unwrap_or("?");
            let entry_type = e.get("type").and_then(|v| v.as_str()).unwrap_or("blob");
            let icon = if entry_type == "tree" { "/" } else { "" };
            format!("{path}{icon}")
        })
        .collect();

    Ok(CallToolResult::success(vec![Content::text(
        formatted.join("\n"),
    )]))
}
