use rmcp::model::{ErrorCode, ErrorData};

/// All error types produced by the gitx-mcp server.
#[derive(Debug, thiserror::Error)]
pub enum GitxError {
    #[error("API request failed: {0}")]
    Api(String),

    #[error("Authentication failed — check your API token")]
    Auth,

    #[error("Resource not found: {0}")]
    NotFound(String),

    #[error("Missing required parameter: {0}")]
    MissingParam(String),

    #[error("Could not resolve repository from directory: {0}")]
    RepoResolution(String),

    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("{0}")]
    Other(#[from] anyhow::Error),
}

impl From<GitxError> for ErrorData {
    fn from(err: GitxError) -> Self {
        let code = match &err {
            GitxError::MissingParam(_) => ErrorCode::INVALID_PARAMS,
            GitxError::NotFound(_) => ErrorCode::INVALID_PARAMS,
            GitxError::Auth => ErrorCode::INVALID_PARAMS,
            _ => ErrorCode::INTERNAL_ERROR,
        };
        ErrorData::new(code, err.to_string(), None)
    }
}

pub type Result<T> = std::result::Result<T, GitxError>;
