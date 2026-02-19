# gitx-mcp

MCP server for Gitea and Forgejo instances, optimized for AI agents. Provides 43 tools covering issues, pull requests, files, branches, commits, labels, milestones, releases, notifications, wiki, organizations, users, and CI/CD actions.

Built with [rmcp](https://github.com/anthropics/rmcp) and communicates over stdio using the [Model Context Protocol](https://modelcontextprotocol.io/).

## Installation

Requires [Rust](https://rustup.rs/) 1.75+.

Install from the git repository:

```sh
cargo install --git https://github.com/0xErgod/gitx-mcp.git
```

Or clone and build locally:

```sh
git clone https://github.com/0xErgod/gitx-mcp.git
cd gitx-mcp
cargo install --path .
```

The binary is installed to `~/.cargo/bin/gitx-mcp`.

## Configuration

gitx-mcp requires two environment variables:

| Variable | Description |
|---|---|
| `GITEA_URL` | Base URL of your Gitea/Forgejo instance (e.g. `https://git.example.com`) |
| `GITEA_TOKEN` | API token with appropriate permissions |

For backward compatibility, `FORGEJO_REMOTE_URL` and `FORGEJO_AUTH_TOKEN` are also accepted.

### Generating an API token

1. Go to your Gitea/Forgejo instance
2. Navigate to **Settings > Applications > Access Tokens**
3. Create a token with the scopes you need (e.g. `repo`, `issue`, `admin:org`)

## Setup with Claude Code

Add gitx-mcp as an MCP server in your Claude Code settings. You can configure it at the project level or globally.

### Project-level setup

Create or edit `.mcp.json` in your project root:

```json
{
  "mcpServers": {
    "gitx-mcp": {
      "command": "gitx-mcp",
      "env": {
        "GITEA_URL": "https://your-gitea-instance.com",
        "GITEA_TOKEN": "your-api-token-here"
      }
    }
  }
}
```

### Global setup

Add to `~/.claude/settings.json` (or use `/mcp` in Claude Code):

```json
{
  "mcpServers": {
    "gitx-mcp": {
      "command": "gitx-mcp",
      "env": {
        "GITEA_URL": "https://your-gitea-instance.com",
        "GITEA_TOKEN": "your-api-token-here"
      }
    }
  }
}
```

### Using a .env file

Alternatively, create a `.env` file in your working directory (see `.env.example`):

```sh
GITEA_URL=https://your-gitea-instance.com
GITEA_TOKEN=your-api-token-here
```

Then the MCP config only needs:

```json
{
  "mcpServers": {
    "gitx-mcp": {
      "command": "gitx-mcp"
    }
  }
}
```

### Repository resolution

Most tools accept `owner` and `repo` parameters to identify the target repository. Alternatively, you can pass a `directory` parameter pointing to a local clone, and gitx-mcp will auto-detect the owner and repo from `.git/config`.

## Tools

### Issues (4 tools)

| Tool | Description |
|---|---|
| `issue_list` | List issues in a repository. Filter by state (open/closed) and labels. |
| `issue_get` | Get full details of a specific issue including body, labels, assignees, and milestone. |
| `issue_create` | Create a new issue with title, body, labels, milestone, and assignees. |
| `issue_edit` | Update an issue's title, body, state, labels, assignees, or milestone. |

### Issue Comments (2 tools)

| Tool | Description |
|---|---|
| `issue_comment_list` | List all comments on a specific issue or pull request. |
| `issue_comment_create` | Add a comment to an existing issue or pull request. |

### Pull Requests (5 tools)

| Tool | Description |
|---|---|
| `pr_list` | List pull requests in a repository. Filter by state (open/closed/all). |
| `pr_get` | Get full PR details including branches, mergeable status, labels, and assignees. |
| `pr_create` | Create a new pull request with head/base branches, title, body, and labels. |
| `pr_edit` | Update a PR's title, body, state, labels, or assignees. |
| `pr_merge` | Merge a pull request using merge, rebase, or squash strategy. |

### Pull Request Reviews (2 tools)

| Tool | Description |
|---|---|
| `pr_review_list` | List reviews on a pull request with reviewer, state, and body. |
| `pr_review_create` | Submit a review: APPROVED, REQUEST_CHANGES, or COMMENT. |

### Pull Request Files (2 tools)

| Tool | Description |
|---|---|
| `pr_files` | List changed files in a PR with status and diff stats. |
| `pr_diff` | Get the raw unified diff of all changes in a pull request. |

### Files (6 tools)

| Tool | Description |
|---|---|
| `file_read` | Read file content at a specific ref. Returns path, size, SHA, and content. |
| `file_list` | List files and directories at a path (single directory level). |
| `file_create` | Create a new file with a commit. Content is plain text (auto base64-encoded). |
| `file_update` | Update an existing file. Requires SHA from `file_read`. |
| `file_delete` | Delete a file. Requires SHA from `file_read`. |
| `tree_get` | Get the full recursive file tree of the repository. |

### Branches (5 tools)

| Tool | Description |
|---|---|
| `branch_list` | List all branches with latest commit SHA and protection status. |
| `branch_create` | Create a new branch from an existing branch or commit SHA. |
| `branch_delete` | Delete a branch. |
| `branch_protection_list` | List branch protection rules. |
| `branch_protection_create` | Create a branch protection rule with push and review settings. |

### Commits (4 tools)

| Tool | Description |
|---|---|
| `commit_list` | List commits, optionally filtered by branch/tag or file path. |
| `commit_get` | Get full commit details including diff stats and parent commits. |
| `commit_diff` | Get the raw unified diff of a specific commit. |
| `commit_compare` | Compare two refs and see commits and changed files between them. |

### Labels (3 tools)

| Tool | Description |
|---|---|
| `label_list` | List all labels with ID, name, color, and description. |
| `label_create` | Create a new label with a name and hex color. |
| `label_edit` | Update a label's name, color, or description. |

### Milestones (3 tools)

| Tool | Description |
|---|---|
| `milestone_list` | List milestones with ID, title, state, and issue counts. |
| `milestone_get` | Get full milestone details including description and due date. |
| `milestone_create` | Create a new milestone with title, description, and due date. |

### Notifications (2 tools)

| Tool | Description |
|---|---|
| `notification_list` | List your notifications with subject, type, and read status. |
| `notification_mark_read` | Mark all or a specific notification as read. |

### Releases (3 tools)

| Tool | Description |
|---|---|
| `release_list` | List releases with tag name, title, and draft/prerelease flags. |
| `release_get` | Get full release details including body and assets. |
| `release_create` | Create a new release with tag, title, and release notes. |

### Repository (2 tools)

| Tool | Description |
|---|---|
| `repo_get` | Get repository metadata: description, default branch, stars, forks, visibility. |
| `repo_search` | Search repositories by keyword across the Gitea instance. |

### Users (2 tools)

| Tool | Description |
|---|---|
| `user_get_me` | Get the authenticated user's profile (username, email, admin status). |
| `user_get` | Get a user's public profile by username. |

### Tags (2 tools)

| Tool | Description |
|---|---|
| `tag_list` | List all tags with name and commit SHA. |
| `tag_create` | Create a new tag pointing to a commit or branch. |

### Wiki (3 tools)

| Tool | Description |
|---|---|
| `wiki_list` | List wiki pages with title and slug. |
| `wiki_get` | Read a wiki page's content by its slug. |
| `wiki_create` | Create a new wiki page with title and markdown content. |

### Organizations (3 tools)

| Tool | Description |
|---|---|
| `org_list` | List organizations the authenticated user belongs to. |
| `org_get` | Get organization details by name. |
| `org_teams` | List teams in an organization with permissions. |

### Actions / CI (4 tools)

| Tool | Description |
|---|---|
| `actions_workflow_list` | List CI/CD workflows configured in the repository. |
| `actions_run_list` | List workflow runs with status and conclusion. |
| `actions_run_get` | Get details of a specific workflow run. |
| `actions_job_logs` | Get logs of a specific CI/CD job for debugging. |

## Key Workflows

### Updating or deleting a file

Always call `file_read` first to get the current file SHA, then pass it to `file_update` or `file_delete`:

```
file_read(path: "README.md") -> SHA: "abc123..."
file_update(path: "README.md", sha: "abc123...", content: "...", message: "update readme")
```

### Assigning labels or milestones

Labels and milestones use numeric IDs, not names. Look up the IDs first:

```
label_list() -> [{id: 1, name: "bug"}, {id: 2, name: "enhancement"}]
issue_create(title: "Fix login", labels: [1])
```

### Merging a pull request

Check that the PR is mergeable before merging:

```
pr_get(index: 42) -> mergeable: true
pr_merge(index: 42, merge_style: "squash")
```

## Releasing

This project uses [release-plz](https://release-plz.ieni.dev/) for local version bumping and changelog generation. No CI/CD required.

### Install release-plz

```sh
cargo install release-plz --locked
```

### Workflow

```sh
# Preview what would change (dry run)
release-plz update --dry-run

# Bump version in Cargo.toml and update CHANGELOG.md
release-plz update

# Review, commit, and tag
git add -A && git commit -m "chore: release v$(grep '^version' Cargo.toml | head -1 | sed 's/.*"\(.*\)"/\1/')"
release-plz release
```

`release-plz update` reads conventional commits since the last git tag, determines the appropriate semver bump, updates `Cargo.toml`, and prepends entries to `CHANGELOG.md`. `release-plz release` creates a `vX.Y.Z` git tag.

Configuration lives in [`release-plz.toml`](release-plz.toml).

## License

MIT
