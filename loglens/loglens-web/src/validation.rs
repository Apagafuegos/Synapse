use axum::http::StatusCode;
use regex::Regex;
use std::collections::HashSet;

/// Maximum allowed length for project names
const MAX_PROJECT_NAME_LENGTH: usize = 100;
/// Maximum allowed length for project descriptions  
const MAX_PROJECT_DESCRIPTION_LENGTH: usize = 1000;
/// Maximum allowed length for user context
const MAX_USER_CONTEXT_LENGTH: usize = 5000;
/// Maximum allowed filename length
const MAX_FILENAME_LENGTH: usize = 255;

/// Allowed file extensions for log uploads
const ALLOWED_EXTENSIONS: &[&str] = &[
    "log", "txt", "json", "xml", "csv", "tsv", "out", "err", "trace", "py", "python", "logs", "debug"
];

/// Allowed AI providers
const ALLOWED_PROVIDERS: &[&str] = &[
    "openrouter", "openai", "claude", "gemini"
];

/// Allowed log levels
const ALLOWED_LOG_LEVELS: &[&str] = &[
    "DEBUG", "INFO", "WARN", "ERROR", "debug", "info", "warn", "error"
];

#[derive(Debug, Clone)]
pub enum ValidationError {
    ProjectNameTooLong(usize),
    ProjectNameEmpty,
    ProjectNameInvalid(String),
    ProjectDescriptionTooLong(usize),
    UserContextTooLong(usize),
    FilenameTooLong(usize),
    FilenameEmpty,
    FilenameInvalid(String),
    FileExtensionNotAllowed(String),
    FileTooLarge(String),
    ProviderNotAllowed(String),
    LogLevelInvalid(String),
    InvalidUuid(String),
}

impl ValidationError {
    pub fn to_status_code(&self) -> StatusCode {
        match self {
            ValidationError::FileTooLarge(_) => StatusCode::PAYLOAD_TOO_LARGE, // 413
            ValidationError::FilenameInvalid(_) => StatusCode::UNPROCESSABLE_ENTITY, // 422
            ValidationError::FileExtensionNotAllowed(_) => StatusCode::UNSUPPORTED_MEDIA_TYPE, // 415
            _ => StatusCode::BAD_REQUEST, // 400
        }
    }

    pub fn to_message(&self) -> String {
        match self {
            ValidationError::ProjectNameTooLong(len) => {
                format!("Project name too long: {} characters (max {})", len, MAX_PROJECT_NAME_LENGTH)
            }
            ValidationError::ProjectNameEmpty => "Project name cannot be empty".to_string(),
            ValidationError::ProjectNameInvalid(name) => {
                format!("Project name contains invalid characters: '{}'", name)
            }
            ValidationError::ProjectDescriptionTooLong(len) => {
                format!("Project description too long: {} characters (max {})", len, MAX_PROJECT_DESCRIPTION_LENGTH)
            }
            ValidationError::UserContextTooLong(len) => {
                format!("User context too long: {} characters (max {})", len, MAX_USER_CONTEXT_LENGTH)
            }
            ValidationError::FilenameTooLong(len) => {
                format!("Filename too long: {} characters (max {})", len, MAX_FILENAME_LENGTH)
            }
            ValidationError::FilenameEmpty => "Filename cannot be empty".to_string(),
            ValidationError::FilenameInvalid(name) => {
                format!("Filename contains invalid characters: '{}'", name)
            }
            ValidationError::FileExtensionNotAllowed(ext) => {
                format!("File extension '{}' not allowed. Allowed: {}", ext, ALLOWED_EXTENSIONS.join(", "))
            }
            ValidationError::FileTooLarge(msg) => {
                msg.clone()
            }
            ValidationError::ProviderNotAllowed(provider) => {
                format!("AI provider '{}' not allowed. Allowed: {}", provider, ALLOWED_PROVIDERS.join(", "))
            }
            ValidationError::LogLevelInvalid(level) => {
                format!("Log level '{}' invalid. Allowed: {}", level, ALLOWED_LOG_LEVELS.join(", "))
            }
            ValidationError::InvalidUuid(id) => {
                format!("Invalid UUID format: '{}'", id)
            }
        }
    }
}

pub type ValidationResult<T> = Result<T, ValidationError>;

pub struct Validator;

impl Validator {
    /// Validate project name
    pub fn validate_project_name(name: &str) -> ValidationResult<()> {
        if name.is_empty() {
            return Err(ValidationError::ProjectNameEmpty);
        }
        
        if name.len() > MAX_PROJECT_NAME_LENGTH {
            return Err(ValidationError::ProjectNameTooLong(name.len()));
        }

        // Allow alphanumeric, spaces, hyphens, underscores, periods
        let valid_chars = Regex::new(r"^[a-zA-Z0-9\s\-_.]+$").unwrap();
        if !valid_chars.is_match(name) {
            return Err(ValidationError::ProjectNameInvalid(name.to_string()));
        }

        Ok(())
    }

    /// Validate project description
    pub fn validate_project_description(description: Option<&String>) -> ValidationResult<()> {
        if let Some(desc) = description {
            if desc.len() > MAX_PROJECT_DESCRIPTION_LENGTH {
                return Err(ValidationError::ProjectDescriptionTooLong(desc.len()));
            }
        }
        Ok(())
    }

    /// Validate user context
    pub fn validate_user_context(context: Option<&String>) -> ValidationResult<()> {
        if let Some(ctx) = context {
            if ctx.len() > MAX_USER_CONTEXT_LENGTH {
                return Err(ValidationError::UserContextTooLong(ctx.len()));
            }
        }
        Ok(())
    }

    /// Validate filename
    pub fn validate_filename(filename: &str) -> ValidationResult<()> {
        if filename.is_empty() {
            return Err(ValidationError::FilenameEmpty);
        }

        if filename.len() > MAX_FILENAME_LENGTH {
            return Err(ValidationError::FilenameTooLong(filename.len()));
        }

        // Check for dangerous characters (removed ':' as it's common in log filenames)
        let dangerous_chars = ['/', '\\', '*', '?', '"', '<', '>', '|', '\0'];
        if filename.chars().any(|c| dangerous_chars.contains(&c)) {
            return Err(ValidationError::FilenameInvalid(filename.to_string()));
        }

        // Check file extension if present
        if let Some(extension) = std::path::Path::new(filename).extension() {
            if let Some(ext_str) = extension.to_str() {
                let allowed: HashSet<&str> = ALLOWED_EXTENSIONS.iter().cloned().collect();
                if !allowed.contains(ext_str.to_lowercase().as_str()) {
                    return Err(ValidationError::FileExtensionNotAllowed(ext_str.to_string()));
                }
            }
        }

        Ok(())
    }

    /// Validate AI provider
    pub fn validate_provider(provider: &str) -> ValidationResult<()> {
        let allowed: HashSet<&str> = ALLOWED_PROVIDERS.iter().cloned().collect();
        if !allowed.contains(provider) {
            return Err(ValidationError::ProviderNotAllowed(provider.to_string()));
        }
        Ok(())
    }

    /// Validate log level
    pub fn validate_log_level(level: &str) -> ValidationResult<()> {
        let allowed: HashSet<&str> = ALLOWED_LOG_LEVELS.iter().cloned().collect();
        if !allowed.contains(level) {
            return Err(ValidationError::LogLevelInvalid(level.to_string()));
        }
        Ok(())
    }

    /// Validate UUID format
    pub fn validate_uuid(id: &str) -> ValidationResult<()> {
        uuid::Uuid::parse_str(id)
            .map_err(|_| ValidationError::InvalidUuid(id.to_string()))?;
        Ok(())
    }

    /// Sanitize text input by removing dangerous characters and normalizing whitespace
    pub fn sanitize_text(input: &str) -> String {
        // Remove null bytes and control characters except newlines and tabs
        let clean: String = input
            .chars()
            .filter(|c| !c.is_control() || *c == '\n' || *c == '\t')
            .collect();
        
        // Normalize whitespace
        clean.trim().to_string()
    }

    /// Validate and sanitize project creation request
    pub fn validate_project_request(name: &str, description: Option<&String>) -> ValidationResult<(String, Option<String>)> {
        let sanitized_name = Self::sanitize_text(name);
        Self::validate_project_name(&sanitized_name)?;
        
        let sanitized_description = description.map(|d| Self::sanitize_text(d));
        Self::validate_project_description(sanitized_description.as_ref())?;
        
        Ok((sanitized_name, sanitized_description))
    }

    /// Validate and sanitize analysis request
    pub fn validate_analysis_request(
        provider: &str,
        level: &str,
        user_context: Option<&String>
    ) -> ValidationResult<(String, String, Option<String>)> {
        let sanitized_provider = Self::sanitize_text(provider);
        Self::validate_provider(&sanitized_provider)?;
        
        let sanitized_level = Self::sanitize_text(level);
        Self::validate_log_level(&sanitized_level)?;
        
        let sanitized_context = user_context.map(|c| Self::sanitize_text(c));
        Self::validate_user_context(sanitized_context.as_ref())?;
        
        Ok((sanitized_provider, sanitized_level, sanitized_context))
    }

    /// Validate file upload
    pub fn validate_file_upload(filename: &str, file_size: usize, max_size: usize) -> ValidationResult<String> {
        let sanitized_filename = Self::sanitize_text(filename);
        Self::validate_filename(&sanitized_filename)?;
        
        if file_size > max_size {
            return Err(ValidationError::FileTooLarge(
                format!("File too large: {} bytes (max {})", file_size, max_size)
            ));
        }
        
        Ok(sanitized_filename)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_project_name_validation() {
        assert!(Validator::validate_project_name("Valid Project Name").is_ok());
        assert!(Validator::validate_project_name("project_123-test.log").is_ok());
        
        assert!(Validator::validate_project_name("").is_err());
        assert!(Validator::validate_project_name(&"a".repeat(200)).is_err());
        assert!(Validator::validate_project_name("invalid/name").is_err());
        assert!(Validator::validate_project_name("invalid:name").is_err());
    }

    #[test]
    fn test_filename_validation() {
        assert!(Validator::validate_filename("test.log").is_ok());
        assert!(Validator::validate_filename("application.out").is_ok());
        assert!(Validator::validate_filename("data.json").is_ok());
        assert!(Validator::validate_filename("logFile.2025-08-26.0.log").is_ok()); // Allow colons in timestamps

        assert!(Validator::validate_filename("").is_err());
        assert!(Validator::validate_filename("test.exe").is_err());
        assert!(Validator::validate_filename("../../../etc/passwd").is_err());
        assert!(Validator::validate_filename("test?file.log").is_err()); // Question marks still dangerous
    }

    #[test]
    fn test_provider_validation() {
        assert!(Validator::validate_provider("openrouter").is_ok());
        assert!(Validator::validate_provider("openai").is_ok());
        
        assert!(Validator::validate_provider("invalid").is_err());
        assert!(Validator::validate_provider("").is_err());
    }

    #[test]
    fn test_uuid_validation() {
        assert!(Validator::validate_uuid("550e8400-e29b-41d4-a716-446655440000").is_ok());
        
        assert!(Validator::validate_uuid("not-a-uuid").is_err());
        assert!(Validator::validate_uuid("").is_err());
    }
}