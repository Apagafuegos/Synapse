-- Add CLI project fields to projects table for unified project management

-- Add columns for CLI project integration
ALTER TABLE projects ADD COLUMN root_path TEXT;
ALTER TABLE projects ADD COLUMN loglens_config TEXT;
ALTER TABLE projects ADD COLUMN last_accessed DATETIME;
ALTER TABLE projects ADD COLUMN project_type TEXT DEFAULT 'unknown';

-- Create index for faster lookups by path
CREATE INDEX idx_projects_root_path ON projects(root_path);
CREATE INDEX idx_projects_last_accessed ON projects(last_accessed DESC);
