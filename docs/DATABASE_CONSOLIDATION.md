# Database Consolidation Summary

## Changes Made

The Synapse project now uses a **single, unified database** located at the project root.

### Key Improvements

1. **Single Database Location**: `<project-root>/data/synapse.db`
   - All Synapse components (web, core, CLI) use the same database
   - No more confusion about multiple `data/` folders

2. **Automatic Database Creation**
   - Database and directory are created automatically on first launch
   - No need to manually set `DATABASE_URL` environment variable
   - No manual database initialization required

3. **Simplified Configuration**
   - Database path is auto-detected by finding the workspace root
   - Searches up the directory tree for `Cargo.toml` with `[workspace]`
   - Falls back to `data/synapse.db` in current directory if workspace not found

4. **Environment Variable (Optional)**
   - Only needed if you want a custom database location
   - Use `SYNAPSE_DATABASE_PATH=/custom/path/to/synapse.db`
   - Default behavior works for 99% of use cases

## Technical Details

### New Module: `synapse-core/src/db_path.rs`

Provides centralized database path resolution:
- `get_database_path()` - Returns the unified database path
- `get_data_dir()` - Returns the data directory path
- `ensure_data_dir()` - Creates data directory if it doesn't exist
- `find_project_root()` - Finds workspace root by searching for Cargo.toml

### Updated Components

1. **synapse-core/src/config.rs**
   - Now uses `db_path` module for all database operations
   - Removed user home directory fallback

2. **synapse-web/src/config.rs**
   - Uses shared `db_path` module instead of `CARGO_MANIFEST_DIR`
   - Removed `DATABASE_URL` environment variable requirement
   - Optional `SYNAPSE_DATABASE_PATH` for custom locations

### Removed

- `synapse-core/data/` folder
- `synapse-web/data/` folder
- Duplicate database files
- Requirement for `DATABASE_URL` environment variable

## Migration Guide

If you have existing data in the old locations:

```bash
# Move data from old location to unified location
cp synapse-web/data/synapse.db data/synapse.db

# Or from synapse-core
cp synapse-core/data/synapse.db data/synapse.db

# Remove old data folders
rm -rf synapse-web/data synapse-core/data
```

## Usage Examples

### Normal Usage (Recommended)
```bash
# Just run Synapse - database is auto-created at data/synapse.db
cargo run -p synapse-web
```

### Custom Database Location (Optional)
```bash
# Set custom database path if needed
export SYNAPSE_DATABASE_PATH=/custom/path/to/db.sqlite
cargo run -p synapse-web
```

### Verification
```bash
# Verify database location
ls -lh data/synapse.db*

# Should show:
# data/synapse.db      (main database file)
# data/synapse.db-wal  (write-ahead log)
# data/synapse.db-shm  (shared memory)
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
