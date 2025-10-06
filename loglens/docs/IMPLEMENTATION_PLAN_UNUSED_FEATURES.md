# LogLens - Unused Features Implementation Plan

**Date Created**: 2025-10-04
**Status**: Planning Phase
**Estimated Total Effort**: 4-6 days (32-48 hours)

---

## Executive Summary

This document outlines the comprehensive plan to implement/delete unused features identified in the LogLens codebase. The plan includes 8 major categories affecting both backend and frontend, totaling approximately 1,382 lines of code requiring action.

**Key Decisions**:
- ✅ **IMPLEMENT**: 5 categories (WebSocket Analysis, Streaming Sources, Model Fields, Advanced Analytics)
- ❌ **DELETE**: 3 categories (Old code, replaced functions, unused helpers)

---

## Phase 1: Cleanup & Deletions (Priority: High)

### Task 1.1: Delete Old WebSocket Helper Functions
**Category**: 1.3 - Deprecated Code
**Location**: `loglens-web/src/handlers/websocket.rs`
**Effort**: 15 minutes
**Lines Removed**: ~30 lines

**Actions**:
1. Delete `send_progress()` function (lines 367-387)
2. Delete `send_cancelled()` function (lines 426-436)
3. Verify no references to these functions exist
4. Run tests to ensure _split variants work correctly

**Rationale**: These functions are superseded by `send_progress_split()` and `send_cancelled_split()`. Keeping both creates maintenance burden and confusion.

---

### Task 1.2: Delete Database create_tables() Method
**Category**: 3.1 - Replaced Database Functions
**Location**: `loglens-web/src/database.rs:44-93`
**Effort**: 10 minutes
**Lines Removed**: ~50 lines

**Actions**:
1. Delete `create_tables()` method from Database impl
2. Verify migration system handles all table creation
3. Update any documentation references
4. Remove from lib.rs if publicly exposed

**Rationale**: SQLx migrations provide version control, rollback capability, and team coordination. Manual table creation is obsolete.

---

## Phase 2: Backend Foundation (Priority: High)

### Task 2.1: Wire Up WebSocket Analysis Progress Endpoint
**Category**: 1.1 - WebSocket Analysis Feature
**Location**: `loglens-web/src/routes.rs`, `loglens-web/src/handlers/websocket.rs`
**Effort**: 30 minutes
**Lines Added**: ~5 lines

**Actions**:
1. Add route to `routes.rs`:
   ```rust
   .route(
       "/projects/:project_id/files/:file_id/analyze/ws",
       get(handlers::websocket::websocket_analysis_handler)
   )
   ```
2. Export handler from `handlers/mod.rs` (if not already)
3. Test WebSocket connection with manual curl/websocat
4. Verify cancellation works via WebSocket message
5. Test database persistence of analysis results

**Files Modified**:
- `loglens-web/src/routes.rs` (+5 lines)
- `loglens-web/src/handlers/mod.rs` (+1 line if needed)

**Testing Checklist**:
- [ ] WebSocket connects successfully
- [ ] Progress messages received during analysis
- [ ] Cancellation via "cancel" message works
- [ ] Analysis stored in database on completion
- [ ] Error messages sent on failure
- [ ] Connection close handled gracefully

---

### Task 2.2: Implement Advanced Analytics Query Functions
**Category**: 3.2 - Database Query Functions
**Location**: New file `loglens-web/src/handlers/analytics.rs`
**Effort**: 2 hours
**Lines Added**: ~200 lines

**Actions**:

#### 2.2.1: Create Analytics Handler Module
Create `loglens-web/src/handlers/analytics.rs`:

```rust
use axum::{extract::{Path, State}, Json};
use crate::{AppState, AppError, models::{PerformanceMetric, ErrorCorrelation}};

/// GET /api/analyses/:id/performance-metrics
pub async fn get_performance_metrics(
    Path(analysis_id): Path<String>,
    State(state): State<AppState>,
) -> Result<Json<Vec<PerformanceMetric>>, AppError> {
    // Query from performance_metrics table
    let metrics = sqlx::query_as::<_, PerformanceMetric>(
        "SELECT * FROM performance_metrics WHERE analysis_id = ? ORDER BY created_at DESC"
    )
    .bind(&analysis_id)
    .fetch_all(state.db.pool())
    .await?;

    Ok(Json(metrics))
}

/// GET /api/projects/:id/error-correlations
pub async fn get_error_correlations(
    Path(project_id): Path<String>,
    State(state): State<AppState>,
) -> Result<Json<Vec<ErrorCorrelation>>, AppError> {
    // Query from error_correlations table
    let correlations = sqlx::query_as::<_, ErrorCorrelation>(
        "SELECT * FROM error_correlations
         WHERE project_id = ?
         ORDER BY correlation_strength DESC
         LIMIT 50"
    )
    .bind(&project_id)
    .fetch_all(state.db.pool())
    .await?;

    Ok(Json(correlations))
}

/// Helper: Create performance metric record
pub async fn create_performance_metric(
    state: &AppState,
    metric: PerformanceMetric,
) -> Result<(), AppError> {
    sqlx::query!(
        "INSERT INTO performance_metrics
         (id, analysis_id, metric_name, metric_value, unit, threshold_value, is_bottleneck, created_at)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
        metric.id,
        metric.analysis_id,
        metric.metric_name,
        metric.metric_value,
        metric.unit,
        metric.threshold_value,
        metric.is_bottleneck,
        metric.created_at
    )
    .execute(state.db.pool())
    .await?;

    Ok(())
}
```

#### 2.2.2: Add Routes
Update `loglens-web/src/routes.rs`:
```rust
.route("/analyses/:id/performance-metrics", get(handlers::analytics::get_performance_metrics))
.route("/projects/:id/error-correlations", get(handlers::analytics::get_error_correlations))
```

#### 2.2.3: Export Module
Update `loglens-web/src/handlers/mod.rs`:
```rust
pub mod analytics;
pub use analytics::{get_performance_metrics, get_error_correlations, create_performance_metric};
```

**Files Created**:
- `loglens-web/src/handlers/analytics.rs` (~200 lines)

**Files Modified**:
- `loglens-web/src/routes.rs` (+2 lines)
- `loglens-web/src/handlers/mod.rs` (+2 lines)

---

### Task 2.3: Implement Cache Helper for Project Analyses
**Category**: 3.3 - Cache Integration
**Location**: `loglens-web/src/cache.rs`, `loglens-web/src/handlers/analysis.rs`
**Effort**: 45 minutes
**Lines Modified**: ~30 lines

**Actions**:

#### 2.3.1: Use Existing Cache Key Function
The function `CacheKeys::project_analyses_key()` already exists but is unused. Integrate it into analysis listing.

#### 2.3.2: Update List Analyses Handler
Modify `loglens-web/src/handlers/analysis.rs` (or wherever `list_analyses` is):

```rust
pub async fn list_analyses(
    Path(project_id): Path<String>,
    State(state): State<AppState>,
) -> Result<Json<AnalysisListResponse>, AppError> {
    // Try cache first
    let cache_key = CacheKeys::project_analyses_key(&project_id);

    if let Some(cached) = state.cache_manager.result_cache.get(&cache_key) {
        let analyses: AnalysisListResponse = serde_json::from_str(&cached)
            .map_err(|e| AppError::internal(format!("Cache deserialization error: {}", e)))?;
        return Ok(Json(analyses));
    }

    // Query database
    let analyses = sqlx::query_as::<_, AnalysisWithFile>(
        "SELECT a.*, l.filename
         FROM analyses a
         LEFT JOIN log_files l ON a.log_file_id = l.id
         WHERE a.project_id = ?
         ORDER BY a.started_at DESC"
    )
    .bind(&project_id)
    .fetch_all(state.db.pool())
    .await?;

    let total = analyses.len() as i64;
    let response = AnalysisListResponse { analyses, total };

    // Cache the result (5 minute TTL)
    let response_json = serde_json::to_string(&response)?;
    state.cache_manager.result_cache.put(
        cache_key,
        response_json,
        Some(std::time::Duration::from_secs(300))
    );

    Ok(Json(response))
}
```

#### 2.3.3: Invalidate Cache on Modifications
Add cache invalidation when analyses are created/deleted:

```rust
// In create_analysis handler:
state.cache_manager.result_cache.remove(&CacheKeys::project_analyses_key(&project_id));

// In delete_analysis handler:
state.cache_manager.result_cache.remove(&CacheKeys::project_analyses_key(&project_id));
```

**Files Modified**:
- `loglens-web/src/handlers/analysis.rs` (~30 lines changed)

---

### Task 2.4: Implement Model Field Features
**Category**: 4 - Data Model Fields
**Location**: Multiple handlers
**Effort**: 3 hours
**Lines Added**: ~150 lines

#### 2.4.1: Use ErrorPattern category & severity Fields
**Location**: `loglens-web/src/handlers/knowledge.rs`

**Actions**:
1. Update `get_error_patterns` to return category & severity
2. Add query parameters for filtering by category/severity
3. Add sorting by severity

```rust
#[derive(Debug, Deserialize)]
pub struct PatternFilters {
    pub category: Option<String>,
    pub severity: Option<String>,
    pub min_frequency: Option<i32>,
}

pub async fn get_error_patterns(
    Path(project_id): Path<String>,
    Query(filters): Query<PatternFilters>,
    State(state): State<AppState>,
) -> Result<Json<Vec<ErrorPattern>>, AppError> {
    let mut query = "SELECT * FROM error_patterns WHERE project_id = ?".to_string();
    let mut conditions = vec![];

    if filters.category.is_some() {
        conditions.push("category = ?");
    }
    if filters.severity.is_some() {
        conditions.push("severity = ?");
    }
    if filters.min_frequency.is_some() {
        conditions.push("frequency >= ?");
    }

    if !conditions.is_empty() {
        query.push_str(" AND ");
        query.push_str(&conditions.join(" AND "));
    }

    query.push_str(" ORDER BY CASE severity
        WHEN 'critical' THEN 1
        WHEN 'high' THEN 2
        WHEN 'medium' THEN 3
        WHEN 'low' THEN 4
        END, frequency DESC");

    // Build and execute query with dynamic binding...
    // (implementation details)

    Ok(Json(patterns))
}
```

#### 2.4.2: Use KnowledgeBaseEntry is_public Field
**Location**: `loglens-web/src/handlers/knowledge.rs`

**Actions**:
1. Add `is_public` to create/update knowledge entry requests
2. Filter knowledge entries based on public visibility
3. Add public knowledge base endpoint (no auth required)

```rust
#[derive(Debug, Deserialize)]
pub struct CreateKnowledgeEntryRequest {
    pub title: String,
    pub problem_description: String,
    pub solution: String,
    pub tags: Option<String>,
    pub severity: String,
    pub is_public: Option<bool>, // New field
}

// New endpoint: GET /api/knowledge/public
pub async fn get_public_knowledge(
    Query(filters): Query<KnowledgeFilters>,
    State(state): State<AppState>,
) -> Result<Json<Vec<KnowledgeBaseEntry>>, AppError> {
    let entries = sqlx::query_as::<_, KnowledgeBaseEntry>(
        "SELECT * FROM knowledge_base
         WHERE is_public = true
         ORDER BY usage_count DESC, created_at DESC
         LIMIT 100"
    )
    .fetch_all(state.db.pool())
    .await?;

    Ok(Json(entries))
}
```

#### 2.4.3: Use StreamingFiltersQuery Fields
**Location**: `loglens-web/src/handlers/streaming.rs:213-223`

**Actions**:
1. Implement actual filtering in `get_recent_logs()`
2. Store recent logs in memory (ring buffer) or database
3. Apply level, source, since filters

```rust
pub async fn get_recent_logs(
    Path(project_id): Path<Uuid>,
    Query(filters): Query<StreamingFiltersQuery>,
    State(state): State<AppState>,
) -> Result<Json<Vec<StreamingLogEntry>>, AppError> {
    // Option 1: Query from a recent_logs table (recommended)
    let mut query = "SELECT * FROM streaming_logs WHERE project_id = ?".to_string();
    let mut conditions = vec![];

    if let Some(level) = &filters.level {
        conditions.push(format!("level = '{}'", level));
    }
    if let Some(source) = &filters.source {
        conditions.push(format!("source = '{}'", source));
    }
    if let Some(since) = &filters.since {
        conditions.push(format!("timestamp >= '{}'", since));
    }

    if !conditions.is_empty() {
        query.push_str(" AND ");
        query.push_str(&conditions.join(" AND "));
    }

    query.push_str(" ORDER BY timestamp DESC LIMIT 1000");

    let logs = sqlx::query_as::<_, StreamingLogEntry>(&query)
        .bind(project_id)
        .fetch_all(state.db.pool())
        .await?;

    Ok(Json(logs))
}
```

**Note**: This requires creating a `streaming_logs` table via migration.

**Files Modified**:
- `loglens-web/src/handlers/knowledge.rs` (~80 lines)
- `loglens-web/src/handlers/streaming.rs` (~40 lines)
- `loglens-web/src/routes.rs` (+1 line for public knowledge endpoint)

**Migrations Required**:
- Add `streaming_logs` table for recent log storage

---

## Phase 3: Streaming Sources Activation (Priority: Medium)

### Task 3.1: Activate Streaming Source Manager
**Category**: 2.1 - Advanced Streaming Sources
**Location**: Multiple files
**Effort**: 4 hours
**Lines Modified**: ~100 lines

**Architecture Decision**: The `StreamingSourceManager` is fully implemented but never initialized. We need to:
1. Add it to AppState
2. Initialize on startup
3. Wire up existing handler routes to use it
4. Persist source configurations

#### 3.1.1: Add StreamingSourceManager to AppState
**Location**: `loglens-web/src/lib.rs`

```rust
pub struct AppState {
    pub db: Database,
    pub config: WebConfig,
    pub circuit_breakers: Arc<CircuitBreakerRegistry>,
    pub cache_manager: Arc<CacheManager>,
    pub streaming_hub: Arc<crate::streaming::StreamingHub>,
    pub streaming_manager: Arc<tokio::sync::RwLock<crate::streaming::sources::StreamingSourceManager>>, // NEW
    pub optimized_db: Arc<OptimizedDbOps>,
    pub metrics_collector: Arc<middleware::metrics::MetricsCollector>,
}

impl AppState {
    pub async fn new(config: WebConfig) -> anyhow::Result<Self> {
        // ... existing initialization ...

        // Initialize streaming hub
        let streaming_hub = Arc::new(streaming::StreamingHub::new());

        // Initialize streaming source manager (NEW)
        let streaming_manager = Arc::new(tokio::sync::RwLock::new(
            crate::streaming::sources::StreamingSourceManager::new(Arc::clone(&streaming_hub))
        ));

        // ... rest of initialization ...

        Ok(Self {
            db,
            config,
            circuit_breakers,
            cache_manager,
            streaming_hub,
            streaming_manager, // NEW
            optimized_db,
            metrics_collector,
        })
    }
}
```

#### 3.1.2: Update Streaming Handlers to Use Manager
**Location**: `loglens-web/src/handlers/streaming.rs`

Replace mock implementations with actual manager calls:

```rust
pub async fn create_streaming_source(
    Path(project_id): Path<Uuid>,
    State(state): State<AppState>,
    Json(request): Json<CreateStreamingSourceRequest>,
) -> Result<Json<StreamingSourceResponse>, AppError> {
    // Validate request
    if request.source_type.is_empty() || request.name.is_empty() {
        return Err(AppError::bad_request("Invalid request"));
    }

    // Parse source type and create config
    let source_type = parse_source_type(&request.source_type, &request.config)?;
    let parser_config = request.parser_config.map(|p| parse_parser_config(p)).unwrap_or_default();

    let config = StreamingSourceConfig {
        source_type,
        project_id,
        name: request.name.clone(),
        parser_config,
        buffer_size: request.buffer_size.unwrap_or(100),
        batch_timeout: Duration::from_secs(request.batch_timeout_seconds.unwrap_or(2)),
        restart_on_error: request.restart_on_error.unwrap_or(true),
        max_restarts: request.max_restarts,
    };

    // Start source via manager
    let mut manager = state.streaming_manager.write().await;
    let source_id = manager.start_source(config).await
        .map_err(|e| AppError::internal(format!("Failed to start source: {}", e)))?;

    // Persist source config to database
    persist_source_config(&state.db, &source_id, &request).await?;

    let response = StreamingSourceResponse {
        source_id,
        name: request.name,
        source_type: request.source_type,
        project_id,
        status: "active".to_string(),
        created_at: chrono::Utc::now().to_rfc3339(),
    };

    Ok(Json(response))
}

pub async fn stop_streaming_source(
    Path((project_id, source_id)): Path<(Uuid, String)>,
    State(state): State<AppState>,
) -> Result<StatusCode, AppError> {
    // Stop source via manager
    let mut manager = state.streaming_manager.write().await;
    manager.stop_source(&source_id).await
        .map_err(|e| AppError::internal(format!("Failed to stop source: {}", e)))?;

    // Remove from database
    delete_source_config(&state.db, &source_id).await?;

    Ok(StatusCode::NO_CONTENT)
}

pub async fn list_streaming_sources(
    Path(project_id): Path<Uuid>,
    State(state): State<AppState>,
) -> Result<Json<Vec<StreamingSourceResponse>>, AppError> {
    // Get active sources from manager
    let manager = state.streaming_manager.read().await;
    let sources = manager.list_sources();

    // Convert to response format
    let responses: Vec<StreamingSourceResponse> = sources
        .into_iter()
        .map(|(id, name)| StreamingSourceResponse {
            source_id: id.clone(),
            name,
            source_type: "file".to_string(), // Would need to track this
            project_id,
            status: "active".to_string(),
            created_at: chrono::Utc::now().to_rfc3339(),
        })
        .collect();

    Ok(Json(responses))
}

// Helper functions
fn parse_source_type(source_type: &str, config: &serde_json::Value) -> Result<StreamingSourceType, AppError> {
    match source_type {
        "file" => {
            let path = config.get("path")
                .and_then(|v| v.as_str())
                .ok_or_else(|| AppError::bad_request("Missing 'path' for file source"))?;
            Ok(StreamingSourceType::File { path: PathBuf::from(path) })
        }
        "command" => {
            let command = config.get("command")
                .and_then(|v| v.as_str())
                .ok_or_else(|| AppError::bad_request("Missing 'command'"))?;
            let args = config.get("args")
                .and_then(|v| v.as_array())
                .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect())
                .unwrap_or_default();
            Ok(StreamingSourceType::Command { command: command.to_string(), args })
        }
        "tcp" => {
            let port = config.get("port")
                .and_then(|v| v.as_u64())
                .ok_or_else(|| AppError::bad_request("Missing 'port' for TCP source"))?;
            Ok(StreamingSourceType::TcpListener { port: port as u16 })
        }
        "stdin" => Ok(StreamingSourceType::Stdin),
        "http" => {
            let path = config.get("path")
                .and_then(|v| v.as_str())
                .ok_or_else(|| AppError::bad_request("Missing 'path' for HTTP source"))?;
            Ok(StreamingSourceType::HttpEndpoint { path: path.to_string() })
        }
        _ => Err(AppError::bad_request(format!("Unknown source type: {}", source_type)))
    }
}

fn parse_parser_config(req: ParserConfigRequest) -> ParserConfig {
    let log_format = match req.log_format.as_str() {
        "json" => LogFormat::Json,
        "syslog" => LogFormat::Syslog,
        "common" => LogFormat::CommonLog,
        _ => LogFormat::Text,
    };

    ParserConfig {
        log_format,
        timestamp_format: req.timestamp_format,
        level_field: req.level_field,
        message_field: req.message_field,
        metadata_fields: req.metadata_fields.unwrap_or_default(),
    }
}
```

#### 3.1.3: Create Database Schema for Source Persistence
**Location**: New migration `migrations/XXXXX_streaming_sources.sql`

```sql
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
```

#### 3.1.4: Implement Persistence Helpers
**Location**: `loglens-web/src/handlers/streaming.rs`

```rust
async fn persist_source_config(
    db: &Database,
    source_id: &str,
    request: &CreateStreamingSourceRequest,
) -> Result<(), AppError> {
    let config_json = serde_json::to_string(&request.config)?;
    let parser_config_json = request.parser_config.as_ref()
        .map(|p| serde_json::to_string(p))
        .transpose()?;

    sqlx::query!(
        "INSERT INTO streaming_sources
         (id, project_id, name, source_type, config, parser_config,
          buffer_size, batch_timeout_seconds, restart_on_error, max_restarts,
          status, created_at, updated_at)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, 'active', datetime('now'), datetime('now'))",
        source_id,
        request.project_id, // Assuming this is added to request
        request.name,
        request.source_type,
        config_json,
        parser_config_json,
        request.buffer_size,
        request.batch_timeout_seconds,
        request.restart_on_error,
        request.max_restarts
    )
    .execute(db.pool())
    .await?;

    Ok(())
}

async fn delete_source_config(db: &Database, source_id: &str) -> Result<(), AppError> {
    sqlx::query!("DELETE FROM streaming_sources WHERE id = ?", source_id)
        .execute(db.pool())
        .await?;
    Ok(())
}
```

#### 3.1.5: Auto-Restore Sources on Startup
**Location**: `loglens-web/src/lib.rs` or `loglens-web/src/main.rs`

```rust
// In AppState::new() or main startup
async fn restore_streaming_sources(state: &AppState) -> anyhow::Result<()> {
    let sources = sqlx::query!(
        "SELECT * FROM streaming_sources WHERE status = 'active'"
    )
    .fetch_all(state.db.pool())
    .await?;

    for source in sources {
        // Parse and recreate source config
        let config_value: serde_json::Value = serde_json::from_str(&source.config)?;
        let source_type = parse_source_type(&source.source_type, &config_value)?;

        let parser_config = if let Some(pc) = source.parser_config {
            serde_json::from_str(&pc)?
        } else {
            ParserConfig::default()
        };

        let config = StreamingSourceConfig {
            source_type,
            project_id: Uuid::parse_str(&source.project_id)?,
            name: source.name.clone(),
            parser_config,
            buffer_size: source.buffer_size as usize,
            batch_timeout: Duration::from_secs(source.batch_timeout_seconds as u64),
            restart_on_error: source.restart_on_error,
            max_restarts: source.max_restarts,
        };

        let mut manager = state.streaming_manager.write().await;
        manager.start_source(config).await?;

        tracing::info!("Restored streaming source: {} ({})", source.name, source.id);
    }

    Ok(())
}
```

**Files Modified**:
- `loglens-web/src/lib.rs` (~40 lines)
- `loglens-web/src/handlers/streaming.rs` (~200 lines)
- `loglens-web/src/handlers/mod.rs` (+1 line)

**Files Created**:
- Migration file for streaming_sources and streaming_logs tables

**Testing Checklist**:
- [ ] File tailing source created and streams logs
- [ ] Command source executes and streams output
- [ ] TCP listener accepts connections and processes logs
- [ ] Sources persist across server restart
- [ ] Source stop functionality works
- [ ] Multiple sources can run concurrently
- [ ] Parser configurations work (JSON, Syslog, Text)

---

## Phase 4: Frontend Integration (Priority: High)

### Task 4.1: Create WebSocket Analysis Progress Component
**Location**: New React component
**Effort**: 6 hours
**Lines Added**: ~400 lines

#### 4.1.1: Create WebSocket Hook
**Location**: `loglens-web/frontend-react/src/hooks/useWebSocketAnalysis.ts`

```typescript
import { useState, useEffect, useCallback, useRef } from 'react';

export interface AnalysisProgress {
  stage: string;
  progress: number; // 0.0 to 1.0
  message: string;
  elapsed_ms: number;
}

export interface AnalysisStats {
  total_lines: number;
  parsed_entries: number;
  filtered_entries: number;
  slimmed_entries: number;
  processing_time_ms: number;
  ai_analysis_time_ms: number;
}

export interface AnalysisComplete {
  analysis: any; // Full AnalysisResponse
  analysis_id: string;
  elapsed_ms: number;
  stats: AnalysisStats;
}

export interface WebSocketMessage {
  type: 'Progress' | 'Error' | 'Complete' | 'Cancelled' | 'Heartbeat';
  data: {
    stage?: string;
    progress?: number;
    message?: string;
    elapsed_ms?: number;
    error?: string;
    analysis?: any;
    analysis_id?: string;
    stats?: AnalysisStats;
    reason?: string;
    timestamp?: number;
  };
}

export interface UseWebSocketAnalysisOptions {
  projectId: string;
  fileId: string;
  provider: string;
  level: string;
  apiKey?: string;
  userContext?: string;
  onProgress?: (progress: AnalysisProgress) => void;
  onComplete?: (result: AnalysisComplete) => void;
  onError?: (error: string) => void;
  onCancel?: (reason: string) => void;
}

export function useWebSocketAnalysis(options: UseWebSocketAnalysisOptions) {
  const [isConnected, setIsConnected] = useState(false);
  const [isAnalyzing, setIsAnalyzing] = useState(false);
  const [progress, setProgress] = useState<AnalysisProgress | null>(null);
  const [result, setResult] = useState<AnalysisComplete | null>(null);
  const [error, setError] = useState<string | null>(null);

  const wsRef = useRef<WebSocket | null>(null);
  const reconnectTimeoutRef = useRef<NodeJS.Timeout | null>(null);

  const connect = useCallback(() => {
    const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
    const host = window.location.host;
    const params = new URLSearchParams({
      level: options.level,
      provider: options.provider,
      ...(options.apiKey && { api_key: options.apiKey }),
      ...(options.userContext && { user_context: options.userContext }),
    });

    const url = `${protocol}//${host}/api/projects/${options.projectId}/files/${options.fileId}/analyze/ws?${params}`;

    const ws = new WebSocket(url);
    wsRef.current = ws;

    ws.onopen = () => {
      console.log('WebSocket connected for analysis');
      setIsConnected(true);
      setIsAnalyzing(true);
      setError(null);
    };

    ws.onmessage = (event) => {
      try {
        const message: WebSocketMessage = JSON.parse(event.data);

        switch (message.type) {
          case 'Progress':
            const progressData: AnalysisProgress = {
              stage: message.data.stage || '',
              progress: message.data.progress || 0,
              message: message.data.message || '',
              elapsed_ms: message.data.elapsed_ms || 0,
            };
            setProgress(progressData);
            options.onProgress?.(progressData);
            break;

          case 'Complete':
            const completeData: AnalysisComplete = {
              analysis: message.data.analysis,
              analysis_id: message.data.analysis_id || '',
              elapsed_ms: message.data.elapsed_ms || 0,
              stats: message.data.stats || {} as AnalysisStats,
            };
            setResult(completeData);
            setIsAnalyzing(false);
            options.onComplete?.(completeData);
            break;

          case 'Error':
            const errorMsg = message.data.error || 'Unknown error occurred';
            setError(errorMsg);
            setIsAnalyzing(false);
            options.onError?.(errorMsg);
            break;

          case 'Cancelled':
            const reason = message.data.reason || 'Analysis cancelled';
            setIsAnalyzing(false);
            options.onCancel?.(reason);
            break;

          case 'Heartbeat':
            // Just acknowledge heartbeat
            console.log('Heartbeat received');
            break;
        }
      } catch (err) {
        console.error('Failed to parse WebSocket message:', err);
      }
    };

    ws.onerror = (event) => {
      console.error('WebSocket error:', event);
      setError('WebSocket connection error');
      setIsAnalyzing(false);
    };

    ws.onclose = (event) => {
      console.log('WebSocket closed:', event.code, event.reason);
      setIsConnected(false);
      setIsAnalyzing(false);

      // Auto-reconnect if unexpected close
      if (event.code !== 1000 && !error) {
        reconnectTimeoutRef.current = setTimeout(() => {
          console.log('Attempting to reconnect...');
          connect();
        }, 5000);
      }
    };
  }, [options]);

  const cancel = useCallback(() => {
    if (wsRef.current && wsRef.current.readyState === WebSocket.OPEN) {
      wsRef.current.send('cancel');
      setIsAnalyzing(false);
    }
  }, []);

  const disconnect = useCallback(() => {
    if (wsRef.current) {
      wsRef.current.close(1000, 'User initiated disconnect');
      wsRef.current = null;
    }
    if (reconnectTimeoutRef.current) {
      clearTimeout(reconnectTimeoutRef.current);
      reconnectTimeoutRef.current = null;
    }
  }, []);

  useEffect(() => {
    return () => {
      disconnect();
    };
  }, [disconnect]);

  return {
    isConnected,
    isAnalyzing,
    progress,
    result,
    error,
    connect,
    cancel,
    disconnect,
  };
}
```

#### 4.1.2: Create Progress Display Component
**Location**: `loglens-web/frontend-react/src/components/AnalysisProgress.tsx`

```typescript
import React from 'react';
import { AnalysisProgress as Progress, AnalysisStats } from '../hooks/useWebSocketAnalysis';

interface AnalysisProgressProps {
  progress: Progress;
  stats?: AnalysisStats;
  onCancel?: () => void;
}

export const AnalysisProgress: React.FC<AnalysisProgressProps> = ({
  progress,
  stats,
  onCancel
}) => {
  const percentage = Math.round(progress.progress * 100);
  const elapsedSeconds = (progress.elapsed_ms / 1000).toFixed(1);

  const getStageLabel = (stage: string): string => {
    const labels: Record<string, string> = {
      'starting': '🔄 Initializing',
      'reading_file': '📖 Reading File',
      'parsing': '🔍 Parsing Logs',
      'filtering': '🔧 Filtering',
      'slimming': '✂️ Optimizing',
      'ai_analysis': '🤖 AI Analysis',
      'finalizing': '✅ Finalizing',
    };
    return labels[stage] || stage;
  };

  return (
    <div className="analysis-progress-container">
      <div className="progress-header">
        <h3>Analysis in Progress</h3>
        <span className="elapsed-time">{elapsedSeconds}s elapsed</span>
      </div>

      <div className="progress-stage">
        <span className="stage-label">{getStageLabel(progress.stage)}</span>
        <span className="percentage">{percentage}%</span>
      </div>

      <div className="progress-bar">
        <div
          className="progress-fill"
          style={{ width: `${percentage}%` }}
          role="progressbar"
          aria-valuenow={percentage}
          aria-valuemin={0}
          aria-valuemax={100}
        />
      </div>

      <p className="progress-message">{progress.message}</p>

      {stats && (
        <div className="progress-stats">
          <div className="stat">
            <span className="stat-label">Total Lines:</span>
            <span className="stat-value">{stats.total_lines.toLocaleString()}</span>
          </div>
          <div className="stat">
            <span className="stat-label">Parsed:</span>
            <span className="stat-value">{stats.parsed_entries.toLocaleString()}</span>
          </div>
          <div className="stat">
            <span className="stat-label">Filtered:</span>
            <span className="stat-value">{stats.filtered_entries.toLocaleString()}</span>
          </div>
          <div className="stat">
            <span className="stat-label">AI Input:</span>
            <span className="stat-value">{stats.slimmed_entries.toLocaleString()}</span>
          </div>
        </div>
      )}

      {onCancel && (
        <button
          onClick={onCancel}
          className="cancel-button"
          aria-label="Cancel analysis"
        >
          Cancel Analysis
        </button>
      )}
    </div>
  );
};
```

#### 4.1.3: Integrate into Analysis Page
**Location**: `loglens-web/frontend-react/src/pages/AnalysisPage.tsx` (or similar)

```typescript
import { useWebSocketAnalysis } from '../hooks/useWebSocketAnalysis';
import { AnalysisProgress } from '../components/AnalysisProgress';

export const AnalysisPage: React.FC = () => {
  const [useWebSocket, setUseWebSocket] = useState(true);

  const {
    isConnected,
    isAnalyzing,
    progress,
    result,
    error,
    connect,
    cancel,
  } = useWebSocketAnalysis({
    projectId: projectId, // from route params
    fileId: fileId, // from route params
    provider: 'openrouter',
    level: 'ERROR',
    onProgress: (prog) => {
      console.log('Analysis progress:', prog);
    },
    onComplete: (res) => {
      console.log('Analysis complete:', res);
      // Navigate to results or display inline
      navigate(`/projects/${projectId}/analyses/${res.analysis_id}`);
    },
    onError: (err) => {
      console.error('Analysis error:', err);
      toast.error(`Analysis failed: ${err}`);
    },
  });

  const startAnalysis = () => {
    if (useWebSocket) {
      connect();
    } else {
      // Use traditional POST endpoint
      startTraditionalAnalysis();
    }
  };

  return (
    <div>
      <h1>Log Analysis</h1>

      <div className="analysis-options">
        <label>
          <input
            type="checkbox"
            checked={useWebSocket}
            onChange={(e) => setUseWebSocket(e.target.checked)}
          />
          Use real-time progress (WebSocket)
        </label>
      </div>

      {!isAnalyzing && (
        <button onClick={startAnalysis}>
          Start Analysis
        </button>
      )}

      {isAnalyzing && progress && (
        <AnalysisProgress
          progress={progress}
          stats={result?.stats}
          onCancel={cancel}
        />
      )}

      {error && (
        <div className="error-message">
          <strong>Error:</strong> {error}
        </div>
      )}

      {result && (
        <div className="analysis-result">
          <h2>Analysis Complete!</h2>
          <p>Analysis ID: {result.analysis_id}</p>
          <p>Time taken: {(result.elapsed_ms / 1000).toFixed(2)}s</p>
          {/* Display or navigate to full results */}
        </div>
      )}
    </div>
  );
};
```

**Files Created**:
- `loglens-web/frontend-react/src/hooks/useWebSocketAnalysis.ts` (~250 lines)
- `loglens-web/frontend-react/src/components/AnalysisProgress.tsx` (~150 lines)

**Files Modified**:
- Analysis page component (~50 lines)
- Styles for progress component

---

### Task 4.2: Add UI for Model Field Features
**Location**: Multiple frontend components
**Effort**: 3 hours
**Lines Added**: ~200 lines

#### 4.2.1: Pattern Filtering by Category/Severity
**Location**: `loglens-web/frontend-react/src/pages/PatternsPage.tsx`

```typescript
const [categoryFilter, setCategoryFilter] = useState<string>('all');
const [severityFilter, setSeverityFilter] = useState<string>('all');

const fetchPatterns = async () => {
  const params = new URLSearchParams();
  if (categoryFilter !== 'all') params.append('category', categoryFilter);
  if (severityFilter !== 'all') params.append('severity', severityFilter);

  const response = await fetch(`/api/projects/${projectId}/patterns?${params}`);
  const patterns = await response.json();
  setPatterns(patterns);
};

// UI
<div className="pattern-filters">
  <select value={categoryFilter} onChange={(e) => setCategoryFilter(e.target.value)}>
    <option value="all">All Categories</option>
    <option value="code">Code Errors</option>
    <option value="infrastructure">Infrastructure</option>
    <option value="configuration">Configuration</option>
    <option value="external">External</option>
  </select>

  <select value={severityFilter} onChange={(e) => setSeverityFilter(e.target.value)}>
    <option value="all">All Severities</option>
    <option value="critical">Critical</option>
    <option value="high">High</option>
    <option value="medium">Medium</option>
    <option value="low">Low</option>
  </select>
</div>

<div className="pattern-list">
  {patterns.map(pattern => (
    <div key={pattern.id} className={`pattern-card severity-${pattern.severity}`}>
      <span className={`category-badge category-${pattern.category}`}>
        {pattern.category}
      </span>
      <span className={`severity-badge severity-${pattern.severity}`}>
        {pattern.severity}
      </span>
      <h3>{pattern.pattern}</h3>
      <p>{pattern.description}</p>
      <span className="frequency">Seen {pattern.frequency} times</span>
    </div>
  ))}
</div>
```

#### 4.2.2: Public Knowledge Base UI
**Location**: New page `loglens-web/frontend-react/src/pages/PublicKnowledgePage.tsx`

```typescript
import React, { useState, useEffect } from 'react';

export const PublicKnowledgePage: React.FC = () => {
  const [knowledge, setKnowledge] = useState([]);
  const [searchTerm, setSearchTerm] = useState('');

  useEffect(() => {
    fetchPublicKnowledge();
  }, [searchTerm]);

  const fetchPublicKnowledge = async () => {
    const response = await fetch(`/api/knowledge/public?search=${searchTerm}`);
    const data = await response.json();
    setKnowledge(data);
  };

  return (
    <div className="public-knowledge-page">
      <h1>🌐 Public Knowledge Base</h1>
      <p>Community-shared solutions for common problems</p>

      <input
        type="search"
        placeholder="Search solutions..."
        value={searchTerm}
        onChange={(e) => setSearchTerm(e.target.value)}
        className="search-input"
      />

      <div className="knowledge-grid">
        {knowledge.map(entry => (
          <div key={entry.id} className="knowledge-card">
            <h3>{entry.title}</h3>
            <span className={`severity-badge severity-${entry.severity}`}>
              {entry.severity}
            </span>
            <p className="problem">{entry.problem_description}</p>
            <details>
              <summary>View Solution</summary>
              <div className="solution">
                <pre>{entry.solution}</pre>
              </div>
            </details>
            <div className="meta">
              <span>Used {entry.usage_count} times</span>
              <span>Tags: {entry.tags || 'None'}</span>
            </div>
          </div>
        ))}
      </div>
    </div>
  );
};
```

#### 4.2.3: Knowledge Entry Creation with Public Toggle
**Location**: `loglens-web/frontend-react/src/components/CreateKnowledgeEntry.tsx`

```typescript
const [isPublic, setIsPublic] = useState(false);

const handleSubmit = async () => {
  const response = await fetch(`/api/projects/${projectId}/knowledge`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({
      title,
      problem_description: problemDescription,
      solution,
      tags,
      severity,
      is_public: isPublic, // NEW
    }),
  });

  if (response.ok) {
    toast.success('Knowledge entry created!');
    onSuccess();
  }
};

// In the form:
<label>
  <input
    type="checkbox"
    checked={isPublic}
    onChange={(e) => setIsPublic(e.target.checked)}
  />
  Share publicly (help the community)
</label>
```

**Files Created**:
- `loglens-web/frontend-react/src/pages/PublicKnowledgePage.tsx` (~120 lines)

**Files Modified**:
- `loglens-web/frontend-react/src/pages/PatternsPage.tsx` (~40 lines)
- `loglens-web/frontend-react/src/components/CreateKnowledgeEntry.tsx` (~10 lines)
- Router configuration (+1 route)

---

### Task 4.3: Streaming Dashboard UI
**Location**: New page
**Effort**: 4 hours
**Lines Added**: ~300 lines

#### 4.3.1: Create Streaming Management Page
**Location**: `loglens-web/frontend-react/src/pages/StreamingPage.tsx`

```typescript
import React, { useState, useEffect } from 'react';

export const StreamingPage: React.FC = () => {
  const [sources, setSources] = useState([]);
  const [stats, setStats] = useState(null);
  const [showCreateModal, setShowCreateModal] = useState(false);

  useEffect(() => {
    fetchSources();
    fetchStats();
    const interval = setInterval(fetchStats, 5000); // Poll stats every 5s
    return () => clearInterval(interval);
  }, []);

  const fetchSources = async () => {
    const response = await fetch(`/api/projects/${projectId}/streaming/sources`);
    const data = await response.json();
    setSources(data);
  };

  const fetchStats = async () => {
    const response = await fetch(`/api/projects/${projectId}/streaming/stats`);
    const data = await response.json();
    setStats(data);
  };

  const createSource = async (sourceConfig) => {
    const response = await fetch(`/api/projects/${projectId}/streaming/sources`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(sourceConfig),
    });

    if (response.ok) {
      toast.success('Streaming source created!');
      fetchSources();
      setShowCreateModal(false);
    }
  };

  const stopSource = async (sourceId) => {
    const response = await fetch(
      `/api/projects/${projectId}/streaming/sources/${sourceId}`,
      { method: 'DELETE' }
    );

    if (response.ok) {
      toast.success('Source stopped');
      fetchSources();
    }
  };

  return (
    <div className="streaming-page">
      <header>
        <h1>📡 Real-Time Log Streaming</h1>
        <button onClick={() => setShowCreateModal(true)}>
          + New Source
        </button>
      </header>

      {stats && (
        <div className="streaming-stats">
          <div className="stat-card">
            <h3>Active Sources</h3>
            <div className="stat-value">{stats.active_sources}</div>
          </div>
          <div className="stat-card">
            <h3>Live Connections</h3>
            <div className="stat-value">{stats.active_connections}</div>
          </div>
          <div className="stat-card">
            <h3>Logs Processed</h3>
            <div className="stat-value">{stats.total_logs_processed.toLocaleString()}</div>
          </div>
        </div>
      )}

      <div className="sources-list">
        <h2>Active Sources</h2>
        {sources.length === 0 ? (
          <p>No streaming sources configured. Create one to get started!</p>
        ) : (
          sources.map(source => (
            <div key={source.source_id} className="source-card">
              <div className="source-header">
                <h3>{source.name}</h3>
                <span className={`status-badge status-${source.status}`}>
                  {source.status}
                </span>
              </div>
              <p className="source-type">{source.source_type}</p>
              <p className="created-at">Created: {new Date(source.created_at).toLocaleString()}</p>
              <button
                onClick={() => stopSource(source.source_id)}
                className="stop-button"
              >
                Stop Source
              </button>
            </div>
          ))
        )}
      </div>

      {showCreateModal && (
        <CreateSourceModal
          onSubmit={createSource}
          onCancel={() => setShowCreateModal(false)}
        />
      )}
    </div>
  );
};

interface CreateSourceModalProps {
  onSubmit: (config: any) => void;
  onCancel: () => void;
}

const CreateSourceModal: React.FC<CreateSourceModalProps> = ({ onSubmit, onCancel }) => {
  const [sourceType, setSourceType] = useState('file');
  const [name, setName] = useState('');
  const [config, setConfig] = useState({});

  const handleSubmit = () => {
    onSubmit({
      name,
      source_type: sourceType,
      config,
      parser_config: {
        log_format: 'text',
      },
      buffer_size: 100,
      batch_timeout_seconds: 2,
      restart_on_error: true,
    });
  };

  return (
    <div className="modal-overlay">
      <div className="modal-content">
        <h2>Create Streaming Source</h2>

        <label>
          Source Name:
          <input
            type="text"
            value={name}
            onChange={(e) => setName(e.target.value)}
            placeholder="e.g., Application Logs"
          />
        </label>

        <label>
          Source Type:
          <select value={sourceType} onChange={(e) => setSourceType(e.target.value)}>
            <option value="file">File (tail -f)</option>
            <option value="command">Command Output</option>
            <option value="tcp">TCP Listener</option>
            <option value="http">HTTP Endpoint</option>
          </select>
        </label>

        {sourceType === 'file' && (
          <label>
            File Path:
            <input
              type="text"
              value={config.path || ''}
              onChange={(e) => setConfig({...config, path: e.target.value})}
              placeholder="/var/log/app.log"
            />
          </label>
        )}

        {sourceType === 'command' && (
          <>
            <label>
              Command:
              <input
                type="text"
                value={config.command || ''}
                onChange={(e) => setConfig({...config, command: e.target.value})}
                placeholder="journalctl -f"
              />
            </label>
            <label>
              Arguments (comma-separated):
              <input
                type="text"
                onChange={(e) => setConfig({
                  ...config,
                  args: e.target.value.split(',').map(s => s.trim())
                })}
                placeholder="-u, myapp.service"
              />
            </label>
          </>
        )}

        {sourceType === 'tcp' && (
          <label>
            TCP Port:
            <input
              type="number"
              value={config.port || ''}
              onChange={(e) => setConfig({...config, port: parseInt(e.target.value)})}
              placeholder="5140"
            />
          </label>
        )}

        {sourceType === 'http' && (
          <label>
            Endpoint Path:
            <input
              type="text"
              value={config.path || ''}
              onChange={(e) => setConfig({...config, path: e.target.value})}
              placeholder="/ingest/logs"
            />
          </label>
        )}

        <div className="modal-actions">
          <button onClick={handleSubmit} disabled={!name || Object.keys(config).length === 0}>
            Create Source
          </button>
          <button onClick={onCancel} className="cancel">
            Cancel
          </button>
        </div>
      </div>
    </div>
  );
};
```

**Files Created**:
- `loglens-web/frontend-react/src/pages/StreamingPage.tsx` (~300 lines)

**Files Modified**:
- Router configuration (+1 route)
- Navigation menu (+1 link)

---

## Phase 5: WASM Module Investigation (Priority: Low)

### Task 5.1: Investigate and Optimize RegexCache
**Category**: 5.1 - WASM RegexCache
**Location**: `loglens-wasm/src/` (need to locate)
**Effort**: 2 hours
**Status**: Investigation Required

**Actions**:
1. Locate RegexCache implementation in WASM module
2. Determine if it's used in log parsing performance path
3. If beneficial, activate caching:
   - Initialize RegexCache singleton
   - Cache compiled regex patterns
   - Benchmark performance improvement
4. If not beneficial, remove the dead code

**Investigation Questions**:
- Where is RegexCache defined?
- What was its intended use case?
- Are regex operations a bottleneck in WASM?
- Is the overhead of caching worth it?

**Decision Point**:
- If >10% performance gain → Activate
- If <10% performance gain → Delete

---

## Phase 6: Documentation & Testing (Priority: Medium)

### Task 6.1: Update API Documentation
**Location**: `loglens-web/README.md`, API docs
**Effort**: 1 hour

**Actions**:
1. Document new WebSocket endpoint
2. Document analytics endpoints
3. Document streaming source management endpoints
4. Update WebSocket events section
5. Add code examples for new features

### Task 6.2: Create Integration Tests
**Location**: `loglens-web/tests/`
**Effort**: 3 hours

**Actions**:
1. WebSocket analysis flow test
2. Streaming source lifecycle test
3. Analytics endpoints test
4. Model field filtering test
5. Public knowledge base test

### Task 6.3: Update Frontend Documentation
**Location**: `loglens-web/frontend-react/README.md`
**Effort**: 30 minutes

**Actions**:
1. Document useWebSocketAnalysis hook
2. Document streaming components
3. Add usage examples

---

## Implementation Timeline

### Week 1: Backend Core (3 days)
- **Day 1**: Phase 1 (Cleanup) + Phase 2 Tasks 2.1-2.2
- **Day 2**: Phase 2 Tasks 2.3-2.4 + Phase 3 Task 3.1 (partial)
- **Day 3**: Phase 3 Task 3.1 (complete) + Testing

### Week 2: Frontend & Polish (3 days)
- **Day 4**: Phase 4 Task 4.1 (WebSocket UI)
- **Day 5**: Phase 4 Tasks 4.2-4.3 (Model fields + Streaming UI)
- **Day 6**: Phase 5 + Phase 6 (WASM investigation + Documentation + Testing)

**Total Estimated Time**: 6 days (48 hours)

---

## Risk Assessment

### High Risk Items
1. **Streaming Sources** - Complex feature with multiple moving parts
   - Mitigation: Start with file tailing only, expand gradually

2. **WebSocket Stability** - Connection drops, reconnection logic
   - Mitigation: Robust error handling, exponential backoff

3. **Database Migrations** - Schema changes require careful planning
   - Mitigation: Test migrations on copy of production data first

### Medium Risk Items
1. **WASM Module** - Unknown location and structure
   - Mitigation: Allow extra investigation time

2. **Performance Impact** - New features may affect existing performance
   - Mitigation: Benchmark before/after, add monitoring

### Low Risk Items
1. **UI Components** - Straightforward React components
2. **Documentation** - Time-consuming but low risk
3. **Cleanup** - Deletion of unused code is safe

---

## Success Criteria

### Functional
- [ ] WebSocket analysis progress works end-to-end
- [ ] Streaming sources can be created and managed
- [ ] All model fields are utilized in UI
- [ ] Analytics endpoints return data correctly
- [ ] Public knowledge base accessible

### Performance
- [ ] WebSocket latency <100ms
- [ ] Analysis progress updates every 500ms
- [ ] Streaming processes 1000+ logs/second
- [ ] Cache hit rate >80% for project analyses

### Quality
- [ ] All tests passing
- [ ] No new compiler warnings
- [ ] Documentation complete
- [ ] Code review approved

---

## Rollback Plan

If critical issues arise during implementation:

1. **Immediate Rollback** (< 5 minutes):
   - Git revert to last stable commit
   - Restart services

2. **Partial Rollback** (< 30 minutes):
   - Disable specific features via feature flags
   - Remove problematic routes
   - Revert database migrations

3. **Full Rollback** (< 2 hours):
   - Restore database from backup
   - Redeploy previous version
   - Notify users of temporary feature unavailability

---

## Post-Implementation Tasks

### Monitoring
- [ ] Add WebSocket connection metrics
- [ ] Monitor streaming source resource usage
- [ ] Track analytics endpoint usage
- [ ] Alert on high error rates

### User Communication
- [ ] Release notes for new features
- [ ] Tutorial videos/documentation
- [ ] Feature announcement
- [ ] Collect user feedback

### Future Enhancements
- [ ] WebSocket authentication
- [ ] Streaming source templates
- [ ] Advanced filtering UI
- [ ] Real-time collaboration features

---

## Appendix A: File Checklist

### Files to Create
- [ ] `loglens-web/src/handlers/analytics.rs`
- [ ] `loglens-web/frontend-react/src/hooks/useWebSocketAnalysis.ts`
- [ ] `loglens-web/frontend-react/src/components/AnalysisProgress.tsx`
- [ ] `loglens-web/frontend-react/src/pages/PublicKnowledgePage.tsx`
- [ ] `loglens-web/frontend-react/src/pages/StreamingPage.tsx`
- [ ] Migration: `XXXXX_streaming_sources.sql`
- [ ] Migration: `XXXXX_streaming_logs.sql`

### Files to Modify
- [ ] `loglens-web/src/routes.rs`
- [ ] `loglens-web/src/lib.rs`
- [ ] `loglens-web/src/handlers/mod.rs`
- [ ] `loglens-web/src/handlers/analysis.rs`
- [ ] `loglens-web/src/handlers/knowledge.rs`
- [ ] `loglens-web/src/handlers/streaming.rs`
- [ ] `loglens-web/src/handlers/websocket.rs`
- [ ] `loglens-web/README.md`
- [ ] Frontend router configuration
- [ ] Frontend navigation menu

### Files to Delete From
- [ ] `loglens-web/src/handlers/websocket.rs` (remove old functions)
- [ ] `loglens-web/src/database.rs` (remove create_tables)

---

## Appendix B: Database Schema Changes

### New Tables

**streaming_sources**:
```sql
CREATE TABLE streaming_sources (
    id TEXT PRIMARY KEY,
    project_id TEXT NOT NULL,
    name TEXT NOT NULL,
    source_type TEXT NOT NULL,
    config TEXT NOT NULL,
    parser_config TEXT,
    buffer_size INTEGER DEFAULT 100,
    batch_timeout_seconds INTEGER DEFAULT 2,
    restart_on_error BOOLEAN DEFAULT true,
    max_restarts INTEGER,
    status TEXT DEFAULT 'active',
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    FOREIGN KEY (project_id) REFERENCES projects(id)
);
```

**streaming_logs**:
```sql
CREATE TABLE streaming_logs (
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
```

### Existing Tables (Already Have Required Columns)
- `error_patterns` (category, severity already exist)
- `knowledge_base` (is_public already exists)
- `performance_metrics` (all fields exist)
- `error_correlations` (all fields exist)

---

**Plan Status**: Ready for Implementation
**Last Updated**: 2025-10-04
**Next Review**: After Phase 1 completion
