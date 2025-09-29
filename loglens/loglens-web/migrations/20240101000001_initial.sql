-- Initial schema for LogLens web backend

-- Projects table
CREATE TABLE projects (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    description TEXT,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Log files table
CREATE TABLE log_files (
    id TEXT PRIMARY KEY,
    project_id TEXT NOT NULL,
    filename TEXT NOT NULL,
    file_size INTEGER NOT NULL,
    line_count INTEGER NOT NULL,
    upload_path TEXT NOT NULL,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE CASCADE
);

-- Analysis results table
CREATE TABLE analyses (
    id TEXT PRIMARY KEY,
    project_id TEXT NOT NULL,
    log_file_id TEXT,
    analysis_type TEXT NOT NULL, -- 'file' or 'realtime'
    provider TEXT NOT NULL,
    level_filter TEXT NOT NULL,
    status INTEGER NOT NULL DEFAULT 0, -- 0=pending, 1=running, 2=completed, 3=failed
    result TEXT, -- JSON serialized AnalysisResponse
    error_message TEXT,
    started_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    completed_at DATETIME,
    FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE CASCADE,
    FOREIGN KEY (log_file_id) REFERENCES log_files(id) ON DELETE SET NULL
);

-- Indexes for performance
CREATE INDEX idx_log_files_project_id ON log_files(project_id);
CREATE INDEX idx_analyses_project_id ON analyses(project_id);
CREATE INDEX idx_analyses_log_file_id ON analyses(log_file_id);
CREATE INDEX idx_analyses_status ON analyses(status);
CREATE INDEX idx_analyses_started_at ON analyses(started_at DESC);