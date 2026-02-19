use rmcp::model::{CallToolResult, Content};
use schemars::JsonSchema;
use serde::Deserialize;

use crate::client::GiteaClient;
use crate::error::Result;
use crate::response;
use crate::server::resolve_owner_repo;

#[derive(Debug, Deserialize, JsonSchema)]
pub struct BranchListParams {
    /// Repository owner. Optional if `directory` is provided.
    pub owner: Option<String>,
    /// Repository name. Optional if `directory` is provided.
    pub repo: Option<String>,
    /// Local directory to auto-detect owner/repo from .git/config.
    pub directory: Option<String>,
    /// Page number (1-based). Defaults to 1.
    pub page: Option<i64>,
    /// Items per page (max 50). Defaults to 20.
    pub limit: Option<i64>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct BranchCreateParams {
    /// Repository owner. Optional if `directory` is provided.
    pub owner: Option<String>,
    /// Repository name. Optional if `directory` is provided.
    pub repo: Option<String>,
    /// Local directory to auto-detect owner/repo from .git/config.
    pub directory: Option<String>,
    /// Name for the new branch.
    pub new_branch_name: String,
    /// Source branch name or commit SHA to create the new branch from. Defaults to the default branch.
    pub old_branch_name: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct BranchDeleteParams {
    /// Repository owner. Optional if `directory` is provided.
    pub owner: Option<String>,
    /// Repository name. Optional if `directory` is provided.
    pub repo: Option<String>,
    /// Local directory to auto-detect owner/repo from .git/config.
    pub directory: Option<String>,
    /// Branch name to delete.
    pub branch: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct BranchProtectionListParams {
    /// Repository owner. Optional if `directory` is provided.
    pub owner: Option<String>,
    /// Repository name. Optional if `directory` is provided.
    pub repo: Option<String>,
    /// Local directory to auto-detect owner/repo from .git/config.
    pub directory: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct BranchProtectionCreateParams {
    /// Repository owner. Optional if `directory` is provided.
    pub owner: Option<String>,
    /// Repository name. Optional if `directory` is provided.
    pub repo: Option<String>,
    /// Local directory to auto-detect owner/repo from .git/config.
    pub directory: Option<String>,
    /// Branch name pattern to protect (e.g. "main", "release/*").
    pub branch_name: String,
    /// Allow direct pushes to this branch (bypassing pull requests).
    pub enable_push: Option<bool>,
    /// Block merging when reviews have been rejected.
    pub block_on_rejected_reviews: Option<bool>,
}

pub async fn branch_list(
    client: &GiteaClient,
    params: BranchListParams,
) -> Result<CallToolResult> {
    let (owner, repo) = resolve_owner_repo(&params.owner, &params.repo, &params.directory)?;
    let mut query: Vec<(&str, String)> = Vec::new();
    query.push(("page", params.page.unwrap_or(1).to_string()));
    query.push(("limit", params.limit.unwrap_or(20).min(50).to_string()));

    let query_refs: Vec<(&str, &str)> = query.iter().map(|(k, v)| (*k, v.as_str())).collect();
    let branches: Vec<serde_json::Value> = client
        .get_with_query(&format!("/repos/{owner}/{repo}/branches"), &query_refs)
        .await?;

    if branches.is_empty() {
        return Ok(CallToolResult::success(vec![Content::text(
            "No branches found.",
        )]));
    }

    let formatted: Vec<String> = branches.iter().map(|b| response::format_branch(b)).collect();

    Ok(CallToolResult::success(vec![Content::text(
        formatted.join("\n"),
    )]))
}

pub async fn branch_create(
    client: &GiteaClient,
    params: BranchCreateParams,
) -> Result<CallToolResult> {
    let (owner, repo) = resolve_owner_repo(&params.owner, &params.repo, &params.directory)?;
    let mut body = serde_json::json!({
        "new_branch_name": params.new_branch_name,
    });

    if let Some(old) = &params.old_branch_name {
        body["old_branch_name"] = serde_json::Value::String(old.clone());
    }

    let branch: serde_json::Value = client
        .post(&format!("/repos/{owner}/{repo}/branches"), &body)
        .await?;

    let name = branch
        .get("name")
        .and_then(|v| v.as_str())
        .unwrap_or(&params.new_branch_name);

    Ok(CallToolResult::success(vec![Content::text(format!(
        "Branch created: {name}"
    ))]))
}

pub async fn branch_delete(
    client: &GiteaClient,
    params: BranchDeleteParams,
) -> Result<CallToolResult> {
    let (owner, repo) = resolve_owner_repo(&params.owner, &params.repo, &params.directory)?;
    client
        .delete(&format!("/repos/{owner}/{repo}/branches/{}", params.branch))
        .await?;

    Ok(CallToolResult::success(vec![Content::text(format!(
        "Branch deleted: {}",
        params.branch
    ))]))
}

pub async fn branch_protection_list(
    client: &GiteaClient,
    params: BranchProtectionListParams,
) -> Result<CallToolResult> {
    let (owner, repo) = resolve_owner_repo(&params.owner, &params.repo, &params.directory)?;
    let rules: Vec<serde_json::Value> = client
        .get(&format!("/repos/{owner}/{repo}/branch_protections"))
        .await?;

    if rules.is_empty() {
        return Ok(CallToolResult::success(vec![Content::text(
            "No branch protection rules found.",
        )]));
    }

    let formatted: Vec<String> = rules
        .iter()
        .map(|r| {
            let name = r
                .get("branch_name")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown");
            let push = r
                .get("enable_push")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            format!("- {name} (push: {push})")
        })
        .collect();

    Ok(CallToolResult::success(vec![Content::text(
        formatted.join("\n"),
    )]))
}

pub async fn branch_protection_create(
    client: &GiteaClient,
    params: BranchProtectionCreateParams,
) -> Result<CallToolResult> {
    let (owner, repo) = resolve_owner_repo(&params.owner, &params.repo, &params.directory)?;
    let mut body = serde_json::json!({
        "branch_name": params.branch_name,
    });

    if let Some(push) = params.enable_push {
        body["enable_push"] = serde_json::Value::Bool(push);
    }
    if let Some(block) = params.block_on_rejected_reviews {
        body["block_on_rejected_reviews"] = serde_json::Value::Bool(block);
    }

    let _rule: serde_json::Value = client
        .post(
            &format!("/repos/{owner}/{repo}/branch_protections"),
            &body,
        )
        .await?;

    Ok(CallToolResult::success(vec![Content::text(format!(
        "Branch protection created for: {}",
        params.branch_name
    ))]))
}
