use std::path::PathBuf;
use std::env;

/// Find the project root by looking for .loglens directory
/// Searches from current directory upwards
fn find_project_root() -> Option<PathBuf> {
    let current_dir = env::current_dir().ok()?;
    let mut path = current_dir.as_path();

    loop {
        let loglens_dir = path.join(".loglens");
        if loglens_dir.exists() && loglens_dir.is_dir() {
            return Some(path.to_path_buf());
        }

        path = path.parent()?;
    }
}

/// Get the unified LogLens database path
///
/// **CRITICAL**: ALL LogLens installations (web, MCP, CLI) MUST use the SAME database.
///
/// Priority:
/// 1. LOGLENS_DATABASE_PATH env var (absolute path override)
/// 2. Project-local: .loglens/index.db (search upwards from cwd)
/// 3. Error - no valid database location found
///
/// This ensures web server, MCP server, and CLI all share the same database.
pub fn get_database_path() -> PathBuf {
    // Check for explicit database path override
    if let Ok(db_path) = env::var("LOGLENS_DATABASE_PATH") {
        return PathBuf::from(db_path);
    }

    // Find project root and use .loglens/index.db
    if let Some(project_root) = find_project_root() {
        return project_root.join(".loglens").join("index.db");
    }

    // FATAL: No .loglens directory found
    // User must run 'loglens init' first or set LOGLENS_DATABASE_PATH
    panic!(
        "No LogLens project found! \n\
        Please run 'loglens init' in your project directory first, \n\
        or set LOGLENS_DATABASE_PATH environment variable."
    );
}

/// Get the .loglens directory path
pub fn get_data_dir() -> PathBuf {
    // Check for explicit override
    if let Ok(db_path) = env::var("LOGLENS_DATABASE_PATH") {
        return PathBuf::from(db_path)
            .parent()
            .map(|p| p.to_path_buf())
            .unwrap_or_else(|| PathBuf::from("."));
    }

    // Find project root and return .loglens directory
    if let Some(project_root) = find_project_root() {
        return project_root.join(".loglens");
    }

    panic!(
        "No LogLens project found! \n\
        Please run 'loglens init' in your project directory first, \n\
        or set LOGLENS_DATABASE_PATH environment variable."
    );
}

/// Ensure the .loglens directory exists
pub fn ensure_data_dir() -> std::io::Result<PathBuf> {
    let data_dir = get_data_dir();
    std::fs::create_dir_all(&data_dir)?;
    Ok(data_dir)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_database_path_env_override() {
        // Test that LOGLENS_DATABASE_PATH env var takes precedence
        std::env::set_var("LOGLENS_DATABASE_PATH", "/custom/path/test.db");
        let db_path = get_database_path();
        assert_eq!(db_path, PathBuf::from("/custom/path/test.db"));
        std::env::remove_var("LOGLENS_DATABASE_PATH");
    }

    #[test]
    fn test_find_project_root() {
        let temp_dir = TempDir::new().unwrap();
        let project_root = temp_dir.path();
        let loglens_dir = project_root.join(".loglens");
        fs::create_dir_all(&loglens_dir).unwrap();

        // Change to subdirectory
        let sub_dir = project_root.join("subdir");
        fs::create_dir_all(&sub_dir).unwrap();

        // Save current dir
        let original_dir = env::current_dir().unwrap();

        // Change to subdir and test
        env::set_current_dir(&sub_dir).unwrap();
        let found_root = find_project_root();
        assert_eq!(found_root, Some(project_root.to_path_buf()));

        // Restore original dir
        env::set_current_dir(original_dir).unwrap();
    }
}
