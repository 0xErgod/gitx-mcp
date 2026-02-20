use std::path::Path;

use crate::error::{GitxError, Result};

/// Owner and repository name pair extracted from a git remote.
#[derive(Debug, Clone)]
pub struct RepoInfo {
    pub owner: String,
    pub repo: String,
}

/// Resolve the owner/repo from a `.git/config` file in the given directory.
///
/// Parses the `[remote "origin"]` section and extracts owner/repo from the URL.
/// Supports SSH (`git@host:owner/repo.git`), HTTPS (`https://host/owner/repo.git`),
/// and path-style URLs.
pub fn resolve_repo(directory: &str) -> Result<RepoInfo> {
    let git_config_path = Path::new(directory).join(".git").join("config");

    if !git_config_path.exists() {
        return Err(GitxError::RepoResolution(format!(
            "No .git/config found in {directory}"
        )));
    }

    let content = std::fs::read_to_string(&git_config_path).map_err(|e| {
        GitxError::RepoResolution(format!("Failed to read .git/config: {e}"))
    })?;

    // Simple parser: find [remote "origin"] section, then url = ...
    let mut in_origin = false;
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('[') {
            in_origin = trimmed == "[remote \"origin\"]";
            continue;
        }
        if in_origin {
            if let Some(url) = trimmed.strip_prefix("url").and_then(|s| {
                let s = s.trim_start();
                s.strip_prefix('=').map(|s| s.trim())
            }) {
                return parse_remote_url(url);
            }
        }
    }

    Err(GitxError::RepoResolution(
        "No remote 'origin' URL found in .git/config".to_string(),
    ))
}

/// Parse a git remote URL into owner/repo.
fn parse_remote_url(url: &str) -> Result<RepoInfo> {
    let url = url.trim();

    // SSH: git@host:owner/repo.git
    if let Some(path) = url.strip_prefix("git@").and_then(|s| s.split_once(':').map(|(_, p)| p)) {
        return extract_owner_repo(path);
    }

    // SSH: ssh://git@host/owner/repo.git
    if url.starts_with("ssh://") {
        if let Some(path) = url
            .strip_prefix("ssh://")
            .and_then(|s| s.split_once('/').map(|(_, p)| p))
        {
            return extract_owner_repo(path);
        }
    }

    // HTTPS: https://host/owner/repo.git
    if url.starts_with("http://") || url.starts_with("https://") {
        if let Ok(parsed) = url::Url::parse(url) {
            let path = parsed.path().trim_start_matches('/');
            return extract_owner_repo(path);
        }
    }

    // Fallback: try treating as path
    extract_owner_repo(url)
}

/// Extract owner/repo from a path like `owner/repo.git` or `owner/repo`.
fn extract_owner_repo(path: &str) -> Result<RepoInfo> {
    let path = path.trim_end_matches(".git").trim_matches('/');
    let parts: Vec<&str> = path.splitn(3, '/').collect();

    if parts.len() < 2 {
        return Err(GitxError::RepoResolution(format!(
            "Cannot extract owner/repo from path: {path}"
        )));
    }

    Ok(RepoInfo {
        owner: parts[0].to_string(),
        repo: parts[1].to_string(),
    })
}
