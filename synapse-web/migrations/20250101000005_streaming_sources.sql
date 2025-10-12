-- Migration: Add streaming sources and streaming logs tables
-- Date: 2025-01-01
-- Description: Tables for managing real-time log streaming sources and buffered logs

-- Store streaming source configurations
CREATE TABLE IF NOT EXISTS streaming_sources (
    id TEXT PRIMARY KEY,
    project_id TEXT NOT NULL,
    name TEXT NOT NULL,
    source_type TEXT NOT NULL,
    config TEXT NOT NULL, -- JSON
    parser_config TEXT, -- JSON
    buffer_size INTEGER DEFAULT 100,
    batch_timeout_seconds INTEGER DEFAULT 2,
    restart_on_error BOOLEAN DEFAULT true,
    max_restarts INTEGER,
    status TEXT DEFAULT 'active',
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    FOREIGN KEY (project_id) REFERENCES projects(id)
);

CREATE INDEX idx_streaming_sources_project ON streaming_sources(project_id);
CREATE INDEX idx_streaming_sources_status ON streaming_sources(status);

-- Store recent streaming logs for filtering/querying
CREATE TABLE IF NOT EXISTS streaming_logs (
    id TEXT PRIMARY KEY,
    project_id TEXT NOT NULL,
    source_id TEXT NOT NULL,
    timestamp TEXT NOT NULL,
    level TEXT,
    message TEXT NOT NULL,
    source TEXT NOT NULL,
    line_number INTEGER,
    created_at TEXT NOT NULL,
    FOREIGN KEY (project_id) REFERENCES projects(id),
    FOREIGN KEY (source_id) REFERENCES streaming_sources(id)
);

CREATE INDEX idx_streaming_logs_project ON streaming_logs(project_id);
CREATE INDEX idx_streaming_logs_source ON streaming_logs(source_id);
CREATE INDEX idx_streaming_logs_timestamp ON streaming_logs(timestamp);
CREATE INDEX idx_streaming_logs_level ON streaming_logs(level);
