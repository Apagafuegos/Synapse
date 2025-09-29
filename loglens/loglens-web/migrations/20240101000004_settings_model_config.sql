-- Add model configuration and timeout fields to settings table
ALTER TABLE settings ADD COLUMN selected_model TEXT;
ALTER TABLE settings ADD COLUMN available_models TEXT; -- JSON array cache
ALTER TABLE settings ADD COLUMN models_last_fetched DATETIME;
ALTER TABLE settings ADD COLUMN analysis_timeout_seconds INTEGER DEFAULT 300;

-- Update existing row with default values
UPDATE settings SET
    selected_model = NULL,
    available_models = NULL,
    models_last_fetched = NULL,
    analysis_timeout_seconds = 300
WHERE id = 1;