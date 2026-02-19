use rmcp::model::{ErrorCode, ErrorData};

/// All error types produced by the gitx-mcp server.
#[derive(Debug, thiserror::Error)]
pub enum GiteaError {
    #[error("API request failed: {0}")]
    Api(String),

    #[error("Authentication failed — check GITEA_TOKEN")]
    Auth,

    #[error("Resource not found: {0}")]
    NotFound(String),

    #[error("Missing required parameter: {0}")]
    MissingParam(String),

    #[error("Could not resolve repository from directory: {0}")]
    RepoResolution(String),

    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("{0}")]
    Other(#[from] anyhow::Error),
}

impl From<GiteaError> for ErrorData {
    fn from(err: GiteaError) -> Self {
        let code = match &err {
            GiteaError::MissingParam(_) => ErrorCode::INVALID_PARAMS,
            GiteaError::NotFound(_) => ErrorCode::INVALID_PARAMS,
            GiteaError::Auth => ErrorCode::INVALID_PARAMS,
            _ => ErrorCode::INTERNAL_ERROR,
        };
        ErrorData::new(code, err.to_string(), None)
    }
}

pub type Result<T> = std::result::Result<T, GiteaError>;
