# LogLens Feature Extension Guide

This guide provides comprehensive documentation for extending features across the entire LogLens project, including backend services, frontend components, database schema, and infrastructure.

## Table of Contents

1. [Project Architecture Overview](#architecture-overview)
2. [Backend Feature Extensions](#backend-extensions)
3. [Frontend Feature Extensions](#frontend-extensions)
4. [Database Schema Extensions](#database-extensions)
5. [API Development](#api-development)
6. [Testing Guidelines](#testing-guidelines)
7. [Configuration and Deployment](#configuration-deployment)
8. [Common Extension Scenarios](#extension-scenarios)

---

## Architecture Overview {#architecture-overview}

LogLens is organized as a Rust workspace with three main crates:

```
loglens/
├── loglens-core/     # Core analysis engine and AI provider abstraction
├── loglens-web/      # Web backend with REST API and WebSocket support
├── loglens-wasm/     # WebAssembly module for client-side processing
└── frontend-react/   # React TypeScript frontend application
```

### Key Architectural Patterns

- **Workspace Structure**: Separate crates for different concerns
- **AI Provider Abstraction**: Pluggable AI backends via trait system
- **Handler-based API**: Modular request handlers with Axum framework
- **Component-based Frontend**: React with TypeScript and custom hooks
- **Migration-based Database**: SQLite with SQLx and versioned migrations

---

## Backend Feature Extensions {#backend-extensions}

### 1. Adding New AI Providers

**Location**: `loglens-core/src/ai_provider/`

**Steps**:
1. **Create Provider Module**:
```rust
// loglens-core/src/ai_provider/newprovider.rs
use async_trait::async_trait;
use crate::ai_provider::{AIProvider, AnalysisRequest, AnalysisResponse};

pub struct NewProvider {
    client: reqwest::Client,
    api_key: String,
}

#[async_trait]
impl AIProvider for NewProvider {
    async fn analyze(&self, request: AnalysisRequest) -> Result<AnalysisResponse, Box<dyn std::error::Error>> {
        // Implementation
    }
    
    fn provider_name(&self) -> &'static str {
        "newprovider"
    }
}
```

2. **Register Provider**:
```rust
// loglens-core/src/ai_provider/mod.rs
pub fn create_provider(name: &str, api_key: String) -> Result<Box<dyn AIProvider>, String> {
    match name {
        "openrouter" => Ok(Box::new(OpenRouter::new(api_key))),
        "claude" => Ok(Box::new(Claude::new(api_key))),
        "newprovider" => Ok(Box::new(NewProvider::new(api_key))),
        _ => Err(format!("Unknown provider: {}", name)),
    }
}
```

3. **Update Configuration**:
```rust
// loglens-core/src/config.rs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub default_provider: String,
    pub providers: HashMap<String, ProviderConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderConfig {
    pub api_key: Option<String>,
    pub model: Option<String>,
    pub max_tokens: Option<u32>,
    // Add provider-specific config
}
```

### 2. Adding New Analysis Types

**Location**: `loglens-core/src/analyzer/`

**Steps**:
1. **Create Analysis Module**:
```rust
// loglens-core/src/analyzer/new_analysis.rs
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewAnalysisResult {
    pub patterns: Vec<String>,
    pub metrics: NewMetrics,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewMetrics {
    pub score: f64,
    pub confidence: f32,
}

pub async fn perform_new_analysis(
    logs: &[LogEntry],
    config: &NewAnalysisConfig,
) -> Result<NewAnalysisResult, Box<dyn std::error::Error>> {
    // Implementation
}
```

2. **Integrate with Analyzer**:
```rust
// loglens-core/src/analyzer.rs
impl Analyzer {
    pub async fn analyze_with_new_type(
        &self,
        logs: &[LogEntry],
        config: NewAnalysisConfig,
    ) -> Result<AnalysisResponse, Box<dyn std::error::Error>> {
        // Combine with existing analysis pipeline
        let basic_analysis = self.ai_provider.analyze(request).await?;
        let new_analysis = perform_new_analysis(logs, &config).await?;
        
        // Merge results
        Ok(AnalysisResponse {
            basic: basic_analysis,
            new_analysis,
            // ... other fields
        })
    }
}
```

### 3. Adding New Output Formats

**Location**: `loglens-core/src/output/`

**Steps**:
1. **Create Output Module**:
```rust
// loglens-core/src/output/new_format.rs
use crate::models::AnalysisResponse;
use askama::Template;

#[derive(Template)]
#[template(path = "new_format.txt")]
pub struct NewFormatTemplate {
    pub analysis: AnalysisResponse,
    pub metadata: NewFormatMetadata,
}

pub fn generate_new_format(
    analysis: AnalysisResponse,
    metadata: NewFormatMetadata,
) -> Result<String, Box<dyn std::error::Error>> {
    let template = NewFormatTemplate { analysis, metadata };
    Ok(template.render()?)
}
```

2. **Register Format**:
```rust
// loglens-core/src/output/mod.rs
pub enum OutputFormat {
    Console,
    Html,
    Json,
    Markdown,
    NewFormat,
}

impl OutputFormat {
    pub fn generate(&self, analysis: AnalysisResponse) -> Result<String, Box<dyn std::error::Error>> {
        match self {
            OutputFormat::Console => console::generate(analysis),
            OutputFormat::NewFormat => new_format::generate(analysis),
        }
    }
}
```

### 4. Adding Web Backend Handlers

**Location**: `loglens-web/src/handlers/`

**Steps**:
1. **Create Handler Module**:
```rust
// loglens-web/src/handlers/new_feature.rs
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
};
use serde_json::json;
use crate::models::{AppResult, AppState};
use crate::database::Database;

pub async fn handle_new_feature(
    State(state): State<AppState>,
    Path(project_id): Path<i64>,
    Json(payload): Json<NewFeatureRequest>,
) -> AppResult<Json<serde_json::Value>> {
    // Implementation
    let result = perform_new_feature(&state.db, project_id, payload).await?;
    Ok(Json(json!({ "success": true, "data": result })))
}
```

2. **Add Routes**:
```rust
// loglens-web/src/routes.rs
use axum::Router;
use crate::handlers::new_feature;

pub fn create_routes() -> Router<AppState> {
    Router::new()
        .route("/api/projects/:id/new-feature", axum::routing::post(new_feature::handle_new_feature))
        // ... other routes
}
```

3. **Add Models**:
```rust
// loglens-web/src/models.rs
#[derive(Debug, Serialize, Deserialize)]
pub struct NewFeatureRequest {
    pub name: String,
    pub config: NewFeatureConfig,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NewFeatureResponse {
    pub id: i64,
    pub status: String,
    pub results: serde_json::Value,
}
```

---

## Frontend Feature Extensions {#frontend-extensions}

### 1. Adding New Pages

**Location**: `loglens-web/frontend-react/src/pages/`

**Steps**:
1. **Create Page Component**:
```tsx
// src/pages/NewFeature.tsx
import React, { useEffect, useState } from 'react';
import { useParams } from 'react-router-dom';
import { logger } from '../utils/logger';
import { api } from '../services/api';

interface NewFeatureData {
  id: number;
  name: string;
  status: string;
}

export const NewFeature: React.FC = () => {
  const { projectId } = useParams<{ projectId: string }>();
  const [data, setData] = useState<NewFeatureData | null>(null);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    const loadData = async () => {
      try {
        logger.info('NewFeature', 'Loading data', { projectId });
        const response = await api.get(`/projects/${projectId}/new-feature`);
        setData(response.data);
      } catch (error) {
        logger.error('NewFeature', 'Failed to load data', { error });
      } finally {
        setLoading(false);
      }
    };

    loadData();
  }, [projectId]);

  if (loading) return <LoadingSpinner />;
  if (!data) return <div>No data available</div>;

  return (
    <div className="p-6">
      <h1 className="text-2xl font-bold mb-4">New Feature</h1>
      {/* Component content */}
    </div>
  );
};
```

2. **Add Routing**:
```tsx
// src/App.tsx
import { NewFeature } from './pages/NewFeature';

function App() {
  return (
    <Routes>
      {/* Existing routes */}
      <Route path="/projects/:id/new-feature" element={<NewFeature />} />
    </Routes>
  );
}
```

### 2. Creating Custom Hooks

**Location**: `loglens-web/frontend-react/src/hooks/`

**Steps**:
1. **Create Custom Hook**:
```tsx
// src/hooks/useNewFeature.tsx
import { useState, useEffect } from 'react';
import { api } from '../services/api';
import { logger } from '../utils/logger';

interface NewFeatureConfig {
  enabled: boolean;
  threshold: number;
}

interface NewFeatureResult {
  id: number;
  score: number;
  recommendations: string[];
}

export const useNewFeature = (projectId: string) => {
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [results, setResults] = useState<NewFeatureResult[]>([]);

  const executeNewFeature = async (config: NewFeatureConfig) => {
    setLoading(true);
    setError(null);

    try {
      logger.info('useNewFeature', 'Executing feature', { projectId, config });
      const response = await api.post(`/projects/${projectId}/new-feature`, config);
      setResults(response.data);
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : 'Unknown error';
      setError(errorMessage);
      logger.error('useNewFeature', 'Execution failed', { error: err });
    } finally {
      setLoading(false);
    }
  };

  return {
    executeNewFeature,
    loading,
    error,
    results,
  };
};
```

### 3. Adding New Components

**Location**: `loglens-web/frontend-react/src/components/`

**Steps**:
1. **Create Reusable Component**:
```tsx
// src/components/NewFeatureComponent.tsx
import React from 'react';
import { NewFeatureResult } from '../types';

interface NewFeatureComponentProps {
  results: NewFeatureResult[];
  onResultClick: (result: NewFeatureResult) => void;
}

export const NewFeatureComponent: React.FC<NewFeatureComponentProps> = ({
  results,
  onResultClick,
}) => {
  return (
    <div className="bg-white rounded-lg shadow p-4">
      <h3 className="text-lg font-semibold mb-3">New Feature Results</h3>
      <div className="space-y-2">
        {results.map((result) => (
          <div
            key={result.id}
            className="p-3 border rounded hover:bg-gray-50 cursor-pointer"
            onClick={() => onResultClick(result)}
          >
            <div className="flex justify-between items-center">
              <span className="font-medium">Result #{result.id}</span>
              <span className="text-sm text-gray-500">Score: {result.score}</span>
            </div>
            <div className="mt-2">
              <span className="text-sm text-gray-600">
                {result.recommendations.length} recommendations
              </span>
            </div>
          </div>
        ))}
      </div>
    </div>
  );
};
```

### 4. Extending API Services

**Location**: `loglens-web/frontend-react/src/services/api.ts`

**Steps**:
1. **Add API Methods**:
```typescript
// src/services/api.ts
export const api = {
  // Existing methods...

  newFeature: {
    execute: (projectId: string, config: NewFeatureConfig) =>
      axios.post(`/api/projects/${projectId}/new-feature`, config),
    
    getResults: (projectId: string, analysisId: string) =>
      axios.get(`/api/projects/${projectId}/new-feature/${analysisId}`),
    
    export: (projectId: string, format: 'json' | 'csv' | 'pdf') =>
      axios.get(`/api/projects/${projectId}/new-feature/export`, {
        params: { format },
        responseType: 'blob',
      }),
  },
};
```

### 5. Adding TypeScript Types

**Location**: `loglens-web/frontend-react/src/types/`

**Steps**:
1. **Extend Type Definitions**:
```typescript
// src/types/index.ts

// New feature types
export interface NewFeatureConfig {
  enabled: boolean;
  threshold: number;
  options?: Record<string, any>;
}

export interface NewFeatureResult {
  id: number;
  timestamp: string;
  score: number;
  confidence: number;
  recommendations: string[];
  metadata?: Record<string, any>;
}

// Extend existing types
export interface Analysis {
  // Existing fields...
  newFeatureData?: NewFeatureResult[];
}

export interface Project {
  // Existing fields...
  newFeatureEnabled?: boolean;
  newFeatureConfig?: NewFeatureConfig;
}
```

---

## Database Schema Extensions {#database-extensions}

### 1. Creating New Migrations

**Location**: `loglens-web/migrations/`

**Steps**:
1. **Create Migration File**:
```sql
-- migrations/20240101000005_new_feature.sql
-- Migration: Add new feature support

CREATE TABLE new_feature_analyses (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    project_id INTEGER NOT NULL,
    analysis_id INTEGER NOT NULL,
    config TEXT NOT NULL, -- JSON config
    results TEXT NOT NULL, -- JSON results
    score REAL,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    
    FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE CASCADE,
    FOREIGN KEY (analysis_id) REFERENCES analyses(id) ON DELETE CASCADE
);

-- Add new feature config to projects table
ALTER TABLE projects ADD COLUMN new_feature_config TEXT; -- JSON
ALTER TABLE projects ADD COLUMN new_feature_enabled BOOLEAN DEFAULT FALSE;

-- Indexes for performance
CREATE INDEX idx_new_feature_analyses_project_id ON new_feature_analyses(project_id);
CREATE INDEX idx_new_feature_analyses_analysis_id ON new_feature_analyses(analysis_id);
```

2. **Create Rollback Migration** (optional):
```sql
-- migrations/20240101000005_new_feature_down.sql
-- Rollback: Remove new feature support

DROP TABLE IF EXISTS new_feature_analyses;
-- Note: SQLite doesn't support DROP COLUMN, so this would require table recreation
```

### 2. Updating Database Models

**Location**: `loglens-web/src/models.rs` or `loglens-web/src/database.rs`

**Steps**:
1. **Add Struct Definitions**:
```rust
// loglens-web/src/models.rs
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct NewFeatureAnalysis {
    pub id: Option<i64>,
    pub project_id: i64,
    pub analysis_id: i64,
    pub config: String, // JSON
    pub results: String, // JSON
    pub score: Option<f64>,
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
    pub updated_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NewFeatureConfig {
    pub enabled: bool,
    pub threshold: f64,
    pub options: Option<serde_json::Value>,
}
```

2. **Add Database Functions**:
```rust
// loglens-web/src/database.rs
impl Database {
    pub async fn create_new_feature_analysis(
        &self,
        analysis: &NewFeatureAnalysis,
    ) -> Result<i64, sqlx::Error> {
        let result = sqlx::query!(
            r#"
            INSERT INTO new_feature_analyses 
            (project_id, analysis_id, config, results, score)
            VALUES (?, ?, ?, ?, ?)
            "#,
            analysis.project_id,
            analysis.analysis_id,
            analysis.config,
            analysis.results,
            analysis.score
        )
        .execute(&self.pool)
        .await?;

        Ok(result.last_insert_rowid())
    }

    pub async fn get_new_feature_analyses(
        &self,
        project_id: i64,
    ) -> Result<Vec<NewFeatureAnalysis>, sqlx::Error> {
        let analyses = sqlx::query_as!(
            NewFeatureAnalysis,
            r#"
            SELECT * FROM new_feature_analyses 
            WHERE project_id = ? 
            ORDER BY created_at DESC
            "#,
            project_id
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(analyses)
    }
}
```

### 3. Database Query Patterns

**Location**: `loglens-web/src/handlers/` or dedicated query modules

**Steps**:
1. **Implement Complex Queries**:
```rust
// src/handlers/new_feature.rs
use crate::database::Database;

pub async fn get_feature_analytics(
    db: &Database,
    project_id: i64,
    date_range: Option<(chrono::DateTime<chrono::Utc>, chrono::DateTime<chrono::Utc>)>,
) -> Result<FeatureAnalytics, Box<dyn std::error::Error>> {
    let query = match date_range {
        Some((start, end)) => {
            sqlx::query_as!(
                FeatureAnalyticsRow,
                r#"
                SELECT 
                    COUNT(*) as total_analyses,
                    AVG(score) as avg_score,
                    MAX(score) as max_score,
                    MIN(score) as min_score
                FROM new_feature_analyses 
                WHERE project_id = ? 
                AND created_at BETWEEN ? AND ?
                "#,
                project_id,
                start,
                end
            )
            .fetch_one(&db.pool)
            .await?
        }
        None => {
            sqlx::query_as!(
                FeatureAnalyticsRow,
                r#"
                SELECT 
                    COUNT(*) as total_analyses,
                    AVG(score) as avg_score,
                    MAX(score) as max_score,
                    MIN(score) as min_score
                FROM new_feature_analyses 
                WHERE project_id = ?
                "#,
                project_id
            )
            .fetch_one(&db.pool)
            .await?
        }
    };

    Ok(FeatureAnalytics::from(query))
}
```

---

## API Development {#api-development}

### 1. RESTful API Design

**Location**: `loglens-web/src/handlers/` and `loglens-web/src/routes.rs`

**Design Principles**:
- Use consistent URL patterns: `/api/resource/:id/subresource`
- Implement proper HTTP methods (GET, POST, PUT, DELETE)
- Return consistent JSON responses with metadata
- Handle errors with appropriate status codes

**Example Implementation**:
```rust
// src/handlers/new_feature.rs
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
};
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Deserialize)]
pub struct NewFeatureQuery {
    pub limit: Option<u32>,
    pub offset: Option<u32>,
    pub sort_by: Option<String>,
}

pub async fn get_new_features(
    State(state): State<AppState>,
    Path(project_id): Path<i64>,
    Query(params): Query<NewFeatureQuery>,
) -> AppResult<Json<ApiResponse<Vec<NewFeatureResponse>>>> {
    let limit = params.limit.unwrap_or(20).min(100);
    let offset = params.offset.unwrap_or(0);
    
    let features = state.db
        .get_new_features_paginated(project_id, limit, offset)
        .await?;
    
    Ok(Json(ApiResponse::success(features)))
}

pub async fn create_new_feature(
    State(state): State<AppState>,
    Path(project_id): Path<i64>,
    Json(request): Json<CreateNewFeatureRequest>,
) -> AppResult<Json<ApiResponse<NewFeatureResponse>>> {
    // Validate request
    if let Err(validation) = validate_new_feature_request(&request) {
        return Ok(Json(ApiResponse::error(
            StatusCode::BAD_REQUEST,
            validation.to_string(),
        )));
    }
    
    // Create feature
    let feature = state.db
        .create_new_feature(project_id, request)
        .await?;
    
    Ok(Json(ApiResponse::success(feature)))
}
```

### 2. WebSocket Extensions

**Location**: `loglens-web/src/handlers/websocket.rs`

**Steps**:
1. **Add New WebSocket Message Types**:
```rust
// src/handlers/websocket.rs
use serde_json::json;
use tokio_tungstenite::tungstenite::Message;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum WsMessage {
    // Existing message types...
    NewFeatureStart { project_id: i64, config: NewFeatureConfig },
    NewFeatureProgress { progress: f64, current: String },
    NewFeatureComplete { results: NewFeatureResponse },
    NewFeatureError { error: String },
}

pub async fn handle_websocket_message(
    msg: Message,
    state: &AppState,
    tx: &mpsc::UnboundedSender<Message>,
) -> Result<(), Box<dyn std::error::Error>> {
    let text = msg.to_text()?;
    let ws_msg: WsMessage = serde_json::from_str(text)?;
    
    match ws_msg {
        WsMessage::NewFeatureStart { project_id, config } => {
            // Start async new feature analysis
            let tx = tx.clone();
            let db = state.db.clone();
            
            tokio::spawn(async move {
                if let Err(e) = perform_new_feature_analysis(
                    project_id, 
                    config, 
                    &db, 
                    tx
                ).await {
                    let error_msg = WsMessage::NewFeatureError { 
                        error: e.to_string() 
                    };
                    let _ = tx.send(Message::text(serde_json::to_string(&error_msg)?));
                }
            });
        }
        // Handle other message types...
    }
    
    Ok(())
}
```

### 3. API Documentation and Validation

**Location**: Handler files and `loglens-web/src/validation.rs`

**Steps**:
1. **Add Request Validation**:
```rust
// src/validation.rs
use validator::{Validate, ValidationError};

#[derive(Debug, Deserialize, Validate)]
pub struct CreateNewFeatureRequest {
    #[validate(length(min = 1, max = 100))]
    pub name: String,
    
    #[validate(range(min = 0.0, max = 1.0))]
    pub threshold: f64,
    
    #[validate(custom = "validate_config")]
    pub config: serde_json::Value,
}

fn validate_config(config: &serde_json::Value) -> Result<(), ValidationError> {
    // Custom validation logic
    if !config.is_object() {
        return Err(ValidationError::new("config must be an object"));
    }
    Ok(())
}
```

---

## Testing Guidelines {#testing-guidelines}

### 1. Backend Testing

**Location**: `loglens-web/tests/`

**Integration Tests**:
```rust
// tests/api_integration.rs
use axum_test::TestServer;
use loglens_web::{create_app, AppState};
use tempfile::TempDir;

#[tokio::test]
async fn test_new_feature_api() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("test.db");
    let state = AppState::new_for_testing(&db_path).await.unwrap();
    let app = create_app(state);
    
    let server = TestServer::new(app).unwrap();
    
    // Test creating new feature
    let response = server
        .post("/api/projects/1/new-feature")
        .json(&serde_json::json!({
            "name": "Test Feature",
            "threshold": 0.8
        }))
        .await;
    
    assert_eq!(response.status_code(), 201);
    
    let body: serde_json::Value = response.json();
    assert_eq!(body["success"], true);
    assert!(body["data"]["id"].is_number());
}
```

**Unit Tests**:
```rust
// src/handlers/new_feature.rs
#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::create_test_database;
    
    #[tokio::test]
    async fn test_create_new_feature() {
        let db = create_test_database().await;
        let request = CreateNewFeatureRequest {
            name: "Test".to_string(),
            threshold: 0.5,
        };
        
        let result = db.create_new_feature(1, request).await.unwrap();
        assert!(result.id > 0);
        assert_eq!(result.name, "Test");
    }
}
```

### 2. Frontend Testing

**Location**: `loglens-web/frontend-react/src/`

**Component Tests**:
```tsx
// src/components/__tests__/NewFeatureComponent.test.tsx
import React from 'react';
import { render, screen, fireEvent } from '@testing-library/react';
import { NewFeatureComponent } from '../NewFeatureComponent';

const mockResults = [
  {
    id: 1,
    score: 0.8,
    recommendations: ['Test recommendation'],
  },
];

describe('NewFeatureComponent', () => {
  it('renders results correctly', () => {
    const mockOnClick = jest.fn();
    render(
      <NewFeatureComponent 
        results={mockResults} 
        onResultClick={mockOnClick} 
      />
    );
    
    expect(screen.getByText('New Feature Results')).toBeInTheDocument();
    expect(screen.getByText('Result #1')).toBeInTheDocument();
    expect(screen.getByText('Score: 0.8')).toBeInTheDocument();
  });
  
  it('calls onResultClick when result is clicked', () => {
    const mockOnClick = jest.fn();
    render(
      <NewFeatureComponent 
        results={mockResults} 
        onResultClick={mockOnClick} 
      />
    );
    
    fireEvent.click(screen.getByText('Result #1'));
    expect(mockOnClick).toHaveBeenCalledWith(mockResults[0]);
  });
});
```

**Hook Tests**:
```tsx
// src/hooks/__tests__/useNewFeature.test.tsx
import { renderHook, act } from '@testing-library/react';
import { useNewFeature } from '../useNewFeature';
import * as api from '../../services/api';

jest.mock('../../services/api');

describe('useNewFeature', () => {
  beforeEach(() => {
    jest.clearAllMocks();
  });
  
  it('executes new feature successfully', async () => {
    const mockResponse = { data: [{ id: 1, score: 0.9 }] };
    (api.post as jest.Mock).mockResolvedValue(mockResponse);
    
    const { result } = renderHook(() => useNewFeature('1'));
    
    await act(async () => {
      await result.current.executeNewFeature({ enabled: true, threshold: 0.8 });
    });
    
    expect(result.current.loading).toBe(false);
    expect(result.current.error).toBe(null);
    expect(result.current.results).toEqual(mockResponse.data);
  });
});
```

### 3. E2E Testing

**Location**: `loglens-web/frontend-react/cypress/`

```typescript
// cypress/e2e/new_feature.cy.ts
describe('New Feature E2E', () => {
  beforeEach(() => {
    cy.login();
    cy.visit('/projects/1/new-feature');
  });
  
  it('should create and display new feature analysis', () => {
    cy.get('[data-testid="feature-name-input"]').type('Test Feature');
    cy.get('[data-testid="threshold-slider"]').set(80);
    cy.get('[data-testid="submit-button"]').click();
    
    cy.get('[data-testid="loading-spinner"]').should('be.visible');
    cy.get('[data-testid="results-container"]', { timeout: 10000 }).should('be.visible');
    
    cy.get('[data-testid="result-item"]').should('have.length.greaterThan', 0);
    cy.get('[data-testid="export-button"]').should('be.visible');
  });
});
```

---

## Configuration and Deployment {#configuration-deployment}

### 1. Configuration Extensions

**Location**: `loglens-web/src/config.rs`

**Steps**:
1. **Add Configuration Fields**:
```rust
// src/config.rs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebConfig {
    // Existing fields...
    
    /// New feature configuration
    pub new_feature: NewFeatureConfig,
    
    /// Feature flags
    pub features: FeatureFlags,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewFeatureConfig {
    pub enabled: bool,
    pub max_concurrent_analyses: usize,
    pub default_threshold: f64,
    pub cache_ttl_seconds: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureFlags {
    pub new_feature_ui: bool,
    pub advanced_analytics: bool,
    pub real_time_updates: bool,
}

impl Default for WebConfig {
    fn default() -> Self {
        Self {
            // Existing defaults...
            new_feature: NewFeatureConfig {
                enabled: true,
                max_concurrent_analyses: 5,
                default_threshold: 0.7,
                cache_ttl_seconds: 3600,
            },
            features: FeatureFlags {
                new_feature_ui: true,
                advanced_analytics: false,
                real_time_updates: true,
            },
        }
    }
}
```

2. **Environment Variable Support**:
```rust
// src/config.rs
impl WebConfig {
    pub fn from_env() -> Result<Self, Box<dyn std::error::Error>> {
        let mut config = Self::default();
        
        // Override with environment variables
        if let Ok(val) = std::env::var("NEW_FEATURE_ENABLED") {
            config.new_feature.enabled = val.parse()?;
        }
        
        if let Ok(val) = std::env::var("MAX_CONCURRENT_ANALYSES") {
            config.new_feature.max_concurrent_analyses = val.parse()?;
        }
        
        Ok(config)
    }
}
```

### 2. Docker Extensions

**Location**: `Dockerfile` and `docker-compose.yml`

**Multi-stage Build Extensions**:
```dockerfile
# Dockerfile
# Add new build stage for new feature dependencies
FROM node:18-alpine AS new-feature-builder
WORKDIR /app/new-feature
COPY package*.json ./
RUN npm ci --only=production

# Main application stage
FROM rust:1.75-alpine AS builder
# ... existing stages ...

# Final stage
FROM debian:bookworm-slim AS runtime
# ... existing setup ...

# Copy new feature assets
COPY --from=new-feature-builder /app/new-feature ./new-feature-assets

# Environment variables for new features
ENV NEW_FEATURE_ENABLED=true
ENV MAX_CONCURRENT_ANALYSES=10
ENV FEATURE_NEW_UI=true
```

**Docker Compose Extensions**:
```yaml
# docker-compose.yml
services:
  loglens:
    build: .
    environment:
      - NEW_FEATURE_ENABLED=${NEW_FEATURE_ENABLED:-true}
      - MAX_CONCURRENT_ANALYSES=${MAX_CONCURRENT_ANALYSES:-5}
      - REDIS_URL=${REDIS_URL:-redis://redis:6379}
    volumes:
      - ./data:/app/data
      - ./new-feature-config:/app/config
    depends_on:
      - redis
      - postgres

  # Add Redis for caching new feature results
  redis:
    image: redis:7-alpine
    volumes:
      - redis_data:/data

  # Add PostgreSQL for analytics (optional)
  postgres:
    image: postgres:15-alpine
    environment:
      POSTGRES_DB: loglens_analytics
      POSTGRES_USER: loglens
      POSTGRES_PASSWORD: ${POSTGRES_PASSWORD}
    volumes:
      - postgres_data:/var/lib/postgresql/data

volumes:
  redis_data:
  postgres_data:
```

### 3. Monitoring and Observability

**Location**: `loglens-web/src/middleware/` and handlers

**Metrics Extensions**:
```rust
// src/middleware/metrics.rs
use prometheus::{Counter, Histogram, Registry, TextEncoder, Encoder};

pub struct Metrics {
    pub new_feature_requests_total: Counter,
    pub new_feature_duration: Histogram,
    pub new_feature_errors_total: Counter,
}

impl Metrics {
    pub fn new() -> Self {
        Self {
            new_feature_requests_total: Counter::new(
                "new_feature_requests_total",
                "Total number of new feature requests"
            ).unwrap(),
            new_feature_duration: Histogram::with_opts(
                prometheus::HistogramOpts::new(
                    "new_feature_duration_seconds",
                    "Duration of new feature analysis"
                ).buckets(vec![1.0, 5.0, 10.0, 30.0, 60.0, 300.0])
            ).unwrap(),
            new_feature_errors_total: Counter::new(
                "new_feature_errors_total",
                "Total number of new feature errors"
            ).unwrap(),
        }
    }
}

// Usage in handlers
pub async fn handle_new_feature_with_metrics(
    State(state): State<AppState>,
    // ... other parameters
) -> AppResult<Json<ApiResponse<NewFeatureResponse>>> {
    let timer = state.metrics.new_feature_duration.start_timer();
    state.metrics.new_feature_requests_total.inc();
    
    let result = match perform_new_feature_analysis(&state, request).await {
        Ok(r) => r,
        Err(e) => {
            state.metrics.new_feature_errors_total.inc();
            return Err(e.into());
        }
    };
    
    timer.observe_duration();
    Ok(Json(ApiResponse::success(result)))
}
```

---

## Common Extension Scenarios {#extension-scenarios}

### 1. Adding a New Log Analysis Type

**Backend Changes**:
1. Create analyzer in `loglens-core/src/analyzer/new_analysis.rs`
2. Add configuration to `loglens-core/src/config.rs`
3. Update API handler in `loglens-web/src/handlers/analysis.rs`
4. Add database migration for new analysis results

**Frontend Changes**:
1. Create page component in `src/pages/NewAnalysis.tsx`
2. Add custom hook in `src/hooks/useNewAnalysis.tsx`
3. Create result components in `src/components/NewAnalysisResults.tsx`
4. Add routing and navigation

**Testing**:
1. Unit tests for analyzer logic
2. Integration tests for API endpoints
3. Component tests for UI elements
4. E2E tests for complete workflow

### 2. Adding Real-time Log Streaming

**Backend Changes**:
1. Implement WebSocket handlers in `loglens-web/src/handlers/streaming.rs`
2. Add streaming sources in `loglens-web/src/streaming/`
3. Create streaming configuration in `loglens-web/src/config.rs`

**Frontend Changes**:
1. Add WebSocket hook in `src/hooks/useStreaming.tsx`
2. Create streaming UI components
3. Add real-time chart updates
4. Implement connection management

**Infrastructure**:
1. Add Redis for pub/sub
2. Configure message queuing
3. Add monitoring for stream health
4. Implement backpressure handling

### 3. Adding Team Collaboration Features

**Backend Changes**:
1. Create user management system
2. Add permissions and sharing models
3. Implement collaboration API endpoints
4. Add notification system

**Frontend Changes**:
1. Add user authentication UI
2. Create sharing components
3. Implement real-time collaboration
4. Add notification system

**Database Changes**:
1. Add users and teams tables
2. Create permissions schema
3. Add activity logging
4. Implement sharing relationships

### 4. Adding Advanced Export Formats

**Backend Changes**:
1. Create export templates in `loglens-core/templates/`
2. Add export handlers in `loglens-web/src/handlers/export.rs`
3. Implement format-specific rendering
4. Add background job processing

**Frontend Changes**:
1. Add export configuration UI
2. Create format preview components
3. Implement download management
4. Add export history tracking

### 5. Adding Machine Learning Model Integration

**Backend Changes**:
1. Add ML model service interface
2. Implement model training pipeline
3. Create prediction API endpoints
4. Add model versioning

**Frontend Changes**:
1. Add model training UI
2. Create prediction result displays
3. Implement model performance charts
4. Add model management interface

**Infrastructure**:
1. Add GPU support if needed
2. Configure model storage
3. Add model serving endpoints
4. Implement monitoring and logging

---

## Best Practices and Guidelines

### Code Quality
- Follow Rust naming conventions and clippy lints
- Use TypeScript strict mode for frontend
- Implement comprehensive error handling
- Write meaningful unit and integration tests
- Document public APIs and complex algorithms

### Performance
- Use connection pooling for database operations
- Implement caching for frequently accessed data
- Optimize database queries with proper indexes
- Use async/await for I/O operations
- Monitor and profile application performance

### Security
- Validate all user inputs
- Use parameterized queries to prevent SQL injection
- Implement proper authentication and authorization
- Keep dependencies updated
- Follow secure coding practices

### Monitoring and Debugging
- Add structured logging throughout the application
- Implement health check endpoints
- Use metrics to monitor application performance
- Add error tracking and alerting
- Document troubleshooting procedures

---

This extension guide provides a comprehensive foundation for adding new features to LogLens while maintaining code quality, performance, and reliability. The modular architecture makes it straightforward to extend functionality across all components of the system.