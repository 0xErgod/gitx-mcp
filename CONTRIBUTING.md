# Contributing to gitx-mcp

Thanks for your interest in contributing. This document covers the basics for getting started.

## Prerequisites

- [Rust](https://rustup.rs/) 1.75+
- A Gitea or Forgejo instance for testing (a local instance works fine)
- An API token with full permissions on a test repository

## Development setup

```sh
git clone https://github.com/0xErgod/gitx-mcp.git
cd gitx-mcp
cp .env.example .env
# Edit .env with your Gitea URL and token
cargo build
```

## Running checks

All three must pass before submitting a PR:

```sh
cargo check
cargo test
cargo clippy
```

## Project structure

```
src/
  server.rs          # Tool routing and descriptions
  client.rs          # HTTP client for Gitea API
  config.rs          # Environment variable configuration
  error.rs           # Error types and MCP error mapping
  repo_resolver.rs   # Auto-detect owner/repo from .git/config
  response.rs        # Markdown formatters for tool responses
  tools/
    mod.rs           # Module declarations
    issues.rs        # issue_list, issue_get, issue_create, issue_edit
    issue_comments.rs
    pulls.rs         # pr_list, pr_get, pr_create, pr_edit, pr_merge
    pull_reviews.rs
    pull_files.rs    # pr_files, pr_diff
    files.rs         # file_read, file_list, file_create, file_update, file_delete, tree_get
    branches.rs
    commits.rs
    labels.rs
    milestones.rs
    notifications.rs
    releases.rs
    repo.rs
    tags.rs
    users.rs
    wiki.rs
    orgs.rs
    actions.rs
  types/
    common.rs        # Shared param structs (RepoParams, PaginationParams)
    mod.rs
```

## Adding a new tool

1. **Add the handler** in `src/tools/<module>.rs`:
   - Create a `Params` struct with `#[derive(Debug, Deserialize, JsonSchema)]`
   - Document every field with `///` doc-comments (schemars converts these to JSON Schema descriptions)
   - Write the async handler function

2. **Register the tool** in `src/server.rs`:
   - Import the params struct
   - Add a `#[tool(description = "...")]` method in the `#[tool_router] impl GitxMcp` block
   - Write a clear description that includes: when to use, what it returns, related tools, and error conditions

3. **Add a response formatter** in `src/response.rs` if the tool returns structured data that needs formatting.

4. **Update the tool count** in the `GitxMcp` struct doc-comment and the `instructions` string in `get_info()`.

5. **Update README.md** with the new tool in the appropriate table.

### Tool description guidelines

Every tool description should answer these questions for an AI agent:

- **When** should I use this tool?
- **What** does it return?
- **What** other tools are related? (workflow hints)
- **What** errors might occur?

Example:

```rust
#[tool(description = "Use this when you need to read the content of a file from the repository at a specific ref (branch, tag, or commit SHA). Returns the file path, size, SHA, and decoded content. IMPORTANT: The returned SHA is required by file_update and file_delete — always call file_read first before updating or deleting a file. Fails with 404 if the file or ref does not exist.")]
```

### Parameter documentation guidelines

- Document every field with a `///` doc-comment
- Include defaults: `/// Page number (1-based). Defaults to 1.`
- Include allowed values: `/// Filter by state: open, closed, or all. Defaults to open.`
- Include cross-tool references: `/// Label IDs (from label_list).`
- Include format examples: `/// Label color as hex (e.g. "#ff0000" or "ff0000").`

## Submitting a pull request

1. Fork and create a feature branch from `main`
2. Make your changes
3. Ensure `cargo check && cargo test && cargo clippy` all pass
4. Write a clear PR description explaining what and why
5. Submit the PR against `main`

## Reporting issues

When reporting a bug, include:

- The tool name and parameters you used
- The error message or unexpected behavior
- Your Gitea/Forgejo version
- Your gitx-mcp version (`gitx-mcp --version` or check `Cargo.toml`)

## Code style

- Follow existing patterns in the codebase
- No unnecessary abstractions — keep it simple
- Match the Gitea API field names in param structs where possible
- Use `serde(rename = "...")` when the Rust name must differ from the API name
