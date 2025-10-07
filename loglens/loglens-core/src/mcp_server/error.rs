use thiserror::Error;
use rmcp::{Error as RmcpError, model::ErrorCode};

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
    #[error("Project not found: {0}")]
    ProjectNotFound(String),
    #[error("Invalid project path: {0}")]
    InvalidProjectPath(String),
    #[error("Analysis not found: {0}")]
    AnalysisNotFound(String),
    #[error("Database error: {0}")]
    DatabaseError(String),
    #[error("File not found: {0}")]
    FileNotFound(String),
}

impl From<McpError> for RmcpError {
    fn from(err: McpError) -> Self {
        match err {
            McpError::InvalidInput(msg) => {
                RmcpError::new(ErrorCode::INVALID_PARAMS, msg, None)
            }
            McpError::AnalysisFailed(msg) => {
                RmcpError::new(ErrorCode::INTERNAL_ERROR, msg, None)
            }
            McpError::ConfigurationError(msg) => {
                RmcpError::new(ErrorCode::INTERNAL_ERROR, msg, None)
            }
            McpError::InternalError(msg) => {
                RmcpError::new(ErrorCode::INTERNAL_ERROR, msg, None)
            }
            McpError::ProjectNotFound(msg) => {
                RmcpError::new(ErrorCode::INVALID_PARAMS, msg, None)
            }
            McpError::InvalidProjectPath(msg) => {
                RmcpError::new(ErrorCode::INVALID_PARAMS, msg, None)
            }
            McpError::AnalysisNotFound(msg) => {
                RmcpError::new(ErrorCode::INVALID_PARAMS, msg, None)
            }
            McpError::DatabaseError(msg) => {
                RmcpError::new(ErrorCode::INTERNAL_ERROR, msg, None)
            }
            McpError::FileNotFound(msg) => {
                RmcpError::new(ErrorCode::INVALID_PARAMS, msg, None)
            }
        }
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

#[cfg(feature = "project-management")]
impl From<sqlx::Error> for McpError {
    fn from(err: sqlx::Error) -> Self {
        McpError::DatabaseError(err.to_string())
    }
}

impl From<std::io::Error> for McpError {
    fn from(err: std::io::Error) -> Self {
        McpError::FileNotFound(err.to_string())
    }
}

// Helper methods for creating common RMCP errors
pub fn invalid_params<T: Into<String>>(msg: T) -> RmcpError {
    RmcpError::new(ErrorCode::INVALID_PARAMS, msg.into(), None)
}

pub fn internal_error<T: Into<String>>(msg: T) -> RmcpError {
    RmcpError::new(ErrorCode::INTERNAL_ERROR, msg.into(), None)
}

pub fn method_not_found<T>() -> RmcpError {
    RmcpError::new(ErrorCode::METHOD_NOT_FOUND, format!("Method not found: {}", std::any::type_name::<T>()), None)
}