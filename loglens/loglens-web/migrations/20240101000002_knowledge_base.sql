-- Knowledge Base System for LogLens Phase 4

-- Error patterns table for recognizing recurring issues
CREATE TABLE error_patterns (
    id TEXT PRIMARY KEY,
    project_id TEXT NOT NULL,
    pattern TEXT NOT NULL, -- Regex pattern for matching errors
    category TEXT NOT NULL, -- Error category (code, infrastructure, config, external)
    description TEXT,
    frequency INTEGER DEFAULT 1, -- How often this pattern has been seen
    last_seen DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    suggested_solution TEXT,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE CASCADE
);

-- Knowledge base entries for common issues and solutions
CREATE TABLE knowledge_base (
    id TEXT PRIMARY KEY,
    project_id TEXT NOT NULL,
    title TEXT NOT NULL,
    problem_description TEXT NOT NULL,
    solution TEXT NOT NULL,
    tags TEXT, -- JSON array of tags
    severity TEXT DEFAULT 'medium', -- low, medium, high, critical
    is_public BOOLEAN DEFAULT FALSE, -- Whether this is shared across projects
    usage_count INTEGER DEFAULT 0, -- How many times this solution has been referenced
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE CASCADE
);

-- Error correlations table for tracking related errors
CREATE TABLE error_correlations (
    id TEXT PRIMARY KEY,
    project_id TEXT NOT NULL,
    primary_error_id TEXT NOT NULL, -- Reference to analyses table
    correlated_error_id TEXT NOT NULL, -- Reference to analyses table
    correlation_strength REAL NOT NULL, -- 0.0 to 1.0 strength
    correlation_type TEXT NOT NULL, -- temporal, causal, contextual
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE CASCADE,
    FOREIGN KEY (primary_error_id) REFERENCES analyses(id) ON DELETE CASCADE,
    FOREIGN KEY (correlated_error_id) REFERENCES analyses(id) ON DELETE CASCADE
);

-- Performance metrics table for bottleneck identification
CREATE TABLE performance_metrics (
    id TEXT PRIMARY KEY,
    analysis_id TEXT NOT NULL,
    metric_name TEXT NOT NULL, -- e.g., "response_time", "error_rate", "throughput"
    metric_value REAL NOT NULL,
    unit TEXT NOT NULL, -- e.g., "ms", "percent", "req/s"
    threshold_value REAL, -- Warning/error threshold
    is_bottleneck BOOLEAN DEFAULT FALSE,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (analysis_id) REFERENCES analyses(id) ON DELETE CASCADE
);

-- Indexes for performance
CREATE INDEX idx_error_patterns_project_id ON error_patterns(project_id);
CREATE INDEX idx_error_patterns_category ON error_patterns(category);
CREATE INDEX idx_error_patterns_frequency ON error_patterns(frequency DESC);
CREATE INDEX idx_knowledge_base_project_id ON knowledge_base(project_id);
CREATE INDEX idx_knowledge_base_severity ON knowledge_base(severity);
CREATE INDEX idx_knowledge_base_tags ON knowledge_base(tags);
CREATE INDEX idx_error_correlations_project_id ON error_correlations(project_id);
CREATE INDEX idx_error_correlations_primary_error ON error_correlations(primary_error_id);
CREATE INDEX idx_error_correlations_strength ON error_correlations(correlation_strength DESC);
CREATE INDEX idx_performance_metrics_analysis_id ON performance_metrics(analysis_id);
CREATE INDEX idx_performance_metrics_bottleneck ON performance_metrics(is_bottleneck);