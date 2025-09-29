# LogLens Web Interface Implementation Plan
## AI Agent-Oriented Development Roadmap

### **Project Overview**
Transform LogLens from CLI-only to full-stack Rust web application with intelligent error analysis, MCP integration, and developer-focused UI. Focus on root cause identification with smart error discrimination and context management.

### **Architecture Summary**
- **Full-Stack Rust**: Axum backend + WASM frontend
- **Hybrid Processing**: Client-side parsing/filtering + server-side AI analysis
- **Smart Context Management**: Progressive error classification and relevance scoring
- **MCP Integration**: Error ticket referencing with three-tier data granularity
- **Performance First**: Developer-grade responsiveness with clean, original design

---

## **PHASE 1: Foundation & Backend Enhancement**

### **Task 1.1: Project Structure Reorganization**
**Agent Focus**: Create proper workspace structure for web application

**Deliverables**:
- Convert to Cargo workspace with multiple crates
- `loglens-core`: Shared analysis logic
- `loglens-cli`: Existing CLI interface
- `loglens-web`: New web application
- `loglens-wasm`: Client-side parsing components

**Implementation Notes**:
```toml
[workspace]
members = ["loglens-core", "loglens-cli", "loglens-web", "loglens-wasm"]
```

### **Task 1.2: Enhanced Error Classification System**
**Agent Focus**: Implement smart error categorization for AI discrimination

**Deliverables**:
- `ErrorCategory` enum with detailed classification
- `RelevanceScorer` for context prioritization
- Pattern matching for infrastructure vs code errors
- Database schema error detection patterns

**Key Components**:
```rust
pub enum ErrorCategory {
    CodeRelated { file: Option<String>, function: Option<String>, line: Option<u32> },
    InfrastructureRelated { component: String, severity: Severity },
    ConfigurationRelated { config_file: Option<String>, missing_setting: Option<String> },
    ExternalServiceRelated { service: String, endpoint: Option<String> },
    UnknownRelated,
}
```

### **Task 1.3: Context Management System**
**Agent Focus**: Implement progressive context management for large logs

**Deliverables**:
- `ContextManager` with token-aware filtering
- Relevance scoring algorithm
- Fallback summarization for oversized contexts
- User context integration for focused analysis

**Key Features**:
- Max token limits with smart truncation
- Priority-based error selection
- User-provided context weighting
- Progressive disclosure of details

### **Task 1.4: AI Provider Enhancement**
**Agent Focus**: Enhance existing AI integration with discrimination capabilities

**Deliverables**:
- Enhanced system prompts for error discrimination
- Context-aware analysis requests
- Error classification in AI responses
- Structured output parsing for categorized results

**System Prompt Strategy**:
```
SYSTEM: You are analyzing filtered logs to identify root causes. Follow these rules:
1. DISTINGUISH ERROR TYPES: Code vs Infrastructure vs Configuration vs External
2. FOCUS ON USER CONTEXT: "{user_provided_context}"
3. CONTEXT SIZE MANAGEMENT: Summarize if exceeding limits
4. OUTPUT STRUCTURE: Category, description, file/line identification, recommendations
```

---

## **PHASE 2: Web Backend Development**

### **Task 2.1: Axum Web Server Setup**
**Agent Focus**: Create performant web server with proper routing

**Deliverables**:
- Axum server with middleware stack
- Error handling and logging
- CORS configuration for development
- Health check endpoints

**Routes Structure**:
```
POST /api/analyze - Log analysis endpoint
GET  /api/projects - List available projects
POST /api/projects - Create new project
GET  /api/projects/:id/analyses - Historical analyses
WebSocket /ws/analysis - Real-time analysis updates
```

### **Task 2.2: Project Management System**
**Agent Focus**: Implement per-project log analysis with persistence

**Deliverables**:
- Project creation and management API
- SQLite database for project metadata
- Analysis history storage
- Knowledge base accumulation per project

**Database Schema**:
```sql
CREATE TABLE projects (id, name, created_at, updated_at);
CREATE TABLE analyses (id, project_id, log_file, user_context, results, created_at);
CREATE TABLE error_patterns (id, project_id, pattern, category, frequency);
```

### **Task 2.3: File Upload & Processing Pipeline**
**Agent Focus**: Handle large log file uploads with streaming processing

**Deliverables**:
- Multipart file upload handling
- Streaming log file processing
- Progress tracking for large files
- Temporary file management

**Processing Pipeline**:
1. Stream upload to temporary storage
2. Initial client-side parsing preview
3. Server-side full analysis
4. Result caching and storage

### **Task 2.4: Real-time Analysis API**
**Agent Focus**: WebSocket-based real-time analysis with progress updates

**Deliverables**:
- WebSocket connection management
- Progress streaming for long analyses
- Cancellation support
- Error handling and reconnection

---

## **PHASE 3: WASM Frontend Development**

### **Task 3.1: WASM Log Parser**
**Agent Focus**: Client-side log parsing and filtering in Rust/WASM

**Deliverables**:
- WASM module for log parsing
- JavaScript bindings for web integration
- Real-time filtering and search
- Memory-efficient processing for large files

**WASM Interface**:
```rust
#[wasm_bindgen]
pub struct LogParser {
    entries: Vec<LogEntry>,
    filters: FilterConfig,
}

#[wasm_bindgen]
impl LogParser {
    pub fn parse_chunk(&mut self, chunk: &str) -> js_sys::Array;
    pub fn filter_by_level(&self, level: &str) -> js_sys::Array;
    pub fn search(&self, query: &str) -> js_sys::Array;
}
```

### **Task 3.2: Web UI Foundation**
**Agent Focus**: Create performant, developer-focused web interface

**Deliverables**:
- Modern, clean design system (no AI-slop aesthetics)
- Dark/light theme with code-editor feel
- Responsive layout for desktop focus
- Keyboard shortcuts for navigation

**Design Principles**:
- Performance over flash
- Developer-friendly monospace fonts
- Color-coded severity levels
- Clean, original aesthetic

### **Task 3.3: Log Visualization Components**
**Agent Focus**: Interactive log viewing with dual-pane layout

**Deliverables**:
- Virtualized log list for performance
- Syntax-highlighted log entries
- Expandable context views
- Line-by-line navigation

**Component Structure**:
```
LogViewer {
  LogList (virtualized),
  LogEntry (syntax highlighted),
  ContextPanel (expandable),
  SearchBar (real-time filtering)
}
```

### **Task 3.4: Error Visualization Dashboard**
**Agent Focus**: Create error analysis visualization with graphs and code views

**Deliverables**:
- Error summary dashboard
- Interactive dependency graphs
- Code-style error highlighting
- Drill-down navigation

**Visualization Types**:
- Error frequency charts
- Timeline of error occurrences
- Dependency flow diagrams
- Stack trace visualization

---

## **PHASE 4: Advanced Features**

### **Task 4.1: Knowledge Base System**
**Agent Focus**: Build project-specific error knowledge accumulation

**Deliverables**:
- Pattern recognition for recurring errors
- Historical comparison capabilities
- Common issue templates
- Suggested solutions based on history

### **Task 4.2: Enhanced MCP Integration**
**Agent Focus**: Implement three-tier MCP referencing system

**Deliverables**:
- Error ticket generation with structured references
- Minimal context payloads for MCP
- Deep-link integration to web interface
- Progressive detail disclosure

**MCP Data Structure**:
```json
{
  "ticket_id": "AUTH-2024-001",
  "error_summary": "Authentication failure in UserService.validateToken()",
  "affected_lines": "lines 1205-1210",
  "root_cause": "JWT expiry not handled gracefully",
  "context_payload": "minimal_error_context",
  "deep_link": "http://localhost:3000/analysis/abc123#lines-1205-1210"
}
```

### **Task 4.3: Advanced Analysis Features**
**Agent Focus**: Implement sophisticated error correlation and root cause analysis

**Deliverables**:
- Cross-error correlation analysis
- Performance bottleneck identification
- Anomaly detection with confidence scoring
- Multi-log file analysis

### **Task 4.4: Export and Reporting**
**Agent Focus**: Generate comprehensive reports in multiple formats

**Deliverables**:
- HTML report generation
- PDF export capability
- JSON/CSV data export
- Shareable analysis links

---

## **PHASE 5: Performance & Polish**

### **Task 5.1: Performance Optimization**
**Agent Focus**: Ensure developer-grade responsiveness

**Deliverables**:
- WASM optimization for large logs
- Server-side caching strategies
- Database query optimization
- Memory usage optimization

**Performance Targets**:
- < 100ms response time for UI interactions
- Support for 50k+ line log files
- < 2s initial load time
- Smooth scrolling in virtualized lists

### **Task 5.2: Error Handling & UX Polish**
**Agent Focus**: Robust error handling and user experience refinement

**Deliverables**:
- Comprehensive error handling
- Loading states and progress indicators
- Graceful degradation for large files
- User feedback and validation

### **Task 5.3: Testing & Documentation**
**Agent Focus**: Comprehensive testing strategy

**Deliverables**:
- Unit tests for all core components
- Integration tests for API endpoints
- WASM module testing
- End-to-end user workflow testing

---

## **Implementation Strategy for AI Agents**

### **Agent Coordination Approach**
1. **Backend-First Development**: Establish solid API foundation before frontend
2. **Incremental Integration**: Test each component independently before integration
3. **Performance Validation**: Benchmark after each major component
4. **User Feedback Loops**: Regular validation against user requirements

### **Testing Strategy**
- Mock large log files for performance testing
- Validate AI discrimination with real-world error scenarios
- Test MCP integration with actual error ticket workflows
- Verify context management with oversized logs

### **Key Quality Gates**
- All new code must pass existing tests
- Performance regressions caught by benchmarks
- UI responsiveness validated on target hardware
- MCP integration tested with real AI conversations

### **Risk Mitigation**
- **Large Log Handling**: Progressive loading and summarization fallbacks
- **AI Context Overflow**: Smart truncation with user notification
- **Performance Bottlenecks**: Profiling at each phase
- **Error Discrimination**: Extensive testing with diverse log types

---

## **Success Criteria**

### **Technical Success**
- Handle 15k+ line logs without performance degradation
- Accurate error classification (code vs infrastructure vs config)
- Sub-second UI responsiveness for common operations
- Effective MCP integration with minimal context bloat

### **User Experience Success**
- Developers can quickly identify root causes
- Clean, original design that doesn't feel like "AI-slop"
- Keyboard shortcuts and power-user features
- Effective visual correlation between errors and code

### **Integration Success**
- Seamless MCP error ticket referencing
- Cross-session project knowledge accumulation
- Effective distinction between related and unrelated errors
- Actionable recommendations for identified issues

---

## **Final Notes**

This implementation plan prioritizes performance, developer experience, and accurate error analysis over flashy features. Each phase builds upon the previous one, ensuring a solid foundation for advanced features. The focus on AI agent development means clear separation of concerns, comprehensive testing, and robust error handling throughout.

The plan addresses the core concern of AI context management through progressive filtering and smart relevance scoring, while maintaining the ability to handle both code-related and infrastructure-related errors effectively.