use rmcp::model::{CallToolResult, Content};
use schemars::JsonSchema;
use serde::Deserialize;

use crate::client::GiteaClient;
use crate::error::Result;

#[derive(Debug, Deserialize, JsonSchema)]
pub struct OrgListParams {}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct OrgGetParams {
    /// Organization name.
    pub org: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct OrgTeamsParams {
    /// Organization name.
    pub org: String,
}

pub async fn org_list(client: &GiteaClient) -> Result<CallToolResult> {
    let orgs: Vec<serde_json::Value> = client.get("/user/orgs").await?;

    if orgs.is_empty() {
        return Ok(CallToolResult::success(vec![Content::text(
            "No organizations found.",
        )]));
    }

    let formatted: Vec<String> = orgs
        .iter()
        .map(|o| {
            let name = o.get("name").and_then(|v| v.as_str()).unwrap_or("?");
            let full_name = o
                .get("full_name")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            if full_name.is_empty() || full_name == name {
                format!("- {name}")
            } else {
                format!("- {name} ({full_name})")
            }
        })
        .collect();

    Ok(CallToolResult::success(vec![Content::text(
        formatted.join("\n"),
    )]))
}

pub async fn org_get(client: &GiteaClient, params: OrgGetParams) -> Result<CallToolResult> {
    let org: serde_json::Value = client.get(&format!("/orgs/{}", params.org)).await?;

    let mut parts = Vec::new();
    let name = org.get("name").and_then(|v| v.as_str()).unwrap_or("?");
    parts.push(format!("## {name}"));

    if let Some(full_name) = org.get("full_name").and_then(|v| v.as_str()) {
        if !full_name.is_empty() && full_name != name {
            parts.push(format!("**Full name:** {full_name}"));
        }
    }

    if let Some(desc) = org.get("description").and_then(|v| v.as_str()) {
        if !desc.is_empty() {
            parts.push(format!("**Description:** {desc}"));
        }
    }

    if let Some(location) = org.get("location").and_then(|v| v.as_str()) {
        if !location.is_empty() {
            parts.push(format!("**Location:** {location}"));
        }
    }

    if let Some(website) = org.get("website").and_then(|v| v.as_str()) {
        if !website.is_empty() {
            parts.push(format!("**Website:** {website}"));
        }
    }

    Ok(CallToolResult::success(vec![Content::text(
        parts.join("\n"),
    )]))
}

pub async fn org_teams(client: &GiteaClient, params: OrgTeamsParams) -> Result<CallToolResult> {
    let teams: Vec<serde_json::Value> = client
        .get(&format!("/orgs/{}/teams", params.org))
        .await?;

    if teams.is_empty() {
        return Ok(CallToolResult::success(vec![Content::text(
            "No teams found.",
        )]));
    }

    let formatted: Vec<String> = teams
        .iter()
        .map(|t| {
            let name = t.get("name").and_then(|v| v.as_str()).unwrap_or("?");
            let id = t.get("id").and_then(|v| v.as_i64()).unwrap_or(0);
            let permission = t
                .get("permission")
                .and_then(|v| v.as_str())
                .unwrap_or("none");
            format!("- {name} (id: {id}, permission: {permission})")
        })
        .collect();

    Ok(CallToolResult::success(vec![Content::text(
        formatted.join("\n"),
    )]))
}
