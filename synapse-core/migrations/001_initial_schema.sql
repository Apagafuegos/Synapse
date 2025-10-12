-- Initial schema for LogLens project management and analysis tracking
-- This schema supports the MCP integration plan for project-linked log analysis

-- Enable WAL mode for better concurrency
PRAGMA journal_mode = WAL;

-- Projects table: Track initialized LogLens projects
CREATE TABLE IF NOT EXISTS projects (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    root_path TEXT UNIQUE NOT NULL,  -- absolute path to project root
    description TEXT,
    created_at TIMESTAMP NOT NULL,
    updated_at TIMESTAMP NOT NULL,
    metadata TEXT                      -- JSON object with additional metadata
);

-- Analyses tracking: Log analysis operations for projects
CREATE TABLE IF NOT EXISTS analyses (
    id TEXT PRIMARY KEY,
    project_id TEXT NOT NULL,
    log_file_path TEXT NOT NULL,     -- path to analyzed log file
    provider TEXT NOT NULL,            -- openrouter, openai, claude, gemini
    level TEXT NOT NULL,               -- ERROR, WARN, INFO, DEBUG
    status TEXT NOT NULL,              -- pending, completed, failed
    created_at TIMESTAMP NOT NULL,
    started_at TIMESTAMP,
    completed_at TIMESTAMP,
    metadata TEXT,                     -- JSON object with additional metadata
    FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE CASCADE
);

-- Analysis results storage: Detailed analysis output
CREATE TABLE IF NOT EXISTS analysis_results (
    analysis_id TEXT PRIMARY KEY,
    summary TEXT,
    full_report TEXT,
    patterns_detected TEXT,           -- JSON array of pattern objects
    issues_found INTEGER,
    metadata TEXT,                     -- JSON object with additional metadata
    FOREIGN KEY (analysis_id) REFERENCES analyses(id) ON DELETE CASCADE
);

-- Performance indexes for common queries
CREATE INDEX IF NOT EXISTS idx_analyses_project ON analyses(project_id);
CREATE INDEX IF NOT EXISTS idx_analyses_status ON analyses(status);
CREATE INDEX IF NOT EXISTS idx_analyses_created ON analyses(created_at DESC);
CREATE INDEX IF NOT EXISTS idx_projects_root_path ON projects(root_path);
