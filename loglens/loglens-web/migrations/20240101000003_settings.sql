-- Settings table for storing user preferences and configuration
CREATE TABLE settings (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    default_provider TEXT NOT NULL DEFAULT 'openai',
    api_key TEXT NOT NULL DEFAULT '',
    max_lines INTEGER NOT NULL DEFAULT 1000,
    default_level TEXT NOT NULL DEFAULT 'INFO',
    show_timestamps BOOLEAN NOT NULL DEFAULT 1,
    show_line_numbers BOOLEAN NOT NULL DEFAULT 1,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Insert default settings row
INSERT INTO settings (id, default_provider, api_key, max_lines, default_level, show_timestamps, show_line_numbers)
VALUES (1, 'openrouter', '', 1000, 'ERROR', 1, 1);

-- Index for performance
CREATE INDEX idx_settings_id ON settings(id);