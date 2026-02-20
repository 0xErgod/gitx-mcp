use serde_json::Value;

/// Format a JSON value into a readable markdown string for agent consumption.
pub fn format_value(val: &Value) -> String {
    match val {
        Value::Null => "No data returned.".to_string(),
        Value::Array(arr) if arr.is_empty() => "No items found.".to_string(),
        Value::Array(arr) => arr
            .iter()
            .map(|item| format_object(item))
            .collect::<Vec<_>>()
            .join("\n---\n"),
        Value::Object(_) => format_object(val),
        other => other.to_string(),
    }
}

/// Format a JSON object into readable key: value lines.
fn format_object(val: &Value) -> String {
    match val {
        Value::Object(map) => {
            let mut lines = Vec::new();
            for (key, value) in map {
                let formatted = format_field(key, value);
                if !formatted.is_empty() {
                    lines.push(formatted);
                }
            }
            lines.join("\n")
        }
        other => other.to_string(),
    }
}

/// Format a single field, handling nested objects and arrays.
fn format_field(key: &str, value: &Value) -> String {
    match value {
        Value::Null => String::new(),
        Value::String(s) if s.is_empty() => String::new(),
        Value::String(s) => format!("**{key}:** {s}"),
        Value::Number(n) => format!("**{key}:** {n}"),
        Value::Bool(b) => format!("**{key}:** {b}"),
        Value::Array(arr) if arr.is_empty() => String::new(),
        Value::Array(arr) => {
            // For arrays of simple values, join inline
            let items: Vec<String> = arr
                .iter()
                .filter_map(|v| match v {
                    Value::String(s) => Some(s.clone()),
                    Value::Object(m) => {
                        // Common pattern: objects with a "name" field
                        m.get("name")
                            .or_else(|| m.get("title"))
                            .or_else(|| m.get("login"))
                            .and_then(|v| v.as_str())
                            .map(|s| s.to_string())
                    }
                    other => Some(other.to_string()),
                })
                .collect();
            if items.is_empty() {
                String::new()
            } else {
                format!("**{key}:** {}", items.join(", "))
            }
        }
        Value::Object(m) => {
            // For nested objects, extract common identifiers
            if let Some(name) = m
                .get("login")
                .or_else(|| m.get("name"))
                .or_else(|| m.get("title"))
                .or_else(|| m.get("full_name"))
                .and_then(|v| v.as_str())
            {
                format!("**{key}:** {name}")
            } else {
                String::new()
            }
        }
    }
}

/// Format an issue object into readable markdown.
pub fn format_issue(issue: &Value) -> String {
    let mut parts = Vec::new();

    if let Some(number) = issue.get("number").and_then(|v| v.as_i64()) {
        let title = issue
            .get("title")
            .and_then(|v| v.as_str())
            .unwrap_or("(untitled)");
        let state = issue
            .get("state")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown");
        parts.push(format!("## #{number} {title} [{state}]"));
    }

    if let Some(user) = issue
        .get("user")
        .and_then(|v| v.get("login"))
        .and_then(|v| v.as_str())
    {
        parts.push(format!("**Author:** {user}"));
    }

    if let Some(labels) = issue.get("labels").and_then(|v| v.as_array()) {
        let label_names: Vec<&str> = labels
            .iter()
            .filter_map(|l| l.get("name").and_then(|v| v.as_str()))
            .collect();
        if !label_names.is_empty() {
            parts.push(format!("**Labels:** {}", label_names.join(", ")));
        }
    }

    if let Some(assignees) = issue.get("assignees").and_then(|v| v.as_array()) {
        let names: Vec<&str> = assignees
            .iter()
            .filter_map(|a| a.get("login").and_then(|v| v.as_str()))
            .collect();
        if !names.is_empty() {
            parts.push(format!("**Assignees:** {}", names.join(", ")));
        }
    }

    if let Some(milestone) = issue
        .get("milestone")
        .and_then(|v| v.get("title"))
        .and_then(|v| v.as_str())
    {
        parts.push(format!("**Milestone:** {milestone}"));
    }

    if let Some(created) = issue.get("created_at").and_then(|v| v.as_str()) {
        parts.push(format!("**Created:** {created}"));
    }

    if let Some(updated) = issue.get("updated_at").and_then(|v| v.as_str()) {
        parts.push(format!("**Updated:** {updated}"));
    }

    if let Some(body) = issue.get("body").and_then(|v| v.as_str()) {
        if !body.is_empty() {
            parts.push(format!("\n{body}"));
        }
    }

    parts.join("\n")
}

/// Format a list of issues into readable markdown.
pub fn format_issue_list(issues: &[Value]) -> String {
    if issues.is_empty() {
        return "No issues found.".to_string();
    }
    issues
        .iter()
        .map(|issue| {
            let number = issue.get("number").and_then(|v| v.as_i64()).unwrap_or(0);
            let title = issue
                .get("title")
                .and_then(|v| v.as_str())
                .unwrap_or("(untitled)");
            let state = issue
                .get("state")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown");
            let labels = issue
                .get("labels")
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|l| l.get("name").and_then(|v| v.as_str()))
                        .collect::<Vec<_>>()
                        .join(", ")
                })
                .unwrap_or_default();
            let label_str = if labels.is_empty() {
                String::new()
            } else {
                format!(" [{labels}]")
            };
            format!("- #{number} {title} ({state}){label_str}")
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// Format a pull request object into readable markdown.
pub fn format_pull_request(pr: &Value) -> String {
    let mut parts = Vec::new();

    if let Some(number) = pr.get("number").and_then(|v| v.as_i64()) {
        let title = pr
            .get("title")
            .and_then(|v| v.as_str())
            .unwrap_or("(untitled)");
        let state = pr
            .get("state")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown");
        parts.push(format!("## PR #{number} {title} [{state}]"));
    }

    if let Some(user) = pr
        .get("user")
        .and_then(|v| v.get("login"))
        .and_then(|v| v.as_str())
    {
        parts.push(format!("**Author:** {user}"));
    }

    if let Some(head) = pr
        .get("head")
        .and_then(|v| v.get("label"))
        .and_then(|v| v.as_str())
    {
        let base = pr
            .get("base")
            .and_then(|v| v.get("label"))
            .and_then(|v| v.as_str())
            .unwrap_or("?");
        parts.push(format!("**Branch:** {head} -> {base}"));
    }

    if let Some(mergeable) = pr.get("mergeable").and_then(|v| v.as_bool()) {
        parts.push(format!("**Mergeable:** {mergeable}"));
    }

    if let Some(labels) = pr.get("labels").and_then(|v| v.as_array()) {
        let label_names: Vec<&str> = labels
            .iter()
            .filter_map(|l| l.get("name").and_then(|v| v.as_str()))
            .collect();
        if !label_names.is_empty() {
            parts.push(format!("**Labels:** {}", label_names.join(", ")));
        }
    }

    if let Some(created) = pr.get("created_at").and_then(|v| v.as_str()) {
        parts.push(format!("**Created:** {created}"));
    }

    if let Some(body) = pr.get("body").and_then(|v| v.as_str()) {
        if !body.is_empty() {
            parts.push(format!("\n{body}"));
        }
    }

    parts.join("\n")
}

/// Format a list of pull requests.
pub fn format_pr_list(prs: &[Value]) -> String {
    if prs.is_empty() {
        return "No pull requests found.".to_string();
    }
    prs.iter()
        .map(|pr| {
            let number = pr.get("number").and_then(|v| v.as_i64()).unwrap_or(0);
            let title = pr
                .get("title")
                .and_then(|v| v.as_str())
                .unwrap_or("(untitled)");
            let state = pr
                .get("state")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown");
            format!("- PR #{number} {title} ({state})")
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// Format a comment object.
pub fn format_comment(comment: &Value) -> String {
    let user = comment
        .get("user")
        .and_then(|v| v.get("login"))
        .and_then(|v| v.as_str())
        .unwrap_or("unknown");
    let created = comment
        .get("created_at")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let body = comment
        .get("body")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let id = comment.get("id").and_then(|v| v.as_i64()).unwrap_or(0);

    format!("**Comment #{id}** by {user} ({created}):\n{body}")
}

/// Format a list of comments.
pub fn format_comment_list(comments: &[Value]) -> String {
    if comments.is_empty() {
        return "No comments found.".to_string();
    }
    comments
        .iter()
        .map(|c| format_comment(c))
        .collect::<Vec<_>>()
        .join("\n\n---\n\n")
}

/// Format a commit object.
pub fn format_commit(commit: &Value) -> String {
    let mut parts = Vec::new();

    let sha = commit
        .get("sha")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown");
    parts.push(format!("**Commit:** {sha}"));

    if let Some(msg) = commit
        .get("commit")
        .and_then(|v| v.get("message"))
        .and_then(|v| v.as_str())
    {
        parts.push(format!("**Message:** {msg}"));
    }

    if let Some(author) = commit
        .get("commit")
        .and_then(|v| v.get("author"))
        .and_then(|v| v.get("name"))
        .and_then(|v| v.as_str())
    {
        let date = commit
            .get("commit")
            .and_then(|v| v.get("author"))
            .and_then(|v| v.get("date"))
            .and_then(|v| v.as_str())
            .unwrap_or("");
        parts.push(format!("**Author:** {author} ({date})"));
    }

    parts.join("\n")
}

/// Format a list of commits.
pub fn format_commit_list(commits: &[Value]) -> String {
    if commits.is_empty() {
        return "No commits found.".to_string();
    }
    commits
        .iter()
        .map(|c| {
            let sha = c
                .get("sha")
                .and_then(|v| v.as_str())
                .map(|s| &s[..7.min(s.len())])
                .unwrap_or("???????");
            let msg = c
                .get("commit")
                .and_then(|v| v.get("message"))
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .lines()
                .next()
                .unwrap_or("");
            format!("- `{sha}` {msg}")
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// Format a branch object.
pub fn format_branch(branch: &Value) -> String {
    let name = branch
        .get("name")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown");
    let protected = branch
        .get("protected")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    let sha = branch
        .get("commit")
        .and_then(|v| v.get("id").or_else(|| v.get("sha")))
        .and_then(|v| v.as_str())
        .map(|s| &s[..7.min(s.len())])
        .unwrap_or("???????");
    let prot_str = if protected { " [protected]" } else { "" };
    format!("- {name} (`{sha}`){prot_str}")
}

/// Format a file content response.
pub fn format_file_content(file: &Value) -> String {
    let name = file
        .get("name")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown");
    let path = file
        .get("path")
        .and_then(|v| v.as_str())
        .unwrap_or(name);
    let file_type = file
        .get("type")
        .and_then(|v| v.as_str())
        .unwrap_or("file");

    if file_type == "dir" {
        return format!("**{path}/** (directory)");
    }

    let content = file
        .get("content")
        .and_then(|v| v.as_str())
        .unwrap_or("");

    // Gitea returns base64-encoded content
    let decoded = if !content.is_empty() {
        use base64::Engine;
        let clean = content.replace('\n', "");
        base64::engine::general_purpose::STANDARD
            .decode(&clean)
            .ok()
            .and_then(|bytes| String::from_utf8(bytes).ok())
            .unwrap_or_else(|| "(binary content)".to_string())
    } else {
        "(empty file)".to_string()
    };

    let size = file.get("size").and_then(|v| v.as_i64()).unwrap_or(0);
    let sha_line = file
        .get("sha")
        .and_then(|v| v.as_str())
        .map(|s| format!("\n**SHA:** {s}"))
        .unwrap_or_default();
    format!("**File:** {path} ({size} bytes){sha_line}\n\n```\n{decoded}\n```")
}

/// Format a directory listing.
pub fn format_file_list(entries: &[Value]) -> String {
    if entries.is_empty() {
        return "No files found.".to_string();
    }
    entries
        .iter()
        .map(|e| {
            let name = e.get("name").and_then(|v| v.as_str()).unwrap_or("?");
            let file_type = e.get("type").and_then(|v| v.as_str()).unwrap_or("file");
            let icon = if file_type == "dir" { "/" } else { "" };
            format!("- {name}{icon}")
        })
        .collect::<Vec<_>>()
        .join("\n")
}
