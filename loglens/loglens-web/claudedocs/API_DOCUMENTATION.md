# LogLens Web API Documentation

Complete API reference for LogLens Web Server v0.1.0

## Base URL

```
http://localhost:3000/api
```

## Table of Contents

1. [Projects](#projects)
2. [Log Files](#log-files)
3. [Analysis](#analysis)
4. [Knowledge Base](#knowledge-base)
5. [Streaming](#streaming)
6. [Metrics](#metrics)
7. [Settings](#settings)
8. [Export](#export)
9. [MCP Integration](#mcp-integration)

---

## Projects

### List All Projects

```http
GET /api/projects
```

**Response**
```json
[
  {
    "id": "uuid",
    "name": "string",
    "description": "string",
    "created_at": "ISO8601",
    "updated_at": "ISO8601"
  }
]
```

### Create Project

```http
POST /api/projects
Content-Type: application/json

{
  "name": "string",
  "description": "string"
}
```

**Response** (201 Created)
```json
{
  "id": "uuid",
  "name": "string",
  "description": "string",
  "created_at": "ISO8601",
  "updated_at": "ISO8601"
}
```

### Get Project

```http
GET /api/projects/:id
```

**Response**
```json
{
  "id": "uuid",
  "name": "string",
  "description": "string",
  "created_at": "ISO8601",
  "updated_at": "ISO8601"
}
```

### Delete Project

```http
DELETE /api/projects/:id
```

**Response** (204 No Content)

---

## Log Files

### List Log Files

```http
GET /api/projects/:id/files
```

**Response**
```json
[
  {
    "id": "uuid",
    "project_id": "uuid",
    "filename": "string",
    "file_size": "number",
    "upload_path": "string",
    "uploaded_at": "ISO8601"
  }
]
```

### Upload Log File

```http
POST /api/projects/:id/files
Content-Type: multipart/form-data

file: <binary>
```

**Response** (201 Created)
```json
{
  "id": "uuid",
  "project_id": "uuid",
  "filename": "string",
  "file_size": "number",
  "upload_path": "string",
  "uploaded_at": "ISO8601"
}
```

### Delete Log File

```http
DELETE /api/projects/:project_id/files/:file_id
```

**Response** (204 No Content)

---

## Analysis

### Start Analysis

```http
POST /api/projects/:project_id/files/:file_id/analyze
Content-Type: application/json

{
  "level_filter": "ERROR|WARN|INFO|DEBUG",
  "provider": "openrouter|openai|claude|gemini",
  "api_key": "string" // optional, uses settings if not provided
}
```

**Response** (201 Created)
```json
{
  "id": "uuid",
  "project_id": "uuid",
  "log_file_id": "uuid",
  "analysis_type": "string",
  "provider": "string",
  "level_filter": "string",
  "status": "pending|running|completed|failed",
  "started_at": "ISO8601"
}
```

### Get Analysis

```http
GET /api/analyses/:id
```

**Response**
```json
{
  "id": "uuid",
  "project_id": "uuid",
  "log_file_id": "uuid",
  "analysis_type": "string",
  "provider": "string",
  "level_filter": "string",
  "status": "pending|running|completed|failed",
  "result": "JSON object",
  "error_message": "string",
  "started_at": "ISO8601",
  "completed_at": "ISO8601"
}
```

### List Analyses

```http
GET /api/projects/:id/analyses?status=completed&limit=50
```

**Query Parameters:**
- `status` (optional): Filter by status
- `limit` (optional): Max results (default: 50, max: 100)
- `offset` (optional): Pagination offset (default: 0)

**Response**
```json
[
  {
    "id": "uuid",
    "project_id": "uuid",
    "log_file_id": "uuid",
    "analysis_type": "string",
    "provider": "string",
    "status": "string",
    "started_at": "ISO8601",
    "completed_at": "ISO8601"
  }
]
```

---

## Knowledge Base

### Create Knowledge Entry

```http
POST /api/projects/:id/knowledge
Content-Type: application/json

{
  "title": "string",
  "problem_description": "string",
  "solution": "string",
  "tags": ["string"],
  "severity": "low|medium|high|critical",
  "is_public": boolean
}
```

**Response** (201 Created)
```json
{
  "id": "uuid",
  "project_id": "uuid",
  "title": "string",
  "problem_description": "string",
  "solution": "string",
  "tags": ["string"],
  "severity": "string",
  "is_public": boolean,
  "usage_count": 0,
  "created_at": "ISO8601",
  "updated_at": "ISO8601"
}
```

### Get Knowledge Entries

```http
GET /api/projects/:id/knowledge?search=error&limit=50
```

**Query Parameters:**
- `search` (optional): Search in title, description, solution
- `category` (optional): Filter by category
- `severity` (optional): Filter by severity
- `limit` (optional): Max results (default: 50, max: 100)
- `offset` (optional): Pagination offset

**Response**
```json
[
  {
    "id": "uuid",
    "project_id": "uuid",
    "title": "string",
    "problem_description": "string",
    "solution": "string",
    "tags": ["string"],
    "severity": "string",
    "usage_count": "number",
    "created_at": "ISO8601"
  }
]
```

### Get Single Knowledge Entry

```http
GET /api/projects/:id/knowledge/:entry_id
```

**Response**
```json
{
  "id": "uuid",
  "project_id": "uuid",
  "title": "string",
  "problem_description": "string",
  "solution": "string",
  "tags": ["string"],
  "severity": "string",
  "is_public": boolean,
  "usage_count": "number",
  "created_at": "ISO8601",
  "updated_at": "ISO8601"
}
```

---

## Streaming

### Create Streaming Source

```http
POST /api/projects/:project_id/streaming/sources
Content-Type: application/json

{
  "name": "string",
  "source_type": "file|http|websocket",
  "config": {},
  "parser_config": {
    "log_format": "json|text|regex",
    "timestamp_format": "string",
    "level_field": "string",
    "message_field": "string"
  },
  "buffer_size": 1000,
  "batch_timeout_seconds": 5
}
```

**Response** (201 Created)
```json
{
  "source_id": "uuid",
  "name": "string",
  "source_type": "string",
  "project_id": "uuid",
  "status": "active",
  "created_at": "ISO8601"
}
```

### Ingest Logs (HTTP)

```http
POST /api/projects/:project_id/streaming/ingest
Content-Type: application/json

[
  {
    "message": "string",
    "level": "ERROR|WARN|INFO|DEBUG",
    "timestamp": "ISO8601"
  }
]
```

**Response** (202 Accepted)

### Get Streaming Stats

```http
GET /api/projects/:project_id/streaming/stats
```

**Response**
```json
{
  "active_sources": "number",
  "active_connections": "number",
  "total_logs_processed": "number",
  "sources": []
}
```

### WebSocket Real-Time Stream

```
ws://localhost:3000/api/projects/:project_id/stream
```

**Sent Messages:**
```json
{
  "type": "subscribe",
  "filters": {
    "level": "ERROR",
    "source": "app"
  }
}
```

**Received Messages:**
```json
{
  "id": "uuid",
  "timestamp": "ISO8601",
  "level": "ERROR",
  "message": "string",
  "source": "string",
  "project_id": "uuid"
}
```

---

## Metrics

### Get Performance Metrics

```http
GET /api/metrics
```

**Response**
```json
{
  "request_count": "number",
  "error_count": "number",
  "error_rate": "number",
  "avg_response_time": "number (milliseconds)",
  "uptime": "number (seconds)",
  "quality_metrics": {
    "analysis_accuracy": "number",
    "analysis_completion_rate": "number",
    "average_confidence_score": "number",
    "system_availability": "number",
    "cache_hit_rate": "number"
  },
  "endpoint_metrics": [
    {
      "path": "string",
      "request_count": "number",
      "error_count": "number",
      "avg_duration": "number (milliseconds)"
    }
  ]
}
```

### Get Health with Metrics

```http
GET /api/health/metrics
```

**Response**
```json
{
  "status": "healthy|degraded",
  "uptime_seconds": "number",
  "request_count": "number",
  "error_rate": "number",
  "avg_response_time_ms": "number",
  "quality_score": "number (0-1)"
}
```

---

## Settings

### Get Settings

```http
GET /api/settings
```

**Response**
```json
{
  "id": "uuid",
  "api_key": "string",
  "default_provider": "string",
  "default_level": "string",
  "created_at": "ISO8601",
  "updated_at": "ISO8601"
}
```

### Update Settings

```http
PATCH /api/settings
Content-Type: application/json

{
  "api_key": "string",
  "default_provider": "openrouter|openai|claude|gemini",
  "default_level": "ERROR|WARN|INFO|DEBUG"
}
```

**Response**
```json
{
  "id": "uuid",
  "api_key": "string",
  "default_provider": "string",
  "default_level": "string",
  "updated_at": "ISO8601"
}
```

---

## Export

### Export HTML Report

```http
GET /api/projects/:id/analyses/:analysis_id/export/html
```

**Response** (200 OK)
```
Content-Type: text/html
Content-Disposition: attachment; filename="analysis_report.html"

<html>...</html>
```

### Export JSON Data

```http
GET /api/projects/:id/analyses/:analysis_id/export/json
```

**Response** (200 OK)
```json
{
  "analysis": {},
  "metadata": {},
  "generated_at": "ISO8601"
}
```

### Export CSV Data

```http
GET /api/projects/:id/analyses/:analysis_id/export/csv
```

**Response** (200 OK)
```
Content-Type: text/csv
Content-Disposition: attachment; filename="analysis_data.csv"
```

### Export PDF Report

```http
GET /api/projects/:id/analyses/:analysis_id/export/pdf
```

**Response** (200 OK)
```
Content-Type: application/pdf
Content-Disposition: attachment; filename="analysis_report.pdf"
```

### Export Markdown Report

```http
GET /api/projects/:id/analyses/:analysis_id/export/md
```

**Response** (200 OK)
```
Content-Type: text/markdown
Content-Disposition: attachment; filename="analysis_report.md"
```

### Create Share Link

```http
POST /api/projects/:id/share
Content-Type: application/json

{
  "analysis_id": "uuid",
  "expires_in_hours": 24,
  "password": "string",
  "allow_download": boolean
}
```

**Response** (201 Created)
```json
{
  "share_id": "uuid",
  "url": "string",
  "expires_at": "ISO8601"
}
```

---

## MCP Integration

### Handle MCP Request

```http
POST /api/projects/:id/mcp
Content-Type: application/json

{
  "request": "MCP request JSON"
}
```

**Response**
```json
{
  "response": "MCP response JSON"
}
```

### Generate MCP Ticket

```http
POST /api/projects/:id/mcp/tickets
Content-Type: application/json

{
  "analysis_id": "uuid",
  "error_summary": "string",
  "affected_lines": "string",
  "root_cause": "string",
  "context_payload": "string"
}
```

**Response** (201 Created)
```json
{
  "ticket_id": "string",
  "error_summary": "string",
  "deep_link": "string",
  "created_at": "ISO8601"
}
```

### Get MCP Context

```http
GET /api/projects/:id/mcp/context/:analysis_id?detail_level=standard
```

**Query Parameters:**
- `detail_level`: minimal|standard|full (default: standard)
- `include_correlations`: boolean (default: false)
- `include_metrics`: boolean (default: false)

**Response**
```json
{
  "ticket_id": "string",
  "context_payload": "string",
  "detail_level": "string",
  "size_bytes": "number",
  "created_at": "ISO8601"
}
```

---

## Error Responses

All endpoints return consistent error responses:

```json
{
  "error": "error_type",
  "message": "Human-readable error message",
  "code": "ERROR_CODE",
  "timestamp": "ISO8601"
}
```

### Common Error Codes

- `400 Bad Request` - Invalid request parameters
- `401 Unauthorized` - Missing or invalid authentication
- `404 Not Found` - Resource not found
- `422 Unprocessable Entity` - Validation error
- `429 Too Many Requests` - Rate limit exceeded
- `500 Internal Server Error` - Server error
- `502 Bad Gateway` - AI provider error
- `503 Service Unavailable` - Service temporarily unavailable

---

## Rate Limiting

- Global rate limit: 1000 requests/minute
- Per-IP rate limit: 100 requests/minute
- Streaming connections: 10 concurrent per project

Rate limit headers:
```
X-RateLimit-Limit: 100
X-RateLimit-Remaining: 95
X-RateLimit-Reset: 1234567890
```

---

## Pagination

List endpoints support pagination:

**Request:**
```
GET /api/projects/:id/analyses?limit=50&offset=100
```

**Response Headers:**
```
X-Total-Count: 523
X-Page-Count: 11
Link: <url>; rel="next", <url>; rel="last"
```

---

## WebSocket Protocol

### Connection

```javascript
const ws = new WebSocket('ws://localhost:3000/api/projects/PROJECT_ID/stream');
```

### Subscribe to Logs

```json
{
  "type": "subscribe",
  "filters": {
    "level": "ERROR",
    "source": "app",
    "since": "ISO8601"
  }
}
```

### Unsubscribe

```json
{
  "type": "unsubscribe"
}
```

### Ping/Pong

Server sends ping every 30s. Client should respond with pong:

```json
{"type": "pong"}
```

---

## Authentication

Currently, LogLens uses API key authentication for AI provider access. Future versions will include:

- JWT token authentication
- OAuth2 integration
- Role-based access control (RBAC)

---

## Versioning

API version is included in the base URL:

```
/api/v1/projects
```

Current version: v1 (implicit, no version in URL yet)

---

## Support

For issues and feature requests:
- GitHub: https://github.com/yourusername/loglens
- Documentation: https://docs.loglens.dev

Last Updated: 2025-10-04
