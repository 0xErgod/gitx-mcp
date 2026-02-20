use std::sync::Arc;

use rmcp::handler::server::router::tool::ToolRouter;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::*;
use rmcp::service::RequestContext;
use rmcp::{tool, tool_handler, tool_router, RoleServer, ServerHandler};

use crate::client::{GitClient, GiteaClient};
use crate::config::Config;
use crate::error::GitxError;
use crate::platform::Platform;
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

const RESOURCE_URI: &str = "repo://detected";

/// The gitx-mcp server. Holds the HTTP client and routes all 43 tools.
#[derive(Debug, Clone)]
pub struct GitxMcp {
    client: Arc<dyn GitClient>,
    tool_router: ToolRouter<Self>,
    detected_repo: Option<repo_resolver::RepoInfo>,
}

/// Resolve owner/repo from tool params — either explicit, from directory auto-detection,
/// or from the server's startup-detected default.
pub fn resolve_owner_repo(
    owner: &Option<String>,
    repo: &Option<String>,
    directory: &Option<String>,
    default_repo: Option<&repo_resolver::RepoInfo>,
) -> std::result::Result<(String, String), GitxError> {
    // 1. Explicit owner+repo
    match (owner, repo) {
        (Some(o), Some(r)) if !o.is_empty() && !r.is_empty() => return Ok((o.clone(), r.clone())),
        _ => {}
    }

    // 2. Explicit directory
    if let Some(dir) = directory.as_deref().filter(|d| !d.is_empty()) {
        let info = repo_resolver::resolve_repo(dir)?;
        return Ok((info.owner, info.repo));
    }

    // 3. Stored default from startup
    if let Some(info) = default_repo {
        return Ok((info.owner.clone(), info.repo.clone()));
    }

    // 4. Last resort — cwd detection
    let info = repo_resolver::resolve_repo(".")?;
    Ok((info.owner, info.repo))
}

/// Helper to convert our Result<CallToolResult> to the ErrorData variant.
fn map_err(r: crate::error::Result<CallToolResult>) -> Result<CallToolResult, ErrorData> {
    r.map_err(ErrorData::from)
}

#[tool_router]
impl GitxMcp {
    pub fn new(config: Config) -> std::result::Result<Self, GitxError> {
        let client: Arc<dyn GitClient> = match config.platform {
            Platform::Gitea => Arc::new(GiteaClient::new(&config)?),
            Platform::GitHub => Arc::new(crate::client::GitHubClient::new(&config)?),
        };

        let detected_repo = match repo_resolver::resolve_repo(".") {
            Ok(info) => {
                tracing::info!("Auto-detected repository: {}/{}", info.owner, info.repo);
                Some(info)
            }
            Err(e) => {
                tracing::debug!("No repository detected in cwd: {e}");
                None
            }
        };

        Ok(Self {
            client,
            tool_router: Self::tool_router(),
            detected_repo,
        })
    }

    // ── Issues ──────────────────────────────────────────────────────

    #[tool(description = "Use this when you need to list issues in a repository. Returns issue numbers, titles, states, and labels. Supports filtering by state (open/closed) and labels. Only returns issues (not pull requests). Use issue_get for full details of a specific issue.")]
    async fn issue_list(&self, Parameters(p): Parameters<IssueListParams>) -> Result<CallToolResult, ErrorData> {
        map_err(crate::tools::issues::issue_list(self.client.as_ref(), p, self.detected_repo.as_ref()).await)
    }

    #[tool(description = "Use this when you need to get the full details of a specific issue including its body, labels, assignees, and milestone. Requires the issue number. Returns number, title, state, body, labels, assignees, milestone, and timestamps. Use issue_comment_list to see comments on the issue.")]
    async fn issue_get(&self, Parameters(p): Parameters<IssueGetParams>) -> Result<CallToolResult, ErrorData> {
        map_err(crate::tools::issues::issue_get(self.client.as_ref(), p, self.detected_repo.as_ref()).await)
    }

    #[tool(description = "Use this when you need to create a new issue in a repository. Provide a title and optionally a body, labels, milestone, and assignees. On Gitea, labels and milestone require numeric IDs — use label_list and milestone_list to look them up first. On GitHub, labels are names (strings). Returns the created issue details. Fails with 404 if the repository is not found, or 403 if you lack permission.")]
    async fn issue_create(&self, Parameters(p): Parameters<IssueCreateParams>) -> Result<CallToolResult, ErrorData> {
        map_err(crate::tools::issues::issue_create(self.client.as_ref(), p, self.detected_repo.as_ref()).await)
    }

    #[tool(description = "Use this when you need to update an existing issue — change its title, body, state (open/closed), labels, assignees, or milestone. On Gitea, labels and milestone require numeric IDs — use label_list and milestone_list to look them up first. On GitHub, labels are names (strings). Labels and assignees replace existing values (not additive). Returns the updated issue details.")]
    async fn issue_edit(&self, Parameters(p): Parameters<IssueEditParams>) -> Result<CallToolResult, ErrorData> {
        map_err(crate::tools::issues::issue_edit(self.client.as_ref(), p, self.detected_repo.as_ref()).await)
    }

    // ── Issue Comments ──────────────────────────────────────────────

    #[tool(description = "Use this when you need to list all comments on a specific issue or pull request. Returns comment authors, dates, and bodies for each comment, or a message if no comments exist.")]
    async fn issue_comment_list(&self, Parameters(p): Parameters<IssueCommentListParams>) -> Result<CallToolResult, ErrorData> {
        map_err(crate::tools::issue_comments::issue_comment_list(self.client.as_ref(), p, self.detected_repo.as_ref()).await)
    }

    #[tool(description = "Use this when you need to add a comment to an existing issue or pull request. Provide the issue number and comment body in markdown. Returns the created comment with author and timestamp. Fails with 404 if the issue does not exist.")]
    async fn issue_comment_create(&self, Parameters(p): Parameters<IssueCommentCreateParams>) -> Result<CallToolResult, ErrorData> {
        map_err(crate::tools::issue_comments::issue_comment_create(self.client.as_ref(), p, self.detected_repo.as_ref()).await)
    }

    // ── Pull Requests ───────────────────────────────────────────────

    #[tool(description = "Use this when you need to list pull requests in a repository. Returns PR numbers, titles, states, and branch info. Supports filtering by state (open/closed/all, defaults to open). Use pr_get for full details of a specific PR.")]
    async fn pr_list(&self, Parameters(p): Parameters<PrListParams>) -> Result<CallToolResult, ErrorData> {
        map_err(crate::tools::pulls::pr_list(self.client.as_ref(), p, self.detected_repo.as_ref()).await)
    }

    #[tool(description = "Use this when you need to get full details of a specific pull request including branches, mergeable status, body, labels, and assignees. Returns number, title, state, head/base branches, mergeable status, body, labels, assignees, and timestamps. Check mergeable status here before calling pr_merge. Use pr_files for changed files or pr_diff for the full diff.")]
    async fn pr_get(&self, Parameters(p): Parameters<PrGetParams>) -> Result<CallToolResult, ErrorData> {
        map_err(crate::tools::pulls::pr_get(self.client.as_ref(), p, self.detected_repo.as_ref()).await)
    }

    #[tool(description = "Use this when you need to create a new pull request. Provide head branch (source), base branch (target), title, and optionally a body, labels, milestone, and assignees. On Gitea, labels require numeric IDs — use label_list to look them up first. On GitHub, labels are names (strings). The head branch must exist and have commits ahead of base. Returns the created PR details. Fails with 404 if branches don't exist, or 409 if a PR already exists for these branches.")]
    async fn pr_create(&self, Parameters(p): Parameters<PrCreateParams>) -> Result<CallToolResult, ErrorData> {
        map_err(crate::tools::pulls::pr_create(self.client.as_ref(), p, self.detected_repo.as_ref()).await)
    }

    #[tool(description = "Use this when you need to update a pull request — change its title, body, state (open/closed), labels, or assignees. On Gitea, labels require numeric IDs from label_list. On GitHub, labels are names (strings). Labels and assignees replace existing values. Returns the updated PR details.")]
    async fn pr_edit(&self, Parameters(p): Parameters<PrEditParams>) -> Result<CallToolResult, ErrorData> {
        map_err(crate::tools::pulls::pr_edit(self.client.as_ref(), p, self.detected_repo.as_ref()).await)
    }

    #[tool(description = "Use this when you need to merge a pull request. Supports merge, rebase, and squash strategies. Use pr_get first to verify the PR is mergeable. Fails with 405 if the PR is not mergeable (conflicts, missing reviews, etc.) or 404 if the PR does not exist.")]
    async fn pr_merge(&self, Parameters(p): Parameters<PrMergeParams>) -> Result<CallToolResult, ErrorData> {
        map_err(crate::tools::pulls::pr_merge(self.client.as_ref(), p, self.detected_repo.as_ref()).await)
    }

    // ── Pull Request Reviews ────────────────────────────────────────

    #[tool(description = "Use this when you need to list reviews on a pull request. Returns review ID, reviewer username, state (APPROVED/CHANGES_REQUESTED/COMMENT), and body for each review.")]
    async fn pr_review_list(&self, Parameters(p): Parameters<PrReviewListParams>) -> Result<CallToolResult, ErrorData> {
        map_err(crate::tools::pull_reviews::pr_review_list(self.client.as_ref(), p, self.detected_repo.as_ref()).await)
    }

    #[tool(description = "Use this when you need to submit a review on a pull request. Event must be one of: APPROVED, REQUEST_CHANGES, or COMMENT (uppercase). Returns the submitted review state. Fails with 404 if the PR does not exist, or 422 if the event type is invalid.")]
    async fn pr_review_create(&self, Parameters(p): Parameters<PrReviewCreateParams>) -> Result<CallToolResult, ErrorData> {
        map_err(crate::tools::pull_reviews::pr_review_create(self.client.as_ref(), p, self.detected_repo.as_ref()).await)
    }

    // ── Pull Request Files ──────────────────────────────────────────

    #[tool(description = "Use this when you need to see which files were changed in a pull request. Returns filename, status (added/modified/deleted), and diff stats (+additions/-deletions) for each file. For the full unified diff content, use pr_diff instead.")]
    async fn pr_files(&self, Parameters(p): Parameters<PrFilesParams>) -> Result<CallToolResult, ErrorData> {
        map_err(crate::tools::pull_files::pr_files(self.client.as_ref(), p, self.detected_repo.as_ref()).await)
    }

    #[tool(description = "Use this when you need to see the raw unified diff of all changes in a pull request. Returns the full diff in unified format. For a summary of changed files with stats, use pr_files instead.")]
    async fn pr_diff(&self, Parameters(p): Parameters<PrDiffParams>) -> Result<CallToolResult, ErrorData> {
        map_err(crate::tools::pull_files::pr_diff(self.client.as_ref(), p, self.detected_repo.as_ref()).await)
    }

    // ── Files ───────────────────────────────────────────────────────

    #[tool(description = "Use this when you need to read the content of a file from the repository at a specific ref (branch, tag, or commit SHA). Returns the file path, size, SHA, and decoded content. IMPORTANT: The returned SHA is required by file_update and file_delete — always call file_read first before updating or deleting a file. Fails with 404 if the file or ref does not exist.")]
    async fn file_read(&self, Parameters(p): Parameters<FileReadParams>) -> Result<CallToolResult, ErrorData> {
        map_err(crate::tools::files::file_read(self.client.as_ref(), p, self.detected_repo.as_ref()).await)
    }

    #[tool(description = "Use this when you need to list files and directories at a path in the repository. Returns names and types (file/dir) for each entry in the directory. This lists a single directory level — use tree_get for a full recursive listing of all files.")]
    async fn file_list(&self, Parameters(p): Parameters<FileListParams>) -> Result<CallToolResult, ErrorData> {
        map_err(crate::tools::files::file_list(self.client.as_ref(), p, self.detected_repo.as_ref()).await)
    }

    #[tool(description = "Use this when you need to create a new file in the repository. Provide the file path, content, and a commit message. Content is plain text (base64-encoding is handled automatically). Creates a commit. Returns the created file path. Fails with 422 if the file already exists (use file_update instead).")]
    async fn file_create(&self, Parameters(p): Parameters<FileCreateParams>) -> Result<CallToolResult, ErrorData> {
        map_err(crate::tools::files::file_create(self.client.as_ref(), p, self.detected_repo.as_ref()).await)
    }

    #[tool(description = "Use this when you need to update an existing file in the repository. Provide the file path, new content, SHA of the current file, and a commit message. You must call file_read first to get the current file SHA. Content is plain text (base64-encoding is handled automatically). Creates a commit. Fails with 409 if the SHA does not match (file was modified since you read it).")]
    async fn file_update(&self, Parameters(p): Parameters<FileUpdateParams>) -> Result<CallToolResult, ErrorData> {
        map_err(crate::tools::files::file_update(self.client.as_ref(), p, self.detected_repo.as_ref()).await)
    }

    #[tool(description = "Use this when you need to delete a file from the repository. Provide the file path, SHA of the current file, and a commit message. You must call file_read first to get the current file SHA. Creates a commit. Fails with 409 if the SHA does not match.")]
    async fn file_delete(&self, Parameters(p): Parameters<FileDeleteParams>) -> Result<CallToolResult, ErrorData> {
        map_err(crate::tools::files::file_delete(self.client.as_ref(), p, self.detected_repo.as_ref()).await)
    }

    #[tool(description = "Use this when you need to get the full file tree of a repository recursively. Returns all file and directory paths in the repository at a given ref. For listing a single directory level, use file_list instead.")]
    async fn tree_get(&self, Parameters(p): Parameters<TreeGetParams>) -> Result<CallToolResult, ErrorData> {
        map_err(crate::tools::files::tree_get(self.client.as_ref(), p, self.detected_repo.as_ref()).await)
    }

    // ── Branches ────────────────────────────────────────────────────

    #[tool(description = "Use this when you need to list all branches in a repository. Returns branch names, latest commit SHA, and protection status for each branch.")]
    async fn branch_list(&self, Parameters(p): Parameters<BranchListParams>) -> Result<CallToolResult, ErrorData> {
        map_err(crate::tools::branches::branch_list(self.client.as_ref(), p, self.detected_repo.as_ref()).await)
    }

    #[tool(description = "Use this when you need to create a new branch from an existing branch or commit SHA. Returns the created branch name. Fails with 409 if the branch name already exists, or 404 if the source branch does not exist.")]
    async fn branch_create(&self, Parameters(p): Parameters<BranchCreateParams>) -> Result<CallToolResult, ErrorData> {
        map_err(crate::tools::branches::branch_create(self.client.as_ref(), p, self.detected_repo.as_ref()).await)
    }

    #[tool(description = "Use this when you need to delete a branch from the repository. Fails with 403 if the branch is protected, or 404 if it does not exist.")]
    async fn branch_delete(&self, Parameters(p): Parameters<BranchDeleteParams>) -> Result<CallToolResult, ErrorData> {
        map_err(crate::tools::branches::branch_delete(self.client.as_ref(), p, self.detected_repo.as_ref()).await)
    }

    #[tool(description = "Use this when you need to list branch protection rules for a repository. Returns branch name patterns and their push/review settings for each rule.")]
    async fn branch_protection_list(&self, Parameters(p): Parameters<BranchProtectionListParams>) -> Result<CallToolResult, ErrorData> {
        map_err(crate::tools::branches::branch_protection_list(self.client.as_ref(), p, self.detected_repo.as_ref()).await)
    }

    #[tool(description = "Use this when you need to create a branch protection rule. Configure whether direct pushes are allowed and whether rejected reviews block merging. Supports glob patterns for branch names (e.g. 'main', 'release/*'). Fails with 422 if a rule for this pattern already exists.")]
    async fn branch_protection_create(&self, Parameters(p): Parameters<BranchProtectionCreateParams>) -> Result<CallToolResult, ErrorData> {
        map_err(crate::tools::branches::branch_protection_create(self.client.as_ref(), p, self.detected_repo.as_ref()).await)
    }

    // ── Commits ─────────────────────────────────────────────────────

    #[tool(description = "Use this when you need to list commits in a repository, optionally filtered by branch/tag or file path. Returns commit SHA, author, date, and message for each commit. Use commit_get for full details including diff stats.")]
    async fn commit_list(&self, Parameters(p): Parameters<CommitListParams>) -> Result<CallToolResult, ErrorData> {
        map_err(crate::tools::commits::commit_list(self.client.as_ref(), p, self.detected_repo.as_ref()).await)
    }

    #[tool(description = "Use this when you need to get the full details of a specific commit by its SHA, including message, author, diff stats, and parent commits. Use commit_diff for the full unified diff of the commit.")]
    async fn commit_get(&self, Parameters(p): Parameters<CommitGetParams>) -> Result<CallToolResult, ErrorData> {
        map_err(crate::tools::commits::commit_get(self.client.as_ref(), p, self.detected_repo.as_ref()).await)
    }

    #[tool(description = "Use this when you need to get the raw unified diff of a specific commit. Returns the diff in unified format. For comparing two different refs, use commit_compare instead.")]
    async fn commit_diff(&self, Parameters(p): Parameters<CommitDiffParams>) -> Result<CallToolResult, ErrorData> {
        map_err(crate::tools::commits::commit_diff(self.client.as_ref(), p, self.detected_repo.as_ref()).await)
    }

    #[tool(description = "Use this when you need to compare two refs (branches, tags, or commit SHAs). Returns the list of commits between them and the changed files with their status.")]
    async fn commit_compare(&self, Parameters(p): Parameters<CommitCompareParams>) -> Result<CallToolResult, ErrorData> {
        map_err(crate::tools::commits::commit_compare(self.client.as_ref(), p, self.detected_repo.as_ref()).await)
    }

    // ── Labels ──────────────────────────────────────────────────────

    #[tool(description = "Use this when you need to list all labels available in a repository. Returns label ID, name, color, and description for each label. Use the returned IDs (Gitea) or names (GitHub) when creating or editing issues and pull requests (issue_create, issue_edit, pr_create, pr_edit).")]
    async fn label_list(&self, Parameters(p): Parameters<LabelListParams>) -> Result<CallToolResult, ErrorData> {
        map_err(crate::tools::labels::label_list(self.client.as_ref(), p, self.detected_repo.as_ref()).await)
    }

    #[tool(description = "Use this when you need to create a new label in a repository. Provide a name and hex color. Returns the created label name. Fails with 422 if a label with the same name already exists.")]
    async fn label_create(&self, Parameters(p): Parameters<LabelCreateParams>) -> Result<CallToolResult, ErrorData> {
        map_err(crate::tools::labels::label_create(self.client.as_ref(), p, self.detected_repo.as_ref()).await)
    }

    #[tool(description = "Use this when you need to update an existing label's name, color, or description. Requires the label ID from label_list. Returns the updated label name.")]
    async fn label_edit(&self, Parameters(p): Parameters<LabelEditParams>) -> Result<CallToolResult, ErrorData> {
        map_err(crate::tools::labels::label_edit(self.client.as_ref(), p, self.detected_repo.as_ref()).await)
    }

    // ── Milestones ──────────────────────────────────────────────────

    #[tool(description = "Use this when you need to list milestones in a repository, optionally filtered by state (open/closed). Returns milestone ID, title, state, and open/closed issue counts. Use the returned IDs when creating or editing issues (issue_create, issue_edit).")]
    async fn milestone_list(&self, Parameters(p): Parameters<MilestoneListParams>) -> Result<CallToolResult, ErrorData> {
        map_err(crate::tools::milestones::milestone_list(self.client.as_ref(), p, self.detected_repo.as_ref()).await)
    }

    #[tool(description = "Use this when you need to get full details of a specific milestone. Requires the milestone ID from milestone_list. Returns title, description, due date, and issue counts.")]
    async fn milestone_get(&self, Parameters(p): Parameters<MilestoneGetParams>) -> Result<CallToolResult, ErrorData> {
        map_err(crate::tools::milestones::milestone_get(self.client.as_ref(), p, self.detected_repo.as_ref()).await)
    }

    #[tool(description = "Use this when you need to create a new milestone in a repository. Provide a title and optionally a description and due date. Returns the created milestone title.")]
    async fn milestone_create(&self, Parameters(p): Parameters<MilestoneCreateParams>) -> Result<CallToolResult, ErrorData> {
        map_err(crate::tools::milestones::milestone_create(self.client.as_ref(), p, self.detected_repo.as_ref()).await)
    }

    // ── Notifications ───────────────────────────────────────────────

    #[tool(description = "Use this when you need to list your notifications (not repository-scoped). Returns notification ID, status (read/unread), subject type, title, and repository for each notification. Use the returned IDs with notification_mark_read to mark specific notifications as read.")]
    async fn notification_list(&self, Parameters(p): Parameters<NotificationListParams>) -> Result<CallToolResult, ErrorData> {
        map_err(crate::tools::notifications::notification_list(self.client.as_ref(), p).await)
    }

    #[tool(description = "Use this when you need to mark notifications as read, either all at once or a specific notification by ID from notification_list.")]
    async fn notification_mark_read(&self, Parameters(p): Parameters<NotificationMarkReadParams>) -> Result<CallToolResult, ErrorData> {
        map_err(crate::tools::notifications::notification_mark_read(self.client.as_ref(), p).await)
    }

    // ── Releases ────────────────────────────────────────────────────

    #[tool(description = "Use this when you need to list releases in a repository. Returns release ID, tag name, title, and draft/prerelease flags for each release. Use release_get with the returned ID for full details.")]
    async fn release_list(&self, Parameters(p): Parameters<ReleaseListParams>) -> Result<CallToolResult, ErrorData> {
        map_err(crate::tools::releases::release_list(self.client.as_ref(), p, self.detected_repo.as_ref()).await)
    }

    #[tool(description = "Use this when you need to get full details of a specific release. Requires the release ID from release_list. Returns the full release object including tag, title, body, draft/prerelease status, and assets.")]
    async fn release_get(&self, Parameters(p): Parameters<ReleaseGetParams>) -> Result<CallToolResult, ErrorData> {
        map_err(crate::tools::releases::release_get(self.client.as_ref(), p, self.detected_repo.as_ref()).await)
    }

    #[tool(description = "Use this when you need to create a new release with a tag, title, and release notes. If the tag doesn't exist, it will be created pointing to target_commitish. For creating just a tag without a release, use tag_create instead. Returns the created release tag name.")]
    async fn release_create(&self, Parameters(p): Parameters<ReleaseCreateParams>) -> Result<CallToolResult, ErrorData> {
        map_err(crate::tools::releases::release_create(self.client.as_ref(), p, self.detected_repo.as_ref()).await)
    }

    // ── Repository ──────────────────────────────────────────────────

    #[tool(description = "Use this when you need to get metadata about a repository. Returns full name, description, default branch, stars, forks, visibility, and primary language.")]
    async fn repo_get(&self, Parameters(p): Parameters<RepoGetParams>) -> Result<CallToolResult, ErrorData> {
        map_err(crate::tools::repo::repo_get(self.client.as_ref(), p, self.detected_repo.as_ref()).await)
    }

    #[tool(description = "Use this when you need to search for repositories by keyword. Returns full name, description, and star count for each matching repository.")]
    async fn repo_search(&self, Parameters(p): Parameters<RepoSearchParams>) -> Result<CallToolResult, ErrorData> {
        map_err(crate::tools::repo::repo_search(self.client.as_ref(), p).await)
    }

    // ── Users ───────────────────────────────────────────────────────

    #[tool(description = "Use this when you need to get information about the currently authenticated user (yourself). Returns username, full name, email, and admin status.")]
    async fn user_get_me(&self, Parameters(_p): Parameters<UserGetMeParams>) -> Result<CallToolResult, ErrorData> {
        map_err(crate::tools::users::user_get_me(self.client.as_ref()).await)
    }

    #[tool(description = "Use this when you need to get public profile information about a specific user by their username. Returns username, full name, and account creation date.")]
    async fn user_get(&self, Parameters(p): Parameters<UserGetParams>) -> Result<CallToolResult, ErrorData> {
        map_err(crate::tools::users::user_get(self.client.as_ref(), p).await)
    }

    // ── Tags ────────────────────────────────────────────────────────

    #[tool(description = "Use this when you need to list all tags in a repository. Returns tag name and short commit SHA for each tag.")]
    async fn tag_list(&self, Parameters(p): Parameters<TagListParams>) -> Result<CallToolResult, ErrorData> {
        map_err(crate::tools::tags::tag_list(self.client.as_ref(), p, self.detected_repo.as_ref()).await)
    }

    #[tool(description = "Use this when you need to create a new tag pointing to a specific commit SHA or branch. For creating a release with release notes, use release_create instead. Returns the created tag name.")]
    async fn tag_create(&self, Parameters(p): Parameters<TagCreateParams>) -> Result<CallToolResult, ErrorData> {
        map_err(crate::tools::tags::tag_create(self.client.as_ref(), p, self.detected_repo.as_ref()).await)
    }

    // ── Wiki ────────────────────────────────────────────────────────

    #[tool(description = "Use this when you need to list all wiki pages in a repository. Returns title and slug for each page. Use the returned slug with wiki_get to read page content. Returns a message if the wiki is disabled for the repository. Note: Wiki CRUD is only available on Gitea/Forgejo; GitHub does not expose a wiki API.")]
    async fn wiki_list(&self, Parameters(p): Parameters<WikiListParams>) -> Result<CallToolResult, ErrorData> {
        map_err(crate::tools::wiki::wiki_list(self.client.as_ref(), p, self.detected_repo.as_ref()).await)
    }

    #[tool(description = "Use this when you need to read the content of a specific wiki page. Requires the page slug from wiki_list. Returns the page title and decoded markdown content. Note: Wiki CRUD is only available on Gitea/Forgejo; GitHub does not expose a wiki API.")]
    async fn wiki_get(&self, Parameters(p): Parameters<WikiGetParams>) -> Result<CallToolResult, ErrorData> {
        map_err(crate::tools::wiki::wiki_get(self.client.as_ref(), p, self.detected_repo.as_ref()).await)
    }

    #[tool(description = "Use this when you need to create a new wiki page with a title and markdown content. Content is plain text (base64-encoding is handled automatically). Returns the created page title. Fails with 403 if wiki is disabled for the repository. Note: Wiki CRUD is only available on Gitea/Forgejo; GitHub does not expose a wiki API.")]
    async fn wiki_create(&self, Parameters(p): Parameters<WikiCreateParams>) -> Result<CallToolResult, ErrorData> {
        map_err(crate::tools::wiki::wiki_create(self.client.as_ref(), p, self.detected_repo.as_ref()).await)
    }

    // ── Organizations ───────────────────────────────────────────────

    #[tool(description = "Use this when you need to list organizations the authenticated user belongs to. Returns organization names and full names.")]
    async fn org_list(&self, Parameters(_p): Parameters<OrgListParams>) -> Result<CallToolResult, ErrorData> {
        map_err(crate::tools::orgs::org_list(self.client.as_ref()).await)
    }

    #[tool(description = "Use this when you need to get details about a specific organization by its name. Returns name, full name, description, location, and website.")]
    async fn org_get(&self, Parameters(p): Parameters<OrgGetParams>) -> Result<CallToolResult, ErrorData> {
        map_err(crate::tools::orgs::org_get(self.client.as_ref(), p).await)
    }

    #[tool(description = "Use this when you need to list teams in an organization. Returns team name, ID, and permission level for each team.")]
    async fn org_teams(&self, Parameters(p): Parameters<OrgTeamsParams>) -> Result<CallToolResult, ErrorData> {
        map_err(crate::tools::orgs::org_teams(self.client.as_ref(), p).await)
    }

    // ── Actions / CI ────────────────────────────────────────────────

    #[tool(description = "Use this when you need to list CI/CD workflows (Actions) configured in a repository. On Gitea, tries the Actions API first, then falls back to listing workflow files in .gitea/workflows or .github/workflows. On GitHub, uses the native workflows API. Returns workflow file names.")]
    async fn actions_workflow_list(&self, Parameters(p): Parameters<ActionsWorkflowListParams>) -> Result<CallToolResult, ErrorData> {
        map_err(crate::tools::actions::actions_workflow_list(self.client.as_ref(), p, self.detected_repo.as_ref()).await)
    }

    #[tool(description = "Use this when you need to list workflow runs for a repository. Returns run number, workflow path, title, and status/conclusion for each run. Use actions_run_get with a run ID for full details.")]
    async fn actions_run_list(&self, Parameters(p): Parameters<ActionsRunListParams>) -> Result<CallToolResult, ErrorData> {
        map_err(crate::tools::actions::actions_run_list(self.client.as_ref(), p, self.detected_repo.as_ref()).await)
    }

    #[tool(description = "Use this when you need to get details of a specific workflow run by its ID. Returns run number, title, status, conclusion, workflow path, event, branch, actor, and timestamps. Use actions_job_logs with a job ID to see logs for debugging.")]
    async fn actions_run_get(&self, Parameters(p): Parameters<ActionsRunGetParams>) -> Result<CallToolResult, ErrorData> {
        map_err(crate::tools::actions::actions_run_get(self.client.as_ref(), p, self.detected_repo.as_ref()).await)
    }

    #[tool(description = "Use this when you need to get the logs of a specific CI/CD job to debug failures. Requires a job ID from the workflow run. Returns the raw log output in a code block.")]
    async fn actions_job_logs(&self, Parameters(p): Parameters<ActionsJobLogsParams>) -> Result<CallToolResult, ErrorData> {
        map_err(crate::tools::actions::actions_job_logs(self.client.as_ref(), p, self.detected_repo.as_ref()).await)
    }
}

// Extracted resource logic — testable without RequestContext.
impl GitxMcp {
    fn build_resource_list(&self) -> std::result::Result<ListResourcesResult, ErrorData> {
        let resources = if let Some(ref info) = self.detected_repo {
            vec![RawResource {
                uri: RESOURCE_URI.to_string(),
                name: "detected-repo".to_string(),
                title: Some(format!("{}/{}", info.owner, info.repo)),
                description: Some(
                    "Auto-detected repository from the server's working directory. \
                     When present, owner and repo params can be omitted from tool calls."
                        .to_string(),
                ),
                mime_type: Some("application/json".to_string()),
                size: None,
                icons: None,
                meta: None,
            }
            .no_annotation()]
        } else {
            vec![]
        };

        Ok(ListResourcesResult {
            resources,
            next_cursor: None,
            meta: None,
        })
    }

    fn build_resource_read(&self, uri: &str) -> std::result::Result<ReadResourceResult, ErrorData> {
        if uri != RESOURCE_URI {
            return Err(ErrorData::resource_not_found(
                format!("Unknown resource URI: {uri}"),
                None,
            ));
        }

        let info = self.detected_repo.as_ref().ok_or_else(|| {
            ErrorData::resource_not_found(
                "No repository detected in the server's working directory".to_string(),
                None,
            )
        })?;

        let json = serde_json::json!({
            "owner": info.owner,
            "repo": info.repo,
        });

        Ok(ReadResourceResult {
            contents: vec![ResourceContents::text(json.to_string(), RESOURCE_URI)],
        })
    }
}

#[tool_handler]
impl ServerHandler for GitxMcp {
    fn get_info(&self) -> ServerInfo {
        let platform_name = match self.client.platform() {
            Platform::Gitea => "Gitea/Forgejo",
            Platform::GitHub => "GitHub",
        };

        let instructions = match self.client.platform() {
            Platform::Gitea => {
                format!(
                    "{platform_name} MCP server with 43 tools covering issues, PRs, files, branches, \
                     commits, labels, milestones, releases, notifications, wiki, organizations, and \
                     CI/CD actions. Read the repo://detected resource to get the auto-detected \
                     owner/repo — when set, owner and repo params can be omitted from all tool calls. \
                     You can still override with explicit owner+repo or directory params. \
                     For file updates/deletes, call file_read first to get the required SHA. \
                     For assigning labels or milestones, use label_list/milestone_list to get numeric IDs."
                )
            }
            Platform::GitHub => {
                format!(
                    "{platform_name} MCP server with 43 tools covering issues, PRs, files, branches, \
                     commits, labels, milestones, releases, notifications, organizations, and \
                     CI/CD actions. Read the repo://detected resource to get the auto-detected \
                     owner/repo — when set, owner and repo params can be omitted from all tool calls. \
                     You can still override with explicit owner+repo or directory params. \
                     For file updates/deletes, call file_read first to get the required SHA. \
                     Labels use names (strings), not numeric IDs. Wiki CRUD is not available on GitHub."
                )
            }
        };

        ServerInfo {
            protocol_version: ProtocolVersion::V_2024_11_05,
            capabilities: ServerCapabilities::builder()
                .enable_resources()
                .enable_tools()
                .build(),
            server_info: Implementation {
                name: "gitx-mcp".to_string(),
                version: env!("CARGO_PKG_VERSION").to_string(),
                title: None,
                description: None,
                icons: None,
                website_url: None,
            },
            instructions: Some(instructions),
        }
    }

    async fn list_resources(
        &self,
        _request: Option<PaginatedRequestParams>,
        _context: RequestContext<RoleServer>,
    ) -> std::result::Result<ListResourcesResult, ErrorData> {
        self.build_resource_list()
    }

    async fn read_resource(
        &self,
        request: ReadResourceRequestParams,
        _context: RequestContext<RoleServer>,
    ) -> std::result::Result<ReadResourceResult, ErrorData> {
        self.build_resource_read(&request.uri)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repo_resolver::RepoInfo;

    // ── resolve_owner_repo tests ───────────────────────────────────

    #[test]
    fn explicit_owner_repo_takes_priority() {
        let owner = Some("alice".to_string());
        let repo = Some("my-repo".to_string());
        let default = RepoInfo {
            owner: "default-owner".to_string(),
            repo: "default-repo".to_string(),
        };

        let (o, r) = resolve_owner_repo(&owner, &repo, &None, Some(&default)).unwrap();
        assert_eq!(o, "alice");
        assert_eq!(r, "my-repo");
    }

    #[test]
    fn empty_strings_skip_to_default() {
        let owner = Some(String::new());
        let repo = Some(String::new());
        let default = RepoInfo {
            owner: "fallback-owner".to_string(),
            repo: "fallback-repo".to_string(),
        };

        let (o, r) = resolve_owner_repo(&owner, &repo, &None, Some(&default)).unwrap();
        assert_eq!(o, "fallback-owner");
        assert_eq!(r, "fallback-repo");
    }

    #[test]
    fn partial_owner_repo_falls_to_default() {
        // Only owner provided, no repo — should skip to default
        let owner = Some("alice".to_string());
        let repo: Option<String> = None;
        let default = RepoInfo {
            owner: "fallback-owner".to_string(),
            repo: "fallback-repo".to_string(),
        };

        let (o, r) = resolve_owner_repo(&owner, &repo, &None, Some(&default)).unwrap();
        assert_eq!(o, "fallback-owner");
        assert_eq!(r, "fallback-repo");
    }

    #[test]
    fn directory_overrides_default() {
        // Create a temp dir with a fake .git/config containing origin
        let tmp = std::env::temp_dir().join("gitx_mcp_test_dir_override");
        let git_dir = tmp.join(".git");
        std::fs::create_dir_all(&git_dir).unwrap();
        std::fs::write(
            git_dir.join("config"),
            "[remote \"origin\"]\n\turl = https://example.com/dir-owner/dir-repo.git\n",
        )
        .unwrap();

        let default = RepoInfo {
            owner: "should-not-use".to_string(),
            repo: "should-not-use".to_string(),
        };

        let (o, r) = resolve_owner_repo(
            &None,
            &None,
            &Some(tmp.to_string_lossy().to_string()),
            Some(&default),
        )
        .unwrap();
        assert_eq!(o, "dir-owner");
        assert_eq!(r, "dir-repo");

        // Cleanup
        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn empty_directory_uses_default() {
        let default = RepoInfo {
            owner: "default-owner".to_string(),
            repo: "default-repo".to_string(),
        };

        let (o, r) = resolve_owner_repo(&None, &None, &Some(String::new()), Some(&default)).unwrap();
        assert_eq!(o, "default-owner");
        assert_eq!(r, "default-repo");
    }

    #[test]
    fn no_default_no_directory_falls_to_cwd() {
        // No explicit params, no default — falls through to cwd "." detection.
        // The test repo may or may not have an origin remote, so just verify
        // the function doesn't panic and returns Ok or a clean error.
        let result = resolve_owner_repo(&None, &None, &None, None);
        // Either it succeeds (origin exists) or fails cleanly
        match result {
            Ok((o, r)) => {
                assert!(!o.is_empty());
                assert!(!r.is_empty());
            }
            Err(e) => {
                // Should be a RepoResolution error, not a panic
                let msg = format!("{e}");
                assert!(msg.contains("origin") || msg.contains(".git"));
            }
        }
    }

    #[test]
    fn bad_directory_no_default_errors() {
        let result = resolve_owner_repo(
            &None,
            &None,
            &Some("/nonexistent/path/that/does/not/exist".to_string()),
            None,
        );
        assert!(result.is_err());
    }

    // ── Helper to build GitxMcp for tests ─────────────────────────

    fn test_server(detected_repo: Option<RepoInfo>) -> GitxMcp {
        let config = crate::config::Config {
            base_url: "http://localhost:3000".to_string(),
            token: "test-token".to_string(),
            platform: Platform::Gitea,
        };
        let client: Arc<dyn GitClient> = Arc::new(crate::client::GiteaClient::new(&config).unwrap());
        GitxMcp {
            client,
            tool_router: GitxMcp::tool_router(),
            detected_repo,
        }
    }

    // ── Resource logic tests ───────────────────────────────────────

    #[test]
    fn resource_uri_constant() {
        assert_eq!(RESOURCE_URI, "repo://detected");
    }

    #[test]
    fn list_resources_with_detected_repo() {
        let server = test_server(Some(RepoInfo {
            owner: "myorg".to_string(),
            repo: "myproject".to_string(),
        }));

        let result = server.build_resource_list().unwrap();
        assert_eq!(result.resources.len(), 1);

        let res = &result.resources[0];
        assert_eq!(res.raw.uri, "repo://detected");
        assert_eq!(res.raw.name, "detected-repo");
        assert_eq!(res.raw.title.as_deref(), Some("myorg/myproject"));
        assert_eq!(res.raw.mime_type.as_deref(), Some("application/json"));
    }

    #[test]
    fn list_resources_without_detected_repo() {
        let server = test_server(None);

        let result = server.build_resource_list().unwrap();
        assert!(result.resources.is_empty());
    }

    #[test]
    fn read_resource_returns_owner_repo_json() {
        let server = test_server(Some(RepoInfo {
            owner: "testowner".to_string(),
            repo: "testrepo".to_string(),
        }));

        let result = server.build_resource_read("repo://detected").unwrap();
        assert_eq!(result.contents.len(), 1);

        // Parse the text content as JSON
        let content = &result.contents[0];
        let text = match content {
            ResourceContents::TextResourceContents { text, .. } => text,
            _ => panic!("Expected text resource content"),
        };
        let parsed: serde_json::Value = serde_json::from_str(text).unwrap();
        assert_eq!(parsed["owner"], "testowner");
        assert_eq!(parsed["repo"], "testrepo");
    }

    #[test]
    fn read_resource_unknown_uri_errors() {
        let server = test_server(Some(RepoInfo {
            owner: "x".to_string(),
            repo: "y".to_string(),
        }));

        let err = server.build_resource_read("repo://unknown").unwrap_err();
        assert_eq!(err.code, ErrorCode::RESOURCE_NOT_FOUND);
    }

    #[test]
    fn read_resource_no_detected_repo_errors() {
        let server = test_server(None);

        let err = server.build_resource_read("repo://detected").unwrap_err();
        assert_eq!(err.code, ErrorCode::RESOURCE_NOT_FOUND);
    }

    #[test]
    fn cwd_repo_detection_graceful() {
        // The test environment may or may not have an origin remote.
        // Just verify resolve_repo doesn't panic.
        let result = repo_resolver::resolve_repo(".");
        match result {
            Ok(info) => {
                assert!(!info.owner.is_empty());
                assert!(!info.repo.is_empty());
            }
            Err(e) => {
                let msg = format!("{e}");
                assert!(msg.contains("origin") || msg.contains(".git"));
            }
        }
    }

    #[test]
    fn resolve_repo_from_fake_git_config() {
        // Create a temp dir with a proper .git/config
        let tmp = std::env::temp_dir().join("gitx_mcp_test_resolve");
        let git_dir = tmp.join(".git");
        std::fs::create_dir_all(&git_dir).unwrap();
        std::fs::write(
            git_dir.join("config"),
            "[remote \"origin\"]\n\turl = git@gitea.example.com:myorg/myproject.git\n",
        )
        .unwrap();

        let info = repo_resolver::resolve_repo(&tmp.to_string_lossy()).unwrap();
        assert_eq!(info.owner, "myorg");
        assert_eq!(info.repo, "myproject");

        let _ = std::fs::remove_dir_all(&tmp);
    }
}
