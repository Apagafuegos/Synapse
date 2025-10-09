use std::path::PathBuf;
use std::env;

/// Get the unified LogLens database path following XDG Base Directory specification
/// Priority: LOGLENS_DATABASE_PATH env var > XDG data directory > fallback
///
/// On different platforms:
/// - Linux: ~/.local/share/loglens/loglens.db
/// - macOS: ~/Library/Application Support/loglens/loglens.db
/// - Windows: %APPDATA%\loglens\loglens.db
pub fn get_database_path() -> PathBuf {
    // Check for explicit database path override
    if let Ok(db_path) = env::var("LOGLENS_DATABASE_PATH") {
        return PathBuf::from(db_path);
    }

    // Use XDG Base Directory specification via directories crate
    if let Some(proj_dirs) = directories::ProjectDirs::from("com", "loglens", "LogLens") {
        return proj_dirs.data_dir().join("loglens.db");
    }

    // Fallback: use current directory (should rarely happen)
    PathBuf::from("loglens.db")
}

/// Get the data directory path
pub fn get_data_dir() -> PathBuf {
    // Check for explicit override
    if let Ok(db_path) = env::var("LOGLENS_DATABASE_PATH") {
        return PathBuf::from(db_path)
            .parent()
            .map(|p| p.to_path_buf())
            .unwrap_or_else(|| PathBuf::from("."));
    }

    // Use XDG Base Directory specification
    if let Some(proj_dirs) = directories::ProjectDirs::from("com", "loglens", "LogLens") {
        return proj_dirs.data_dir().to_path_buf();
    }

    // Fallback
    PathBuf::from(".")
}

/// Ensure the data directory exists
pub fn ensure_data_dir() -> std::io::Result<PathBuf> {
    let data_dir = get_data_dir();
    std::fs::create_dir_all(&data_dir)?;
    Ok(data_dir)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_database_path() {
        let db_path = get_database_path();
        // Should end with loglens.db regardless of platform
        assert!(db_path.to_str().unwrap().ends_with("loglens.db"));
    }

    #[test]
    fn test_get_data_dir() {
        let data_dir = get_data_dir();
        // Should return a valid path
        assert!(!data_dir.to_str().unwrap().is_empty());
    }

    #[test]
    fn test_database_path_env_override() {
        // Test that LOGLENS_DATABASE_PATH env var takes precedence
        std::env::set_var("LOGLENS_DATABASE_PATH", "/custom/path/test.db");
        let db_path = get_database_path();
        assert_eq!(db_path, PathBuf::from("/custom/path/test.db"));
        std::env::remove_var("LOGLENS_DATABASE_PATH");
    }
}
