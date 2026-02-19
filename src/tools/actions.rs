use rmcp::model::{CallToolResult, Content};
use schemars::JsonSchema;
use serde::Deserialize;

use crate::client::GiteaClient;
use crate::error::Result;
use crate::server::resolve_owner_repo;

#[derive(Debug, Deserialize, JsonSchema)]
pub struct ActionsWorkflowListParams {
    /// Repository owner. Optional if `directory` is provided.
    pub owner: Option<String>,
    /// Repository name. Optional if `directory` is provided.
    pub repo: Option<String>,
    /// Local directory to auto-detect owner/repo from .git/config.
    pub directory: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct ActionsRunListParams {
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
pub struct ActionsRunGetParams {
    /// Repository owner. Optional if `directory` is provided.
    pub owner: Option<String>,
    /// Repository name. Optional if `directory` is provided.
    pub repo: Option<String>,
    /// Local directory to auto-detect owner/repo from .git/config.
    pub directory: Option<String>,
    /// Workflow run ID.
    pub run_id: i64,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct ActionsJobLogsParams {
    /// Repository owner. Optional if `directory` is provided.
    pub owner: Option<String>,
    /// Repository name. Optional if `directory` is provided.
    pub repo: Option<String>,
    /// Local directory to auto-detect owner/repo from .git/config.
    pub directory: Option<String>,
    /// Job ID (from the workflow run details in actions_run_get).
    pub job_id: i64,
}

pub async fn actions_workflow_list(
    client: &GiteaClient,
    params: ActionsWorkflowListParams,
) -> Result<CallToolResult> {
    let (owner, repo) = resolve_owner_repo(&params.owner, &params.repo, &params.directory)?;

    // Gitea's Actions API: list workflow files by reading .gitea/workflows or .github/workflows
    // Try the action tasks endpoint instead
    let result: serde_json::Value = client
        .get(&format!("/repos/{owner}/{repo}/actions/tasks"))
        .await
        .unwrap_or(serde_json::json!({"workflow_runs": []}));

    let workflows = result
        .get("workflow_runs")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();

    if workflows.is_empty() {
        // Fallback: list files in .gitea/workflows directory
        let files: std::result::Result<Vec<serde_json::Value>, _> = client
            .get(&format!(
                "/repos/{owner}/{repo}/contents/.gitea/workflows"
            ))
            .await;

        match files {
            Ok(entries) if !entries.is_empty() => {
                let formatted: Vec<String> = entries
                    .iter()
                    .map(|e| {
                        let name =
                            e.get("name").and_then(|v| v.as_str()).unwrap_or("?");
                        format!("- {name}")
                    })
                    .collect();
                return Ok(CallToolResult::success(vec![Content::text(format!(
                    "Workflow files:\n{}",
                    formatted.join("\n")
                ))]));
            }
            _ => {
                // Try .github/workflows
                let files2: std::result::Result<Vec<serde_json::Value>, _> = client
                    .get(&format!(
                        "/repos/{owner}/{repo}/contents/.github/workflows"
                    ))
                    .await;

                match files2 {
                    Ok(entries) if !entries.is_empty() => {
                        let formatted: Vec<String> = entries
                            .iter()
                            .map(|e| {
                                let name = e
                                    .get("name")
                                    .and_then(|v| v.as_str())
                                    .unwrap_or("?");
                                format!("- {name}")
                            })
                            .collect();
                        return Ok(CallToolResult::success(vec![Content::text(
                            format!(
                                "Workflow files:\n{}",
                                formatted.join("\n")
                            ),
                        )]));
                    }
                    _ => {
                        return Ok(CallToolResult::success(vec![Content::text(
                            "No workflows found.",
                        )]));
                    }
                }
            }
        }
    }

    let formatted: Vec<String> = workflows
        .iter()
        .map(|w| {
            let name = w.get("name").and_then(|v| v.as_str()).unwrap_or("?");
            let status = w
                .get("status")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown");
            format!("- {name} ({status})")
        })
        .collect();

    Ok(CallToolResult::success(vec![Content::text(
        formatted.join("\n"),
    )]))
}

pub async fn actions_run_list(
    client: &GiteaClient,
    params: ActionsRunListParams,
) -> Result<CallToolResult> {
    let (owner, repo) = resolve_owner_repo(&params.owner, &params.repo, &params.directory)?;
    let mut query: Vec<(&str, String)> = Vec::new();
    query.push(("page", params.page.unwrap_or(1).to_string()));
    query.push(("limit", params.limit.unwrap_or(20).min(50).to_string()));

    let query_refs: Vec<(&str, &str)> = query.iter().map(|(k, v)| (*k, v.as_str())).collect();
    let result: serde_json::Value = client
        .get_with_query(
            &format!("/repos/{owner}/{repo}/actions/runs"),
            &query_refs,
        )
        .await?;

    let runs = result
        .get("workflow_runs")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();

    if runs.is_empty() {
        return Ok(CallToolResult::success(vec![Content::text(
            "No workflow runs found.",
        )]));
    }

    let formatted: Vec<String> = runs
        .iter()
        .map(|r| {
            let run_num = r.get("run_number").and_then(|v| v.as_i64()).unwrap_or(0);
            let title = r
                .get("display_title")
                .and_then(|v| v.as_str())
                .or_else(|| r.get("name").and_then(|v| v.as_str()))
                .unwrap_or("(untitled)");
            let workflow = r
                .get("path")
                .and_then(|v| v.as_str())
                .map(|p| p.split('@').next().unwrap_or(p))
                .unwrap_or("?");
            let status = r
                .get("status")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown");
            let conclusion = r
                .get("conclusion")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let state = if !conclusion.is_empty() {
                conclusion
            } else {
                status
            };
            format!("- #{run_num} [{workflow}] {title} ({state})")
        })
        .collect();

    Ok(CallToolResult::success(vec![Content::text(
        formatted.join("\n"),
    )]))
}

pub async fn actions_run_get(
    client: &GiteaClient,
    params: ActionsRunGetParams,
) -> Result<CallToolResult> {
    let (owner, repo) = resolve_owner_repo(&params.owner, &params.repo, &params.directory)?;
    let run: serde_json::Value = client
        .get(&format!(
            "/repos/{owner}/{repo}/actions/runs/{}",
            params.run_id
        ))
        .await?;

    let mut parts = Vec::new();

    let run_num = run.get("run_number").and_then(|v| v.as_i64()).unwrap_or(0);
    let title = run
        .get("display_title")
        .and_then(|v| v.as_str())
        .or_else(|| run.get("name").and_then(|v| v.as_str()))
        .unwrap_or("(untitled)");
    let status = run
        .get("status")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown");
    parts.push(format!("## Run #{run_num}: {title} [{status}]"));

    if let Some(conclusion) = run.get("conclusion").and_then(|v| v.as_str()) {
        if !conclusion.is_empty() {
            parts.push(format!("**Conclusion:** {conclusion}"));
        }
    }

    if let Some(workflow) = run.get("path").and_then(|v| v.as_str()) {
        let name = workflow.split('@').next().unwrap_or(workflow);
        parts.push(format!("**Workflow:** {name}"));
    }

    if let Some(event) = run.get("event").and_then(|v| v.as_str()) {
        parts.push(format!("**Event:** {event}"));
    }

    if let Some(branch) = run.get("head_branch").and_then(|v| v.as_str()) {
        parts.push(format!("**Branch:** {branch}"));
    }

    if let Some(actor) = run
        .get("actor")
        .and_then(|v| v.get("login"))
        .and_then(|v| v.as_str())
    {
        parts.push(format!("**Actor:** {actor}"));
    }

    if let Some(started) = run.get("started_at").and_then(|v| v.as_str()) {
        parts.push(format!("**Started:** {started}"));
    }

    if let Some(completed) = run.get("completed_at").and_then(|v| v.as_str()) {
        parts.push(format!("**Completed:** {completed}"));
    }

    Ok(CallToolResult::success(vec![Content::text(
        parts.join("\n"),
    )]))
}

pub async fn actions_job_logs(
    client: &GiteaClient,
    params: ActionsJobLogsParams,
) -> Result<CallToolResult> {
    let (owner, repo) = resolve_owner_repo(&params.owner, &params.repo, &params.directory)?;
    let logs = client
        .get_raw(&format!(
            "/repos/{owner}/{repo}/actions/jobs/{}/logs",
            params.job_id
        ))
        .await?;

    if logs.is_empty() {
        return Ok(CallToolResult::success(vec![Content::text(
            "No logs available.",
        )]));
    }

    Ok(CallToolResult::success(vec![Content::text(format!(
        "```\n{logs}\n```"
    ))]))
}
