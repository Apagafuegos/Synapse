# Synapse API Documentation

This document provides comprehensive API documentation for Synapse, including all endpoints for the web server, streaming, knowledge base, MCP integration, and export functionality.

## Base URL
```
http://localhost:3000
```

## Authentication
Synapse uses API key-based authentication. Configure API keys through the settings endpoint.

## Response Format
All responses use JSON format with the following structure:
```json
{
  "success": true,
  "data": {},
  "error": null,
  "timestamp": "2024-01-01T00:00:00Z"
}
```

---

## Projects API

### List All Projects
```http
GET /api/projects
```

**Query Parameters:**
- `page` (optional): Page number for pagination
- `limit` (optional): Number of projects per page
- `search` (optional): Search term for project names

**Response:**
```json
{
  "success": true,
  "data": {
    "projects": [
      {
        "id": "uuid",
        "name": "Project Name",
        "description": "Project description",
        "created_at": "2024-01-01T00:00:00Z",
        "updated_at": "2024-01-01T00:00:00Z",
        "file_count": 5,
        "analysis_count": 12
      }
    ],
    "total": 25,
    "page": 1,
    "limit": 10
  }
}
```

### Create Project
```http
POST /api/projects
```

**Request Body:**
```json
{
  "name": "Project Name",
  "description": "Project description",
  "metadata": {
    "environment": "production",
    "team": "backend"
  }
}
```

### Get Project Details
```http
GET /api/projects/{id}
```

### Update Project
```http
PUT /api/projects/{id}
```

### Delete Project
```http
DELETE /api/projects/{id}
```

---

## Files API

### List Project Files
```http
GET /api/projects/{id}/files
```

**Response:**
```json
{
  "success": true,
  "data": {
    "files": [
      {
        "id": "uuid",
        "filename": "app.log",
        "size": 1024000,
        "line_count": 5000,
        "created_at": "2024-01-01T00:00:00Z",
        "analysis_count": 3
      }
    ]
  }
}
```

### Upload File
```http
POST /api/projects/{id}/files
```

**Content-Type:** `multipart/form-data`

**Form Data:**
- `file`: Log file (.log, .txt formats)
- `description`: Optional file description

### Delete File
```http
DELETE /api/projects/{id}/files/{file_id}
```

---

## Analysis API

### Start Analysis
```http
POST /api/projects/{id}/files/{file_id}/analyze
```

**Request Body:**
```json
{
  "provider": "openrouter",
  "model": "anthropic/claude-3-haiku",
  "level": "ERROR",
  "max_lines": 1000,
  "options": {
    "include_patterns": true,
    "include_correlations": true,
    "include_anomalies": true,
    "include_performance": true
  }
}
```

**Response:**
```json
{
  "success": true,
  "data": {
    "analysis_id": "uuid",
    "status": "started",
    "estimated_duration": 120
  }
}
```

### List Project Analyses
```http
GET /api/projects/{id}/analyses
```

**Query Parameters:**
- `status` (optional): Filter by status (running, completed, failed)
- `provider` (optional): Filter by AI provider
- `limit` (optional): Number of analyses per page
- `offset` (optional): Offset for pagination

### Get Analysis Details
```http
GET /api/analyses/{id}
```

**Response:**
```json
{
  "success": true,
  "data": {
    "id": "uuid",
    "status": "completed",
    "provider": "openrouter",
    "model": "anthropic/claude-3-haiku",
    "created_at": "2024-01-01T00:00:00Z",
    "completed_at": "2024-01-01T00:02:00Z",
    "duration": 120,
    "results": {
      "executive_summary": {
        "total_errors": 150,
        "critical_errors": 5,
        "confidence_score": 0.85
      },
      "error_analysis": {
        "categories": {
          "infrastructure": 80,
          "code": 45,
          "external": 25
        }
      },
      "patterns": [
        {
          "pattern": "database connection timeout",
          "frequency": 25,
          "confidence": 0.9,
          "severity": "high",
          "examples": ["[ERROR] Database connection timed out after 30s"]
        }
      ],
      "performance": {
        "bottlenecks": [
          {
            "component": "database",
            "impact_score": 0.8,
            "recommendations": ["Increase connection pool size"]
          }
        ]
      },
      "anomalies": [
        {
          "type": "timing",
          "confidence": 0.75,
          "description": "Unusual spike in response times"
        }
      ],
      "correlations": [
        {
          "errors": ["Database timeout", "API response failure"],
          "strength": 0.8,
          "description": "Database timeouts correlate with API failures"
        }
      ]
    }
  }
}
```

### Stream Analysis Results
```http
GET /api/analyses/{id}/stream
```

**Accept:** `text/event-stream`

**Server-Sent Events:**
```
data: {"type": "progress", "stage": "parsing", "progress": 25}
data: {"type": "progress", "stage": "analysis", "progress": 50}
data: {"type": "result", "data": {...}}
data: {"type": "complete", "analysis_id": "uuid"}
```

### Cancel Analysis
```http
POST /api/analyses/{id}/cancel
```

### Delete Analysis
```http
DELETE /api/analyses/{id}
```

---

## Streaming API

### List Streaming Sources
```http
GET /api/streaming/sources
```

**Response:**
```json
{
  "success": true,
  "data": {
    "sources": [
      {
        "id": "uuid",
        "name": "app-logs",
        "source_type": "file",
        "status": "active",
        "project_id": "uuid",
        "config": {
          "file_path": "/var/log/app.log",
          "buffer_size": 1000
        },
        "statistics": {
          "lines_processed": 50000,
          "connection_count": 1,
          "last_activity": "2024-01-01T00:00:00Z"
        },
        "created_at": "2024-01-01T00:00:00Z"
      }
    ]
  }
}
```

### Create Streaming Source
```http
POST /api/streaming/sources
```

**Request Body:**
```json
{
  "name": "app-logs",
  "source_type": "file",
  "project_id": "uuid",
  "config": {
    "file_path": "/var/log/app.log",
    "buffer_size": 1000,
    "timeout": 30,
    "parser": {
      "format": "json",
      "timestamp_field": "timestamp",
      "level_field": "level"
    }
  }
}
```

**Source Types:**
- `file`: Tail log files
- `command`: Execute and stream command output
- `tcp`: TCP listener for log streams
- `http`: HTTP endpoint for log reception
- `stdin`: Standard input streaming

### Get Streaming Source Details
```http
GET /api/streaming/sources/{id}
```

### Delete Streaming Source
```http
DELETE /api/streaming/sources/{id}
```

### Get Streaming Statistics
```http
GET /api/streaming/stats
```

**Response:**
```json
{
  "success": true,
  "data": {
    "total_sources": 5,
    "active_sources": 3,
    "total_lines_processed": 1000000,
    "average_processing_rate": 500.5,
    "by_type": {
      "file": 2,
      "command": 1,
      "tcp": 2
    }
  }
}
```

### Restart Streaming Source
```http
POST /api/streaming/sources/{id}/restart
```

---

## Knowledge Base API

### List Knowledge Entries
```http
GET /api/knowledge
```

**Query Parameters:**
- `category` (optional): Filter by category
- `tags` (optional): Filter by tags (comma-separated)
- `public` (optional): Filter by public status
- `search` (optional): Search term
- `limit` (optional): Number of entries per page

**Response:**
```json
{
  "success": true,
  "data": {
    "entries": [
      {
        "id": "uuid",
        "title": "Database Connection Timeout",
        "problem": "Application experiences intermittent database connection timeouts",
        "solution": "Increase connection pool size and implement connection retry logic",
        "category": "infrastructure",
        "tags": ["database", "timeout", "connection-pool"],
        "public": true,
        "usage_count": 15,
        "created_at": "2024-01-01T00:00:00Z",
        "updated_at": "2024-01-01T00:00:00Z"
      }
    ],
    "total": 50,
    "page": 1,
    "limit": 10
  }
}
```

### Create Knowledge Entry
```http
POST /api/knowledge
```

**Request Body:**
```json
{
  "title": "Database Connection Timeout",
  "problem": "Application experiences intermittent database connection timeouts",
  "solution": "Increase connection pool size and implement connection retry logic",
  "category": "infrastructure",
  "tags": ["database", "timeout", "connection-pool"],
  "public": true,
  "related_patterns": ["connection-refused", "timeout-error"]
}
```

### Get Knowledge Entry
```http
GET /api/knowledge/{id}
```

### Update Knowledge Entry
```http
PUT /api/knowledge/{id}
```

### Delete Knowledge Entry
```http
DELETE /api/knowledge/{id}
```

### Search Knowledge Base
```http
GET /api/knowledge/search
```

**Query Parameters:**
- `q`: Search query (required)
- `category` (optional): Filter by category
- `tags` (optional): Filter by tags
- `limit` (optional): Maximum number of results

### Share Knowledge Entry
```http
POST /api/knowledge/{id}/share
```

**Request Body:**
```json
{
  "public": true,
  "share_token": "optional-token"
}
```

---

## Export API

### Export Analysis
```http
POST /api/analyses/{id}/export
```

**Request Body:**
```json
{
  "format": "html",
  "include_charts": true,
  "include_correlations": true,
  "template": "default",
  "metadata": {
    "title": "Analysis Report",
    "description": "Comprehensive log analysis report"
  }
}
```

**Formats:**
- `html`: HTML report with charts
- `pdf`: PDF report
- `json`: Structured JSON data
- `csv`: CSV format for spreadsheets
- `markdown`: Markdown documentation

**Response:**
```json
{
  "success": true,
  "data": {
    "export_id": "uuid",
    "status": "processing",
    "format": "html",
    "estimated_size": 2048000
  }
}
```

### Get Export Status
```http
GET /api/exports/{id}
```

**Response:**
```json
{
  "success": true,
  "data": {
    "id": "uuid",
    "status": "completed",
    "format": "html",
    "size": 2048000,
    "download_url": "/api/exports/{id}/download",
    "expires_at": "2024-01-08T00:00:00Z",
    "created_at": "2024-01-01T00:00:00Z"
  }
}
```

### Download Export
```http
GET /api/exports/{id}/download
```

**Content-Type:** Varies by format

### Create Shareable Link
```http
POST /api/exports/{id}/share
```

**Request Body:**
```json
{
  "expires_in": "7d",
  "password": "optional-password",
  "allow_download": true,
  "max_downloads": 10
}
```

---

## Settings API

### Get Settings
```http
GET /api/settings
```

**Response:**
```json
{
  "success": true,
  "data": {
    "ai_providers": {
      "openrouter": {
        "api_key": "encrypted-key",
        "models": ["anthropic/claude-3-haiku"],
        "enabled": true
      },
      "openai": {
        "api_key": "encrypted-key",
        "models": ["gpt-4"],
        "enabled": false
      }
    },
    "analysis": {
      "default_provider": "openrouter",
      "default_model": "anthropic/claude-3-haiku",
      "max_lines": 5000,
      "timeout": 600
    },
    "ui": {
      "theme": "dark",
      "show_timestamps": true,
      "show_line_numbers": true
    },
    "streaming": {
      "default_buffer_size": 1000,
      "default_timeout": 30,
      "auto_restart": true
    }
  }
}
```

### Update Settings
```http
PUT /api/settings
```

**Request Body:**
```json
{
  "ai_providers": {
    "openrouter": {
      "api_key": "sk-or-new-key",
      "enabled": true
    }
  },
  "analysis": {
    "max_lines": 10000,
    "timeout": 900
  }
}
```

### Fetch Available Models
```http
POST /api/settings/models/fetch
```

**Request Body:**
```json
{
  "provider": "openrouter"
}
```

---

## MCP Integration API

### MCP Analysis Endpoint
```http
POST /api/mcp/analyze
```

**Request Body:**
```json
{
  "logs": ["[ERROR] Database connection failed", "[WARN] Retry attempt 1"],
  "level": "ERROR",
  "provider": "openrouter",
  "options": {
    "include_patterns": true,
    "include_correlations": true,
    "max_lines": 100
  }
}
```

### List Available MCP Tools
```http
GET /api/mcp/tools
```

**Response:**
```json
{
  "success": true,
  "data": {
    "tools": [
      {
        "name": "analyze_logs",
        "description": "Analyze log content with AI",
        "parameters": {
          "logs": {"type": "array", "required": true},
          "level": {"type": "string", "required": false},
          "provider": {"type": "string", "required": false}
        }
      }
    ]
  }
}
```

### Execute MCP Tool
```http
POST /api/mcp/tools/{tool_name}
```

---

## System API

### Health Check
```http
GET /api/health
```

**Response:**
```json
{
  "success": true,
  "data": {
    "status": "healthy",
    "version": "1.0.0",
    "uptime": 86400,
    "database": "connected",
    "ai_providers": {
      "openrouter": "available",
      "openai": "unavailable"
    }
  }
}
```

### Dashboard Statistics
```http
GET /api/dashboard/stats
```

**Response:**
```json
{
  "success": true,
  "data": {
    "total_projects": 25,
    "total_analyses": 150,
    "total_files": 80,
    "analyses_this_week": 12,
    "average_processing_time": 120.5,
    "critical_errors_this_week": 5,
    "active_streaming_sources": 3,
    "knowledge_entries": 50
  }
}
```

---

## WebSocket API

### Connection
```
ws://localhost:3000/ws
```

### Message Format

**Subscribe to Analysis Updates:**
```json
{
  "type": "subscribe",
  "channel": "analysis",
  "analysis_id": "uuid"
}
```

**Analysis Progress Update:**
```json
{
  "type": "analysis_progress",
  "data": {
    "analysis_id": "uuid",
    "stage": "parsing",
    "progress": 45,
    "message": "Processing log entries..."
  }
}
```

**Analysis Complete:**
```json
{
  "type": "analysis_complete",
  "data": {
    "analysis_id": "uuid",
    "status": "completed",
    "results": {...}
  }
}
```

---

## Error Codes

| Status Code | Description |
|-------------|-------------|
| 200 | Success |
| 400 | Bad Request |
| 401 | Unauthorized |
| 403 | Forbidden |
| 404 | Not Found |
| 409 | Conflict |
| 422 | Validation Error |
| 429 | Rate Limited |
| 500 | Internal Server Error |
| 503 | Service Unavailable |

## Error Response Format
```json
{
  "success": false,
  "error": {
    "code": "VALIDATION_ERROR",
    "message": "Invalid request parameters",
    "details": {
      "field": "max_lines",
      "message": "Must be between 100 and 10000"
    }
  },
  "timestamp": "2024-01-01T00:00:00Z"
}
```

## Rate Limiting

- **Standard endpoints**: 100 requests per minute
- **Upload endpoints**: 10 requests per minute
- **Streaming endpoints**: 1000 requests per minute
- **WebSocket connections**: 100 connections per IP

## Pagination

List endpoints support pagination with the following parameters:
- `limit`: Number of items per page (max: 100)
- `offset`: Number of items to skip
- `page`: Page number (alternative to offset)

Response includes pagination metadata:
```json
{
  "data": [...],
  "pagination": {
    "total": 250,
    "page": 1,
    "limit": 10,
    "pages": 25
  }
}
```

---

This API documentation provides comprehensive coverage of all Synapse endpoints and functionality, enabling developers to integrate and interact with the platform effectively.