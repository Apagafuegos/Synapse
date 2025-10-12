use std::path::PathBuf;
use std::env;
use dirs;

/// Get the global Synapse data directory (~/.synapse/data)
fn get_global_data_dir() -> PathBuf {
    let home_dir = dirs::home_dir().unwrap_or_else(|| {
        panic!("Could not find home directory");
    });
    home_dir.join(".synapse").join("data")
}

/// Get the unified Synapse database path
///
/// **CRITICAL**: ALL Synapse installations (web, MCP, CLI) MUST use the SAME database.
///
/// Priority:
/// 1. SYNAPSE_DATABASE_PATH env var (absolute path override)
/// 2. Global: ~/.synapse/data/synapse.db (ALWAYS use this)
/// 3. Error - fallback failed
///
/// This ensures web server, MCP server, and CLI all share the same database.
pub fn get_database_path() -> PathBuf {
    // Check for explicit database path override
    if let Ok(db_path) = env::var("SYNAPSE_DATABASE_PATH") {
        return PathBuf::from(db_path);
    }

    // ALWAYS use the global database path at ~/.synapse/data/synapse.db
    let global_data_dir = get_global_data_dir();
    global_data_dir.join("synapse.db")
}

/// Get the .synapse directory path
pub fn get_data_dir() -> PathBuf {
    // Check for explicit override
    if let Ok(db_path) = env::var("SYNAPSE_DATABASE_PATH") {
        return PathBuf::from(db_path)
            .parent()
            .map(|p| p.to_path_buf())
            .unwrap_or_else(|| PathBuf::from("."));
    }

    // ALWAYS use the global data directory
    get_global_data_dir()
}

/// Ensure the .synapse directory exists
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
        // Test that SYNAPSE_DATABASE_PATH env var takes precedence
        std::env::set_var("SYNAPSE_DATABASE_PATH", "/custom/path/test.db");
        let db_path = get_database_path();
        assert_eq!(db_path, PathBuf::from("/custom/path/test.db"));
        std::env::remove_var("SYNAPSE_DATABASE_PATH");
    }

    #[test]
    fn test_global_database_path() {
        // Test that the database path always points to the global location
        let db_path = get_database_path();
        let expected = dirs::home_dir().unwrap().join(".synapse").join("data").join("synapse.db");
        assert_eq!(db_path, expected);
    }
}
