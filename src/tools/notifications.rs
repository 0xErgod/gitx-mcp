use rmcp::model::{CallToolResult, Content};
use schemars::JsonSchema;
use serde::Deserialize;

use crate::client::GiteaClient;
use crate::error::Result;

#[derive(Debug, Deserialize, JsonSchema)]
pub struct NotificationListParams {
    /// Filter by status: unread, read, or all. Defaults to unread.
    pub status: Option<String>,
    /// Page number (1-based). Defaults to 1.
    pub page: Option<i64>,
    /// Items per page (max 50). Defaults to 20.
    pub limit: Option<i64>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct NotificationMarkReadParams {
    /// Specific notification ID to mark as read. If omitted, marks all as read.
    pub id: Option<i64>,
}

pub async fn notification_list(
    client: &GiteaClient,
    params: NotificationListParams,
) -> Result<CallToolResult> {
    let mut query: Vec<(&str, String)> = Vec::new();

    if let Some(status) = &params.status {
        // Gitea API uses status-types parameter
        query.push(("status-types", status.clone()));
    }
    query.push(("page", params.page.unwrap_or(1).to_string()));
    query.push(("limit", params.limit.unwrap_or(20).min(50).to_string()));

    let query_refs: Vec<(&str, &str)> = query.iter().map(|(k, v)| (*k, v.as_str())).collect();
    let notifications: Vec<serde_json::Value> = client
        .get_with_query("/notifications", &query_refs)
        .await?;

    if notifications.is_empty() {
        return Ok(CallToolResult::success(vec![Content::text(
            "No notifications found.",
        )]));
    }

    let formatted: Vec<String> = notifications
        .iter()
        .map(|n| {
            let id = n.get("id").and_then(|v| v.as_i64()).unwrap_or(0);
            let subject_title = n
                .get("subject")
                .and_then(|v| v.get("title"))
                .and_then(|v| v.as_str())
                .unwrap_or("(no title)");
            let subject_type = n
                .get("subject")
                .and_then(|v| v.get("type"))
                .and_then(|v| v.as_str())
                .unwrap_or("unknown");
            let repo_name = n
                .get("repository")
                .and_then(|v| v.get("full_name"))
                .and_then(|v| v.as_str())
                .unwrap_or("unknown");
            let unread = n.get("unread").and_then(|v| v.as_bool()).unwrap_or(false);
            let status = if unread { "unread" } else { "read" };
            format!("- [{status}] #{id} {subject_type}: {subject_title} ({repo_name})")
        })
        .collect();

    Ok(CallToolResult::success(vec![Content::text(
        formatted.join("\n"),
    )]))
}

pub async fn notification_mark_read(
    client: &GiteaClient,
    params: NotificationMarkReadParams,
) -> Result<CallToolResult> {
    if let Some(id) = params.id {
        client
            .patch::<serde_json::Value>(
                &format!("/notifications/threads/{id}"),
                &serde_json::json!({}),
            )
            .await?;
        Ok(CallToolResult::success(vec![Content::text(format!(
            "Notification #{id} marked as read."
        ))]))
    } else {
        client
            .put::<serde_json::Value>("/notifications", &serde_json::json!({}))
            .await?;
        Ok(CallToolResult::success(vec![Content::text(
            "All notifications marked as read.",
        )]))
    }
}
