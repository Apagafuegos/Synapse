use std::path::PathBuf;
use std::env;
use dirs;

/// Get the global LogLens data directory (~/.loglens/data)
fn get_global_data_dir() -> PathBuf {
    let home_dir = dirs::home_dir().unwrap_or_else(|| {
        panic!("Could not find home directory");
    });
    home_dir.join(".loglens").join("data")
}

/// Get the unified LogLens database path
///
/// **CRITICAL**: ALL LogLens installations (web, MCP, CLI) MUST use the SAME database.
///
/// Priority:
/// 1. LOGLENS_DATABASE_PATH env var (absolute path override)
/// 2. Global: ~/.loglens/data/loglens.db (ALWAYS use this)
/// 3. Error - fallback failed
///
/// This ensures web server, MCP server, and CLI all share the same database.
pub fn get_database_path() -> PathBuf {
    // Check for explicit database path override
    if let Ok(db_path) = env::var("LOGLENS_DATABASE_PATH") {
        return PathBuf::from(db_path);
    }

    // ALWAYS use the global database path at ~/.loglens/data/loglens.db
    let global_data_dir = get_global_data_dir();
    global_data_dir.join("loglens.db")
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

    // ALWAYS use the global data directory
    get_global_data_dir()
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

    #[test]
    fn test_database_path_env_override() {
        // Test that LOGLENS_DATABASE_PATH env var takes precedence
        std::env::set_var("LOGLENS_DATABASE_PATH", "/custom/path/test.db");
        let db_path = get_database_path();
        assert_eq!(db_path, PathBuf::from("/custom/path/test.db"));
        std::env::remove_var("LOGLENS_DATABASE_PATH");
    }

    #[test]
    fn test_global_database_path() {
        // Test that the database path always points to the global location
        let db_path = get_database_path();
        let expected = dirs::home_dir().unwrap().join(".loglens").join("data").join("loglens.db");
        assert_eq!(db_path, expected);
    }
}
