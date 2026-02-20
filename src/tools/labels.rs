use rmcp::model::{CallToolResult, Content};
use schemars::JsonSchema;
use serde::Deserialize;

use crate::client::GitClient;
use crate::error::Result;
use crate::repo_resolver::RepoInfo;
use crate::server::resolve_owner_repo;

#[derive(Debug, Deserialize, JsonSchema)]
pub struct LabelListParams {
    /// Repository owner. Optional if `directory` is provided.
    pub owner: Option<String>,
    /// Repository name. Optional if `directory` is provided.
    pub repo: Option<String>,
    /// Local directory to auto-detect owner/repo from .git/config.
    pub directory: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct LabelCreateParams {
    /// Repository owner. Optional if `directory` is provided.
    pub owner: Option<String>,
    /// Repository name. Optional if `directory` is provided.
    pub repo: Option<String>,
    /// Local directory to auto-detect owner/repo from .git/config.
    pub directory: Option<String>,
    /// Label name.
    pub name: String,
    /// Label color as hex (e.g. "#ff0000" or "ff0000").
    pub color: String,
    /// Label description.
    pub description: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct LabelEditParams {
    /// Repository owner. Optional if `directory` is provided.
    pub owner: Option<String>,
    /// Repository name. Optional if `directory` is provided.
    pub repo: Option<String>,
    /// Local directory to auto-detect owner/repo from .git/config.
    pub directory: Option<String>,
    /// Label ID (from label_list).
    pub id: i64,
    /// New label name.
    pub name: Option<String>,
    /// New label color.
    pub color: Option<String>,
    /// New label description.
    pub description: Option<String>,
}

pub async fn label_list(client: &dyn GitClient, params: LabelListParams, default_repo: Option<&RepoInfo>) -> Result<CallToolResult> {
    let (owner, repo) = resolve_owner_repo(&params.owner, &params.repo, &params.directory, default_repo)?;
    let val = client
        .get_json(&format!("/repos/{owner}/{repo}/labels"))
        .await?;
    let labels = val.as_array().cloned().unwrap_or_default();

    if labels.is_empty() {
        return Ok(CallToolResult::success(vec![Content::text(
            "No labels found.",
        )]));
    }

    let formatted: Vec<String> = labels
        .iter()
        .map(|l| {
            let name = l.get("name").and_then(|v| v.as_str()).unwrap_or("?");
            let color = l.get("color").and_then(|v| v.as_str()).unwrap_or("000000");
            let id = l.get("id").and_then(|v| v.as_i64()).unwrap_or(0);
            let desc = l
                .get("description")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            if desc.is_empty() {
                format!("- {name} (#{color}) [id: {id}]")
            } else {
                format!("- {name} (#{color}) [id: {id}] - {desc}")
            }
        })
        .collect();

    Ok(CallToolResult::success(vec![Content::text(
        formatted.join("\n"),
    )]))
}

pub async fn label_create(
    client: &dyn GitClient,
    params: LabelCreateParams,
    default_repo: Option<&RepoInfo>,
) -> Result<CallToolResult> {
    let (owner, repo) = resolve_owner_repo(&params.owner, &params.repo, &params.directory, default_repo)?;
    let color = if params.color.starts_with('#') {
        params.color.clone()
    } else {
        format!("#{}", params.color)
    };

    let mut body = serde_json::json!({
        "name": params.name,
        "color": color,
    });

    if let Some(desc) = &params.description {
        body["description"] = serde_json::Value::String(desc.clone());
    }

    let label = client
        .post_json(&format!("/repos/{owner}/{repo}/labels"), &body)
        .await?;

    let name = label
        .get("name")
        .and_then(|v| v.as_str())
        .unwrap_or(&params.name);

    Ok(CallToolResult::success(vec![Content::text(format!(
        "Label created: {name}"
    ))]))
}

pub async fn label_edit(
    client: &dyn GitClient,
    params: LabelEditParams,
    default_repo: Option<&RepoInfo>,
) -> Result<CallToolResult> {
    let (owner, repo) = resolve_owner_repo(&params.owner, &params.repo, &params.directory, default_repo)?;
    let mut body = serde_json::json!({});

    if let Some(name) = &params.name {
        body["name"] = serde_json::Value::String(name.clone());
    }
    if let Some(color) = &params.color {
        let c = if color.starts_with('#') {
            color.clone()
        } else {
            format!("#{color}")
        };
        body["color"] = serde_json::Value::String(c);
    }
    if let Some(desc) = &params.description {
        body["description"] = serde_json::Value::String(desc.clone());
    }

    let label = client
        .patch_json(
            &format!("/repos/{owner}/{repo}/labels/{}", params.id),
            &body,
        )
        .await?;

    let name = label.get("name").and_then(|v| v.as_str()).unwrap_or("?");

    Ok(CallToolResult::success(vec![Content::text(format!(
        "Label updated: {name}"
    ))]))
}
