use rmcp::handler::server::router::tool::ToolRouter;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::*;
use rmcp::{tool, tool_handler, tool_router, ServerHandler};

use crate::client::GiteaClient;
use crate::config::Config;
use crate::error::GiteaError;
use crate::repo_resolver;

// Tool parameter types
use crate::tools::actions::{
    ActionsJobLogsParams, ActionsRunGetParams, ActionsRunListParams, ActionsWorkflowListParams,
};
use crate::tools::branches::{
    BranchCreateParams, BranchDeleteParams, BranchListParams, BranchProtectionCreateParams,
    BranchProtectionListParams,
};
use crate::tools::commits::{CommitCompareParams, CommitDiffParams, CommitGetParams, CommitListParams};
use crate::tools::files::{
    FileCreateParams, FileDeleteParams, FileListParams, FileReadParams, FileUpdateParams,
    TreeGetParams,
};
use crate::tools::issue_comments::{IssueCommentCreateParams, IssueCommentListParams};
use crate::tools::issues::{IssueCreateParams, IssueEditParams, IssueGetParams, IssueListParams};
use crate::tools::labels::{LabelCreateParams, LabelEditParams, LabelListParams};
use crate::tools::milestones::{MilestoneCreateParams, MilestoneGetParams, MilestoneListParams};
use crate::tools::notifications::{NotificationListParams, NotificationMarkReadParams};
use crate::tools::orgs::{OrgGetParams, OrgListParams, OrgTeamsParams};
use crate::tools::pull_files::{PrDiffParams, PrFilesParams};
use crate::tools::pull_reviews::{PrReviewCreateParams, PrReviewListParams};
use crate::tools::pulls::{PrCreateParams, PrEditParams, PrGetParams, PrListParams, PrMergeParams};
use crate::tools::releases::{ReleaseCreateParams, ReleaseGetParams, ReleaseListParams};
use crate::tools::repo::{RepoGetParams, RepoSearchParams};
use crate::tools::tags::{TagCreateParams, TagListParams};
use crate::tools::users::{UserGetMeParams, UserGetParams};
use crate::tools::wiki::{WikiCreateParams, WikiGetParams, WikiListParams};

/// The gitx-mcp server. Holds the HTTP client and routes all 57 tools.
#[derive(Debug, Clone)]
pub struct GitxMcp {
    client: GiteaClient,
    tool_router: ToolRouter<Self>,
}

/// Resolve owner/repo from tool params — either explicit or from directory auto-detection.
pub fn resolve_owner_repo(
    owner: &Option<String>,
    repo: &Option<String>,
    directory: &Option<String>,
) -> std::result::Result<(String, String), GiteaError> {
    match (owner, repo) {
        (Some(o), Some(r)) if !o.is_empty() && !r.is_empty() => Ok((o.clone(), r.clone())),
        _ => {
            let dir = directory.as_deref().unwrap_or(".");
            let info = repo_resolver::resolve_repo(dir)?;
            Ok((info.owner, info.repo))
        }
    }
}

/// Helper to convert our Result<CallToolResult> to the ErrorData variant.
fn map_err(r: crate::error::Result<CallToolResult>) -> Result<CallToolResult, ErrorData> {
    r.map_err(ErrorData::from)
}

#[tool_router]
impl GitxMcp {
    pub fn new(config: Config) -> std::result::Result<Self, GiteaError> {
        let client = GiteaClient::new(&config)?;
        Ok(Self {
            client,
            tool_router: Self::tool_router(),
        })
    }

    // ── Issues ──────────────────────────────────────────────────────

    #[tool(description = "Use this when you need to list issues in a repository. Returns issue titles, numbers, states, and labels. Supports filtering by state (open/closed) and labels.")]
    async fn issue_list(&self, Parameters(p): Parameters<IssueListParams>) -> Result<CallToolResult, ErrorData> {
        map_err(crate::tools::issues::issue_list(&self.client, p).await)
    }

    #[tool(description = "Use this when you need to get the full details of a specific issue including its body, labels, assignees, and milestone. Requires the issue number.")]
    async fn issue_get(&self, Parameters(p): Parameters<IssueGetParams>) -> Result<CallToolResult, ErrorData> {
        map_err(crate::tools::issues::issue_get(&self.client, p).await)
    }

    #[tool(description = "Use this when you need to create a new issue in a repository. Provide a title and optionally a body, labels, milestone, and assignees.")]
    async fn issue_create(&self, Parameters(p): Parameters<IssueCreateParams>) -> Result<CallToolResult, ErrorData> {
        map_err(crate::tools::issues::issue_create(&self.client, p).await)
    }

    #[tool(description = "Use this when you need to update an existing issue — change its title, body, state (open/closed), labels, assignees, or milestone.")]
    async fn issue_edit(&self, Parameters(p): Parameters<IssueEditParams>) -> Result<CallToolResult, ErrorData> {
        map_err(crate::tools::issues::issue_edit(&self.client, p).await)
    }

    // ── Issue Comments ──────────────────────────────────────────────

    #[tool(description = "Use this when you need to list all comments on a specific issue. Returns comment authors, dates, and bodies. Returns a descriptive message if no comments exist.")]
    async fn issue_comment_list(&self, Parameters(p): Parameters<IssueCommentListParams>) -> Result<CallToolResult, ErrorData> {
        map_err(crate::tools::issue_comments::issue_comment_list(&self.client, p).await)
    }

    #[tool(description = "Use this when you need to add a comment to an existing issue. Provide the issue number and comment body.")]
    async fn issue_comment_create(&self, Parameters(p): Parameters<IssueCommentCreateParams>) -> Result<CallToolResult, ErrorData> {
        map_err(crate::tools::issue_comments::issue_comment_create(&self.client, p).await)
    }

    // ── Pull Requests ───────────────────────────────────────────────

    #[tool(description = "Use this when you need to list pull requests in a repository. Supports filtering by state (open/closed/all).")]
    async fn pr_list(&self, Parameters(p): Parameters<PrListParams>) -> Result<CallToolResult, ErrorData> {
        map_err(crate::tools::pulls::pr_list(&self.client, p).await)
    }

    #[tool(description = "Use this when you need to get full details of a specific pull request including branches, mergeable status, body, labels, and assignees.")]
    async fn pr_get(&self, Parameters(p): Parameters<PrGetParams>) -> Result<CallToolResult, ErrorData> {
        map_err(crate::tools::pulls::pr_get(&self.client, p).await)
    }

    #[tool(description = "Use this when you need to create a new pull request. Provide head branch, base branch, title, and optionally a body.")]
    async fn pr_create(&self, Parameters(p): Parameters<PrCreateParams>) -> Result<CallToolResult, ErrorData> {
        map_err(crate::tools::pulls::pr_create(&self.client, p).await)
    }

    #[tool(description = "Use this when you need to update a pull request — change its title, body, state, labels, or assignees.")]
    async fn pr_edit(&self, Parameters(p): Parameters<PrEditParams>) -> Result<CallToolResult, ErrorData> {
        map_err(crate::tools::pulls::pr_edit(&self.client, p).await)
    }

    #[tool(description = "Use this when you need to merge a pull request. Supports merge, rebase, and squash strategies.")]
    async fn pr_merge(&self, Parameters(p): Parameters<PrMergeParams>) -> Result<CallToolResult, ErrorData> {
        map_err(crate::tools::pulls::pr_merge(&self.client, p).await)
    }

    // ── Pull Request Reviews ────────────────────────────────────────

    #[tool(description = "Use this when you need to list reviews on a pull request. Shows reviewer, state (approved/changes_requested), and body.")]
    async fn pr_review_list(&self, Parameters(p): Parameters<PrReviewListParams>) -> Result<CallToolResult, ErrorData> {
        map_err(crate::tools::pull_reviews::pr_review_list(&self.client, p).await)
    }

    #[tool(description = "Use this when you need to submit a review on a pull request. Choose approve, request_changes, or comment as the review type.")]
    async fn pr_review_create(&self, Parameters(p): Parameters<PrReviewCreateParams>) -> Result<CallToolResult, ErrorData> {
        map_err(crate::tools::pull_reviews::pr_review_create(&self.client, p).await)
    }

    // ── Pull Request Files ──────────────────────────────────────────

    #[tool(description = "Use this when you need to see which files were changed in a pull request with their status (added/modified/deleted) and diff stats.")]
    async fn pr_files(&self, Parameters(p): Parameters<PrFilesParams>) -> Result<CallToolResult, ErrorData> {
        map_err(crate::tools::pull_files::pr_files(&self.client, p).await)
    }

    #[tool(description = "Use this when you need to see the raw unified diff of all changes in a pull request.")]
    async fn pr_diff(&self, Parameters(p): Parameters<PrDiffParams>) -> Result<CallToolResult, ErrorData> {
        map_err(crate::tools::pull_files::pr_diff(&self.client, p).await)
    }

    // ── Files ───────────────────────────────────────────────────────

    #[tool(description = "Use this when you need to read the content of a file from the repository at a specific ref (branch, tag, or commit SHA). Returns the decoded file content.")]
    async fn file_read(&self, Parameters(p): Parameters<FileReadParams>) -> Result<CallToolResult, ErrorData> {
        map_err(crate::tools::files::file_read(&self.client, p).await)
    }

    #[tool(description = "Use this when you need to list files and directories at a path in the repository. Returns names and types (file/dir).")]
    async fn file_list(&self, Parameters(p): Parameters<FileListParams>) -> Result<CallToolResult, ErrorData> {
        map_err(crate::tools::files::file_list(&self.client, p).await)
    }

    #[tool(description = "Use this when you need to create a new file in the repository. Provide the file path, content, and a commit message. Creates a commit.")]
    async fn file_create(&self, Parameters(p): Parameters<FileCreateParams>) -> Result<CallToolResult, ErrorData> {
        map_err(crate::tools::files::file_create(&self.client, p).await)
    }

    #[tool(description = "Use this when you need to update an existing file in the repository. Provide the file path, new content, SHA of the current file, and a commit message.")]
    async fn file_update(&self, Parameters(p): Parameters<FileUpdateParams>) -> Result<CallToolResult, ErrorData> {
        map_err(crate::tools::files::file_update(&self.client, p).await)
    }

    #[tool(description = "Use this when you need to delete a file from the repository. Provide the file path, SHA of the current file, and a commit message.")]
    async fn file_delete(&self, Parameters(p): Parameters<FileDeleteParams>) -> Result<CallToolResult, ErrorData> {
        map_err(crate::tools::files::file_delete(&self.client, p).await)
    }

    #[tool(description = "Use this when you need to get the full file tree of a repository recursively. Returns all file paths in the repository at a given ref.")]
    async fn tree_get(&self, Parameters(p): Parameters<TreeGetParams>) -> Result<CallToolResult, ErrorData> {
        map_err(crate::tools::files::tree_get(&self.client, p).await)
    }

    // ── Branches ────────────────────────────────────────────────────

    #[tool(description = "Use this when you need to list all branches in a repository. Shows branch names, latest commit SHA, and protection status.")]
    async fn branch_list(&self, Parameters(p): Parameters<BranchListParams>) -> Result<CallToolResult, ErrorData> {
        map_err(crate::tools::branches::branch_list(&self.client, p).await)
    }

    #[tool(description = "Use this when you need to create a new branch from an existing branch or commit SHA.")]
    async fn branch_create(&self, Parameters(p): Parameters<BranchCreateParams>) -> Result<CallToolResult, ErrorData> {
        map_err(crate::tools::branches::branch_create(&self.client, p).await)
    }

    #[tool(description = "Use this when you need to delete a branch from the repository.")]
    async fn branch_delete(&self, Parameters(p): Parameters<BranchDeleteParams>) -> Result<CallToolResult, ErrorData> {
        map_err(crate::tools::branches::branch_delete(&self.client, p).await)
    }

    #[tool(description = "Use this when you need to list branch protection rules for a repository.")]
    async fn branch_protection_list(&self, Parameters(p): Parameters<BranchProtectionListParams>) -> Result<CallToolResult, ErrorData> {
        map_err(crate::tools::branches::branch_protection_list(&self.client, p).await)
    }

    #[tool(description = "Use this when you need to create a branch protection rule to prevent force pushes, require reviews, or restrict who can push.")]
    async fn branch_protection_create(&self, Parameters(p): Parameters<BranchProtectionCreateParams>) -> Result<CallToolResult, ErrorData> {
        map_err(crate::tools::branches::branch_protection_create(&self.client, p).await)
    }

    // ── Commits ─────────────────────────────────────────────────────

    #[tool(description = "Use this when you need to list commits in a repository, optionally filtered by branch or file path.")]
    async fn commit_list(&self, Parameters(p): Parameters<CommitListParams>) -> Result<CallToolResult, ErrorData> {
        map_err(crate::tools::commits::commit_list(&self.client, p).await)
    }

    #[tool(description = "Use this when you need to get the full details of a specific commit by its SHA, including diff stats and parent commits.")]
    async fn commit_get(&self, Parameters(p): Parameters<CommitGetParams>) -> Result<CallToolResult, ErrorData> {
        map_err(crate::tools::commits::commit_get(&self.client, p).await)
    }

    #[tool(description = "Use this when you need to get the raw unified diff of a specific commit.")]
    async fn commit_diff(&self, Parameters(p): Parameters<CommitDiffParams>) -> Result<CallToolResult, ErrorData> {
        map_err(crate::tools::commits::commit_diff(&self.client, p).await)
    }

    #[tool(description = "Use this when you need to compare two refs (branches, tags, or commit SHAs) and see the commits and diff between them.")]
    async fn commit_compare(&self, Parameters(p): Parameters<CommitCompareParams>) -> Result<CallToolResult, ErrorData> {
        map_err(crate::tools::commits::commit_compare(&self.client, p).await)
    }

    // ── Labels ──────────────────────────────────────────────────────

    #[tool(description = "Use this when you need to list all labels available in a repository.")]
    async fn label_list(&self, Parameters(p): Parameters<LabelListParams>) -> Result<CallToolResult, ErrorData> {
        map_err(crate::tools::labels::label_list(&self.client, p).await)
    }

    #[tool(description = "Use this when you need to create a new label in a repository. Provide a name and color.")]
    async fn label_create(&self, Parameters(p): Parameters<LabelCreateParams>) -> Result<CallToolResult, ErrorData> {
        map_err(crate::tools::labels::label_create(&self.client, p).await)
    }

    #[tool(description = "Use this when you need to update an existing label's name, color, or description.")]
    async fn label_edit(&self, Parameters(p): Parameters<LabelEditParams>) -> Result<CallToolResult, ErrorData> {
        map_err(crate::tools::labels::label_edit(&self.client, p).await)
    }

    // ── Milestones ──────────────────────────────────────────────────

    #[tool(description = "Use this when you need to list milestones in a repository, optionally filtered by state (open/closed).")]
    async fn milestone_list(&self, Parameters(p): Parameters<MilestoneListParams>) -> Result<CallToolResult, ErrorData> {
        map_err(crate::tools::milestones::milestone_list(&self.client, p).await)
    }

    #[tool(description = "Use this when you need to get full details of a specific milestone by its ID.")]
    async fn milestone_get(&self, Parameters(p): Parameters<MilestoneGetParams>) -> Result<CallToolResult, ErrorData> {
        map_err(crate::tools::milestones::milestone_get(&self.client, p).await)
    }

    #[tool(description = "Use this when you need to create a new milestone in a repository.")]
    async fn milestone_create(&self, Parameters(p): Parameters<MilestoneCreateParams>) -> Result<CallToolResult, ErrorData> {
        map_err(crate::tools::milestones::milestone_create(&self.client, p).await)
    }

    // ── Notifications ───────────────────────────────────────────────

    #[tool(description = "Use this when you need to list your unread notifications. Shows notification subjects and reasons.")]
    async fn notification_list(&self, Parameters(p): Parameters<NotificationListParams>) -> Result<CallToolResult, ErrorData> {
        map_err(crate::tools::notifications::notification_list(&self.client, p).await)
    }

    #[tool(description = "Use this when you need to mark notifications as read, either all or a specific notification by ID.")]
    async fn notification_mark_read(&self, Parameters(p): Parameters<NotificationMarkReadParams>) -> Result<CallToolResult, ErrorData> {
        map_err(crate::tools::notifications::notification_mark_read(&self.client, p).await)
    }

    // ── Releases ────────────────────────────────────────────────────

    #[tool(description = "Use this when you need to list releases in a repository.")]
    async fn release_list(&self, Parameters(p): Parameters<ReleaseListParams>) -> Result<CallToolResult, ErrorData> {
        map_err(crate::tools::releases::release_list(&self.client, p).await)
    }

    #[tool(description = "Use this when you need to get full details of a specific release by its ID.")]
    async fn release_get(&self, Parameters(p): Parameters<ReleaseGetParams>) -> Result<CallToolResult, ErrorData> {
        map_err(crate::tools::releases::release_get(&self.client, p).await)
    }

    #[tool(description = "Use this when you need to create a new release with a tag, title, and release notes.")]
    async fn release_create(&self, Parameters(p): Parameters<ReleaseCreateParams>) -> Result<CallToolResult, ErrorData> {
        map_err(crate::tools::releases::release_create(&self.client, p).await)
    }

    // ── Repository ──────────────────────────────────────────────────

    #[tool(description = "Use this when you need to get metadata about a repository including its description, default branch, stars, forks, and visibility.")]
    async fn repo_get(&self, Parameters(p): Parameters<RepoGetParams>) -> Result<CallToolResult, ErrorData> {
        map_err(crate::tools::repo::repo_get(&self.client, p).await)
    }

    #[tool(description = "Use this when you need to search for repositories by keyword across the Gitea instance.")]
    async fn repo_search(&self, Parameters(p): Parameters<RepoSearchParams>) -> Result<CallToolResult, ErrorData> {
        map_err(crate::tools::repo::repo_search(&self.client, p).await)
    }

    // ── Users ───────────────────────────────────────────────────────

    #[tool(description = "Use this when you need to get information about the currently authenticated user (yourself).")]
    async fn user_get_me(&self, Parameters(_p): Parameters<UserGetMeParams>) -> Result<CallToolResult, ErrorData> {
        map_err(crate::tools::users::user_get_me(&self.client).await)
    }

    #[tool(description = "Use this when you need to get public profile information about a specific user by their username.")]
    async fn user_get(&self, Parameters(p): Parameters<UserGetParams>) -> Result<CallToolResult, ErrorData> {
        map_err(crate::tools::users::user_get(&self.client, p).await)
    }

    // ── Tags ────────────────────────────────────────────────────────

    #[tool(description = "Use this when you need to list all tags in a repository.")]
    async fn tag_list(&self, Parameters(p): Parameters<TagListParams>) -> Result<CallToolResult, ErrorData> {
        map_err(crate::tools::tags::tag_list(&self.client, p).await)
    }

    #[tool(description = "Use this when you need to create a new tag pointing to a specific commit SHA or branch.")]
    async fn tag_create(&self, Parameters(p): Parameters<TagCreateParams>) -> Result<CallToolResult, ErrorData> {
        map_err(crate::tools::tags::tag_create(&self.client, p).await)
    }

    // ── Wiki ────────────────────────────────────────────────────────

    #[tool(description = "Use this when you need to list all wiki pages in a repository.")]
    async fn wiki_list(&self, Parameters(p): Parameters<WikiListParams>) -> Result<CallToolResult, ErrorData> {
        map_err(crate::tools::wiki::wiki_list(&self.client, p).await)
    }

    #[tool(description = "Use this when you need to read the content of a specific wiki page by its slug/title.")]
    async fn wiki_get(&self, Parameters(p): Parameters<WikiGetParams>) -> Result<CallToolResult, ErrorData> {
        map_err(crate::tools::wiki::wiki_get(&self.client, p).await)
    }

    #[tool(description = "Use this when you need to create a new wiki page with a title and markdown content.")]
    async fn wiki_create(&self, Parameters(p): Parameters<WikiCreateParams>) -> Result<CallToolResult, ErrorData> {
        map_err(crate::tools::wiki::wiki_create(&self.client, p).await)
    }

    // ── Organizations ───────────────────────────────────────────────

    #[tool(description = "Use this when you need to list organizations the authenticated user belongs to.")]
    async fn org_list(&self, Parameters(_p): Parameters<OrgListParams>) -> Result<CallToolResult, ErrorData> {
        map_err(crate::tools::orgs::org_list(&self.client).await)
    }

    #[tool(description = "Use this when you need to get details about a specific organization by its name.")]
    async fn org_get(&self, Parameters(p): Parameters<OrgGetParams>) -> Result<CallToolResult, ErrorData> {
        map_err(crate::tools::orgs::org_get(&self.client, p).await)
    }

    #[tool(description = "Use this when you need to list teams in an organization.")]
    async fn org_teams(&self, Parameters(p): Parameters<OrgTeamsParams>) -> Result<CallToolResult, ErrorData> {
        map_err(crate::tools::orgs::org_teams(&self.client, p).await)
    }

    // ── Actions / CI ────────────────────────────────────────────────

    #[tool(description = "Use this when you need to list CI/CD workflows (Actions) configured in a repository.")]
    async fn actions_workflow_list(&self, Parameters(p): Parameters<ActionsWorkflowListParams>) -> Result<CallToolResult, ErrorData> {
        map_err(crate::tools::actions::actions_workflow_list(&self.client, p).await)
    }

    #[tool(description = "Use this when you need to list workflow runs for a repository, optionally filtered by workflow.")]
    async fn actions_run_list(&self, Parameters(p): Parameters<ActionsRunListParams>) -> Result<CallToolResult, ErrorData> {
        map_err(crate::tools::actions::actions_run_list(&self.client, p).await)
    }

    #[tool(description = "Use this when you need to get details of a specific workflow run by its ID.")]
    async fn actions_run_get(&self, Parameters(p): Parameters<ActionsRunGetParams>) -> Result<CallToolResult, ErrorData> {
        map_err(crate::tools::actions::actions_run_get(&self.client, p).await)
    }

    #[tool(description = "Use this when you need to get the logs of a specific CI/CD job to debug failures.")]
    async fn actions_job_logs(&self, Parameters(p): Parameters<ActionsJobLogsParams>) -> Result<CallToolResult, ErrorData> {
        map_err(crate::tools::actions::actions_job_logs(&self.client, p).await)
    }
}

#[tool_handler]
impl ServerHandler for GitxMcp {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            protocol_version: ProtocolVersion::V_2024_11_05,
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            server_info: Implementation {
                name: "gitx-mcp".to_string(),
                version: env!("CARGO_PKG_VERSION").to_string(),
                title: None,
                description: None,
                icons: None,
                website_url: None,
            },
            instructions: Some(
                "Gitea/Forgejo MCP server with 57 tools covering issues, PRs, files, branches, \
                 commits, labels, milestones, releases, notifications, wiki, organizations, and \
                 CI/CD actions. Use owner+repo params or directory param to auto-detect repository."
                    .to_string(),
            ),
        }
    }
}
