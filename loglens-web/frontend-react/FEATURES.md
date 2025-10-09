# LogLens Frontend Features

## Overview

The LogLens frontend is a modern React application with TypeScript, providing a rich user interface for log analysis, error pattern detection, knowledge management, and real-time streaming capabilities.

## New Features (Phase 4)

### Pattern Filtering by Category and Severity

**Location**: `/projects/:projectId/patterns`

**Features**:
- Filter error patterns by category (code, infrastructure, configuration, external)
- Filter by severity level (critical, high, medium, low)
- Search patterns by text
- Visual severity indicators with color coding
- Frequency display for each pattern

**Usage**:
```typescript
// Navigate to patterns page for a project
navigate(`/projects/${projectId}/patterns`);

// API call with filters
const params = new URLSearchParams();
params.append('category', 'code');
params.append('severity', 'critical');

fetch(`/api/projects/${projectId}/patterns?${params}`);
```

### Public Knowledge Base

**Location**: `/knowledge/public`

**Features**:
- Browse community-shared solutions
- Search knowledge entries
- Filter by severity
- Expandable solution details
- Usage statistics tracking

**Usage**:
```typescript
// Navigate to public knowledge
navigate('/knowledge/public');

// Search public knowledge
fetch(`/api/knowledge/public?search=${searchTerm}`);
```

### Knowledge Entry Creation

**Component**: `CreateKnowledgeEntry`

**Features**:
- Create new knowledge entries
- Public sharing toggle
- Severity classification
- Tag support
- Rich text solution input

**Usage**:
```tsx
import CreateKnowledgeEntry from '@components/CreateKnowledgeEntry';

<CreateKnowledgeEntry
  projectId={projectId}
  onSuccess={() => {
    // Refresh knowledge list
    fetchKnowledge();
  }}
  onCancel={() => {
    // Close modal
    setShowModal(false);
  }}
/>
```

### Streaming Dashboard

**Location**: `/projects/:projectId/streaming`

**Features**:
- Real-time log streaming from multiple sources
- Create streaming sources (file, command, TCP, HTTP)
- Monitor active sources and connections
- Live statistics dashboard
- Stop/start streaming sources
- Logs processed counter

**Source Types**:
1. **File**: Tail log files in real-time (`tail -f`)
2. **Command**: Stream output from system commands
3. **TCP Listener**: Accept logs via TCP connections
4. **HTTP Endpoint**: Receive logs via HTTP POST

**Usage**:
```typescript
// Navigate to streaming page
navigate(`/projects/${projectId}/streaming`);

// Create a file streaming source
const sourceConfig = {
  project_id: projectId,
  name: 'Application Logs',
  source_type: 'file',
  config: {
    path: '/var/log/app.log'
  },
  parser_config: {
    log_format: 'text'
  },
  buffer_size: 100,
  batch_timeout_seconds: 2,
  restart_on_error: true,
};

await fetch(`/api/projects/${projectId}/streaming/sources`, {
  method: 'POST',
  headers: { 'Content-Type': 'application/json' },
  body: JSON.stringify(sourceConfig),
});
```

## Component Architecture

### Pages

#### PatternsPage (`src/pages/PatternsPage.tsx`)
- Error pattern listing with advanced filtering
- Category and severity filters
- Real-time search
- Color-coded severity badges
- Example lines display

#### PublicKnowledgePage (`src/pages/PublicKnowledgePage.tsx`)
- Public knowledge base browser
- Search functionality
- Expandable solutions
- Usage statistics
- Tag display

#### StreamingPage (`src/pages/StreamingPage.tsx`)
- Streaming source management
- Real-time statistics
- Source creation modal
- Active source monitoring
- Source lifecycle management

### Components

#### CreateKnowledgeEntry (`src/components/CreateKnowledgeEntry.tsx`)
- Modal-based knowledge entry form
- Public sharing toggle
- Severity selection
- Tags management
- Form validation

### Hooks

All existing hooks are preserved:
- `useWebSocketAnalysis`: WebSocket-based analysis with progress tracking
- `useTheme`: Theme management (light/dark)
- `useWebSocket`: Generic WebSocket connection
- `useSettings`: Application settings management
- `useAnalysisDetail`: Analysis detail fetching and caching

## Routing

New routes added to `App.tsx`:

```typescript
<Route path="/projects/:projectId/patterns" element={<PatternsPage />} />
<Route path="/projects/:projectId/streaming" element={<StreamingPage />} />
<Route path="/knowledge/public" element={<PublicKnowledgePage />} />
```

## API Integration

### Pattern API
```typescript
// Get patterns with filters
GET /api/projects/:id/patterns?category=code&severity=critical

// Response
[
  {
    id: string,
    project_id: string,
    pattern: string,
    description: string,
    category: 'code' | 'infrastructure' | 'configuration' | 'external',
    severity: 'low' | 'medium' | 'high' | 'critical',
    frequency: number,
    example_lines: string,
    created_at: string,
    updated_at: string
  }
]
```

### Knowledge API
```typescript
// Create knowledge entry
POST /api/projects/:id/knowledge
{
  title: string,
  problem_description: string,
  solution: string,
  tags?: string,
  severity: 'low' | 'medium' | 'high' | 'critical',
  is_public: boolean
}

// Get public knowledge
GET /api/knowledge/public?search=authentication

// Response
[
  {
    id: string,
    project_id: string,
    title: string,
    problem_description: string,
    solution: string,
    tags: string,
    severity: string,
    usage_count: number,
    is_public: boolean,
    created_at: string,
    updated_at: string
  }
]
```

### Streaming API
```typescript
// Create streaming source
POST /api/projects/:id/streaming/sources
{
  project_id: string,
  name: string,
  source_type: 'file' | 'command' | 'tcp' | 'http',
  config: {
    path?: string,      // for file
    command?: string,   // for command
    args?: string[],    // for command
    port?: number,      // for tcp
  },
  parser_config: {
    log_format: 'text' | 'json' | 'syslog'
  },
  buffer_size: number,
  batch_timeout_seconds: number,
  restart_on_error: boolean
}

// Get streaming stats
GET /api/projects/:id/streaming/stats

// Response
{
  active_sources: number,
  active_connections: number,
  total_logs_processed: number
}
```

## Styling

All new components follow the existing Tailwind CSS design system:

- **Color Scheme**: Consistent with existing dark/light theme support
- **Severity Colors**:
  - Critical: Red (bg-red-100, text-red-800, border-red-300)
  - High: Orange (bg-orange-100, text-orange-800, border-orange-300)
  - Medium: Yellow (bg-yellow-100, text-yellow-800, border-yellow-300)
  - Low: Blue (bg-blue-100, text-blue-800, border-blue-300)
- **Category Colors**:
  - Code: Purple (bg-purple-100, text-purple-800)
  - Infrastructure: Green (bg-green-100, text-green-800)
  - Configuration: Indigo (bg-indigo-100, text-indigo-800)
  - External: Pink (bg-pink-100, text-pink-800)

## Testing

Integration tests should cover:

1. **Pattern Filtering**
   - Category filter functionality
   - Severity filter functionality
   - Search functionality
   - Combined filters

2. **Knowledge Base**
   - Public knowledge listing
   - Knowledge entry creation
   - Public toggle functionality
   - Search functionality

3. **Streaming**
   - Source creation for all types
   - Source listing
   - Source stopping
   - Statistics retrieval

## Accessibility

All new components follow WCAG 2.1 Level AA standards:

- Semantic HTML elements
- ARIA labels on interactive elements
- Keyboard navigation support
- Focus management
- Color contrast compliance
- Screen reader support

## Performance

Optimizations implemented:

- **Lazy Loading**: All pages are lazy-loaded via React.lazy()
- **Efficient Filtering**: Client-side filtering for improved responsiveness
- **Debounced Search**: Search inputs use debouncing to reduce API calls
- **Optimistic Updates**: UI updates before API confirmation where appropriate
- **Caching**: React Query for data caching and invalidation

## Future Enhancements

Potential improvements for consideration:

1. Real-time streaming log viewer with WebSocket updates
2. Advanced analytics dashboard for streaming sources
3. Bulk operations for pattern management
4. Export functionality for knowledge base entries
5. Pattern recommendation engine
6. Streaming source templates
7. Advanced filtering UI with saved filters
8. Collaborative knowledge base features
