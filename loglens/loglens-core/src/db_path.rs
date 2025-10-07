use std::path::PathBuf;
use std::env;

/// Find the LogLens project root directory by looking for Cargo.toml with workspace definition
fn find_project_root() -> Option<PathBuf> {
    // Start from current directory
    let mut current = env::current_dir().ok()?;

    loop {
        let cargo_toml = current.join("Cargo.toml");
        if cargo_toml.exists() {
            // Check if this is the workspace root by looking for [workspace] in Cargo.toml
            if let Ok(content) = std::fs::read_to_string(&cargo_toml) {
                if content.contains("[workspace]") {
                    return Some(current);
                }
            }
        }

        // Move up to parent directory
        if !current.pop() {
            break;
        }
    }

    None
}

/// Get the unified LogLens database path
/// Priority: LOGLENS_DATABASE_PATH env var > project_root/data/loglens.db
pub fn get_database_path() -> PathBuf {
    // Check for explicit database path override
    if let Ok(db_path) = env::var("LOGLENS_DATABASE_PATH") {
        return PathBuf::from(db_path);
    }

    // Try to find project root
    if let Some(project_root) = find_project_root() {
        return project_root.join("data").join("loglens.db");
    }

    // Fallback: use current directory
    PathBuf::from("data/loglens.db")
}

/// Get the data directory path
pub fn get_data_dir() -> PathBuf {
    get_database_path()
        .parent()
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| PathBuf::from("data"))
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
        assert!(db_path.to_str().unwrap().ends_with("data/loglens.db"));
    }

    #[test]
    fn test_get_data_dir() {
        let data_dir = get_data_dir();
        assert!(data_dir.to_str().unwrap().ends_with("data"));
    }
}
