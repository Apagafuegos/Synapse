use thiserror::Error;
use rmcp::model::ErrorCode;

#[derive(Error, Debug)]
pub enum McpError {
    #[error("Invalid input: {0}")]
    InvalidInput(String),
    #[error("Analysis failed: {0}")]
    AnalysisFailed(String),
    #[error("Configuration error: {0}")]
    ConfigurationError(String),
    #[error("Internal error: {0}")]
    InternalError(String),
}

impl From<McpError> for rmcp::Error {
    fn from(err: McpError) -> Self {
        rmcp::Error::new(ErrorCode::INTERNAL_ERROR, err.to_string(), None)
    }
}

impl From<anyhow::Error> for McpError {
    fn from(err: anyhow::Error) -> Self {
        McpError::InternalError(err.to_string())
    }
}

impl From<crate::ai_provider::AIError> for McpError {
    fn from(err: crate::ai_provider::AIError) -> Self {
        McpError::AnalysisFailed(err.to_string())
    }
}