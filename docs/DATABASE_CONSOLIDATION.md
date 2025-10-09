# Database Consolidation Summary

## Changes Made

The LogLens project now uses a **single, unified database** located at the project root.

### Key Improvements

1. **Single Database Location**: `<project-root>/data/loglens.db`
   - All LogLens components (web, core, CLI) use the same database
   - No more confusion about multiple `data/` folders

2. **Automatic Database Creation**
   - Database and directory are created automatically on first launch
   - No need to manually set `DATABASE_URL` environment variable
   - No manual database initialization required

3. **Simplified Configuration**
   - Database path is auto-detected by finding the workspace root
   - Searches up the directory tree for `Cargo.toml` with `[workspace]`
   - Falls back to `data/loglens.db` in current directory if workspace not found

4. **Environment Variable (Optional)**
   - Only needed if you want a custom database location
   - Use `LOGLENS_DATABASE_PATH=/custom/path/to/loglens.db`
   - Default behavior works for 99% of use cases

## Technical Details

### New Module: `loglens-core/src/db_path.rs`

Provides centralized database path resolution:
- `get_database_path()` - Returns the unified database path
- `get_data_dir()` - Returns the data directory path
- `ensure_data_dir()` - Creates data directory if it doesn't exist
- `find_project_root()` - Finds workspace root by searching for Cargo.toml

### Updated Components

1. **loglens-core/src/config.rs**
   - Now uses `db_path` module for all database operations
   - Removed user home directory fallback

2. **loglens-web/src/config.rs**
   - Uses shared `db_path` module instead of `CARGO_MANIFEST_DIR`
   - Removed `DATABASE_URL` environment variable requirement
   - Optional `LOGLENS_DATABASE_PATH` for custom locations

### Removed

- `loglens-core/data/` folder
- `loglens-web/data/` folder
- Duplicate database files
- Requirement for `DATABASE_URL` environment variable

## Migration Guide

If you have existing data in the old locations:

```bash
# Move data from old location to unified location
cp loglens-web/data/loglens.db data/loglens.db

# Or from loglens-core
cp loglens-core/data/loglens.db data/loglens.db

# Remove old data folders
rm -rf loglens-web/data loglens-core/data
```

## Usage Examples

### Normal Usage (Recommended)
```bash
# Just run LogLens - database is auto-created at data/loglens.db
cargo run -p loglens-web
```

### Custom Database Location (Optional)
```bash
# Set custom database path if needed
export LOGLENS_DATABASE_PATH=/custom/path/to/db.sqlite
cargo run -p loglens-web
```

### Verification
```bash
# Verify database location
ls -lh data/loglens.db*

# Should show:
# data/loglens.db      (main database file)
# data/loglens.db-wal  (write-ahead log)
# data/loglens.db-shm  (shared memory)
```

## Benefits

1. **Simplicity**: No manual configuration required
2. **Consistency**: All components use the same database
3. **Reliability**: Automatic directory and database creation
4. **Flexibility**: Custom path available if needed
5. **Clean Structure**: One data folder, one database file

## Testing

The changes have been verified to:
- ✅ Auto-create database directory on first launch
- ✅ Use correct project root location
- ✅ Work without any environment variables
- ✅ Eliminate duplicate data folders
- ✅ Support custom database paths when needed
