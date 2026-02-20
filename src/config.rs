use crate::error::{GitxError, Result};
use crate::platform::Platform;

/// Server configuration loaded from environment variables.
#[derive(Debug, Clone)]
pub struct Config {
    /// Base URL of the git hosting instance (e.g. `https://git.example.com` or `https://github.com`)
    pub base_url: String,
    /// API token for authentication
    pub token: String,
    /// Which platform this config targets
    pub platform: Platform,
}

impl Config {
    /// Load configuration from environment variables.
    ///
    /// Platform detection priority:
    /// 1. `GIT_PLATFORM` env var (explicit: "gitea", "forgejo", or "github")
    /// 2. If `GITHUB_TOKEN` is set (and no Gitea vars) → GitHub
    /// 3. If `GITEA_URL`/`GITEA_TOKEN` (or Forgejo equivalents) are set → Gitea
    /// 4. Error if nothing is configured
    pub fn from_env() -> Result<Self> {
        let _ = dotenvy::dotenv(); // ignore missing .env

        // Check what env vars are available
        let explicit_platform = std::env::var("GIT_PLATFORM").ok();
        let gitea_url = std::env::var("GITEA_URL")
            .or_else(|_| std::env::var("FORGEJO_REMOTE_URL"))
            .ok();
        let gitea_token = std::env::var("GITEA_TOKEN")
            .or_else(|_| std::env::var("FORGEJO_AUTH_TOKEN"))
            .ok();
        let github_token = std::env::var("GITHUB_TOKEN").ok();
        let github_url = std::env::var("GITHUB_URL").ok();

        // 1. Explicit platform override
        if let Some(ref p) = explicit_platform {
            match p.to_lowercase().as_str() {
                "gitea" | "forgejo" => {
                    let base_url = gitea_url.ok_or_else(|| {
                        GitxError::MissingParam(
                            "GIT_PLATFORM=gitea but GITEA_URL (or FORGEJO_REMOTE_URL) is not set"
                                .to_string(),
                        )
                    })?;
                    let token = gitea_token.ok_or_else(|| {
                        GitxError::MissingParam(
                            "GIT_PLATFORM=gitea but GITEA_TOKEN (or FORGEJO_AUTH_TOKEN) is not set"
                                .to_string(),
                        )
                    })?;
                    let base_url = base_url.trim_end_matches('/').to_string();
                    return Ok(Config {
                        base_url,
                        token,
                        platform: Platform::Gitea,
                    });
                }
                "github" => {
                    let token = github_token.or(gitea_token).ok_or_else(|| {
                        GitxError::MissingParam(
                            "GIT_PLATFORM=github but GITHUB_TOKEN is not set".to_string(),
                        )
                    })?;
                    let base_url = github_url
                        .unwrap_or_else(|| "https://github.com".to_string());
                    let base_url = base_url.trim_end_matches('/').to_string();
                    return Ok(Config {
                        base_url,
                        token,
                        platform: Platform::GitHub,
                    });
                }
                other => {
                    return Err(GitxError::MissingParam(format!(
                        "GIT_PLATFORM={other} is not recognized. Use 'gitea', 'forgejo', or 'github'."
                    )));
                }
            }
        }

        // 2. Auto-detect: GITHUB_TOKEN set (and no Gitea vars) → GitHub
        if github_token.is_some() && gitea_url.is_none() && gitea_token.is_none() {
            let token = github_token.unwrap();
            let base_url = github_url
                .unwrap_or_else(|| "https://github.com".to_string());
            let base_url = base_url.trim_end_matches('/').to_string();
            return Ok(Config {
                base_url,
                token,
                platform: Platform::GitHub,
            });
        }

        // 3. Both tokens set — detect from git remote in CWD
        if let (Some(ref gh_token), Some(ref gt_url), Some(ref gt_token)) =
            (&github_token, &gitea_url, &gitea_token)
        {
            if let Some(platform) = detect_platform_from_remote(gt_url) {
                match platform {
                    Platform::GitHub => {
                        let base_url = github_url
                            .unwrap_or_else(|| "https://github.com".to_string())
                            .trim_end_matches('/')
                            .to_string();
                        return Ok(Config {
                            base_url,
                            token: gh_token.clone(),
                            platform: Platform::GitHub,
                        });
                    }
                    Platform::Gitea => {
                        return Ok(Config {
                            base_url: gt_url.trim_end_matches('/').to_string(),
                            token: gt_token.clone(),
                            platform: Platform::Gitea,
                        });
                    }
                }
            }
            return Err(GitxError::MissingParam(
                "Both GITHUB_TOKEN and GITEA_URL/GITEA_TOKEN are set. \
                 Set GIT_PLATFORM=github or GIT_PLATFORM=gitea to disambiguate."
                    .to_string(),
            ));
        }

        // 4. Auto-detect: Gitea/Forgejo vars only
        if let (Some(base_url), Some(token)) = (gitea_url, gitea_token) {
            let base_url = base_url.trim_end_matches('/').to_string();
            return Ok(Config {
                base_url,
                token,
                platform: Platform::Gitea,
            });
        }

        Err(GitxError::MissingParam(
            "No git platform credentials found. Set GITEA_URL + GITEA_TOKEN for Gitea/Forgejo, \
             or GITHUB_TOKEN for GitHub."
                .to_string(),
        ))
    }
}

/// Try to detect platform from the git remote URL in the current working directory.
/// Returns `Some(Platform)` if a remote origin was found and matched.
fn detect_platform_from_remote(gitea_url: &str) -> Option<Platform> {
    let output = std::process::Command::new("git")
        .args(["config", "--get", "remote.origin.url"])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let remote = String::from_utf8_lossy(&output.stdout);
    let remote = remote.trim();
    if remote.is_empty() {
        return None;
    }

    // Extract host from gitea_url for comparison
    let gitea_host = url::Url::parse(gitea_url)
        .ok()
        .and_then(|u| u.host_str().map(|h| h.to_lowercase()));

    // Check if remote matches GitHub
    if remote.contains("github.com") {
        return Some(Platform::GitHub);
    }

    // Check if remote matches the configured Gitea host
    if let Some(ref host) = gitea_host {
        if remote.contains(host.as_str()) {
            return Some(Platform::Gitea);
        }
    }

    None
}
