use rmcp::model::{CallToolResult, Content};
use schemars::JsonSchema;
use serde::Deserialize;

use crate::client::GiteaClient;
use crate::error::Result;

#[derive(Debug, Deserialize, JsonSchema)]
pub struct UserGetMeParams {}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct UserGetParams {
    /// Username to look up.
    pub username: String,
}

pub async fn user_get_me(client: &GiteaClient) -> Result<CallToolResult> {
    let user: serde_json::Value = client.get("/user").await?;

    let mut parts = Vec::new();
    let login = user
        .get("login")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown");
    parts.push(format!("**Username:** {login}"));

    if let Some(full_name) = user.get("full_name").and_then(|v| v.as_str()) {
        if !full_name.is_empty() {
            parts.push(format!("**Full name:** {full_name}"));
        }
    }

    if let Some(email) = user.get("email").and_then(|v| v.as_str()) {
        if !email.is_empty() {
            parts.push(format!("**Email:** {email}"));
        }
    }

    let admin = user
        .get("is_admin")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    if admin {
        parts.push("**Role:** admin".to_string());
    }

    Ok(CallToolResult::success(vec![Content::text(
        parts.join("\n"),
    )]))
}

pub async fn user_get(client: &GiteaClient, params: UserGetParams) -> Result<CallToolResult> {
    let user: serde_json::Value = client
        .get(&format!("/users/{}", params.username))
        .await?;

    let mut parts = Vec::new();
    let login = user
        .get("login")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown");
    parts.push(format!("**Username:** {login}"));

    if let Some(full_name) = user.get("full_name").and_then(|v| v.as_str()) {
        if !full_name.is_empty() {
            parts.push(format!("**Full name:** {full_name}"));
        }
    }

    if let Some(created) = user.get("created").and_then(|v| v.as_str()) {
        parts.push(format!("**Created:** {created}"));
    }

    Ok(CallToolResult::success(vec![Content::text(
        parts.join("\n"),
    )]))
}
