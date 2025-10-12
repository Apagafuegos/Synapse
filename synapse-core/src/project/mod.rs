// Synapse project management module
//
// This module provides project initialization and management capabilities
// for the MCP integration, enabling Synapse to be deeply integrated into
// developer workflows.

pub mod config;
pub mod detect;
pub mod init;
pub mod link;
pub mod metadata;
pub mod models;
pub mod registry;
pub mod validate;

#[cfg(feature = "project-management")]
pub mod database;

#[cfg(feature = "project-management")]
pub mod queries;

// Re-export main types and functions
pub use config::ProjectConfig;
pub use detect::{detect_project_type, get_suggested_log_paths, ProjectType};
pub use init::{initialize_project, InitializationResult};
pub use link::{link_project, unlink_project, LinkResult, UnlinkResult};
pub use metadata::ProjectMetadata;
pub use models::{Analysis, AnalysisResult, AnalysisStatus, Pattern, Project};
pub use registry::{ProjectRegistry, RegistryEntry};
pub use validate::{validate_links, validate_and_repair, validate_project, ValidationReport, ProjectValidation};

#[cfg(feature = "project-management")]
pub use database::{create_pool, create_schema, initialize_database, verify_schema};

#[cfg(feature = "project-management")]
pub use queries::*;
