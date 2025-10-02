// Core domain types
export interface Project {
  id: string;
  name: string;
  description?: string;
  created_at: string;
  updated_at: string;
  analysis_count?: number;
}

export interface CreateProjectRequest {
  name: string;
  description?: string;
}

export interface LogFile {
  id: string;
  project_id: string;
  filename: string;
  file_size: number;      // Changed from 'size' to match backend
  created_at: string;     // Changed from 'upload_time' to match backend
  line_count?: number;
}

export interface LogEntry {
  line_number: number;
  timestamp?: string;
  level?: LogLevel;
  message: string;
  raw_line: string;
  is_truncated?: boolean;
}

export type LogLevel = 'ERROR' | 'WARN' | 'INFO' | 'DEBUG';

export interface Analysis {
  id: string;
  project_id: string;
  file_id: string;
  status: AnalysisStatus;
  user_context?: string;
  ai_provider: string;
  created_at: string;
  updated_at: string;
  result?: AnalysisResult;
  progress?: number;
  error?: string;
}

export type AnalysisStatus = 'pending' | 'running' | 'completed' | 'failed' | 'cancelled';

export interface AnalysisRequest {
  user_context?: string;
  provider: string;
  level: string;
  max_lines?: number;
  selected_model?: string;
  timeout_seconds?: number;
}

export interface AnalysisResult {
  summary: string;
  sequence_of_events?: string;
  root_cause?: {
    description?: string;
    confidence?: number;
  };
  errors_found: ErrorAnalysis[];
  patterns: PatternAnalysis[];
  performance: PerformanceAnalysis;
  anomalies: AnomalyAnalysis[];
  correlations: CorrelationAnalysis[];
  recommendations: string[];
  confidence_score: number;
  related_errors?: string[];
  unrelated_errors?: string[];
}

export interface ErrorAnalysis {
  category: ErrorCategory;
  description: string;
  file_location?: string;
  line_numbers: number[];
  frequency: number;
  severity: 'low' | 'medium' | 'high' | 'critical';
  context: string[];
  recommendations: string[];
}

export interface PatternAnalysis {
  pattern: string;
  frequency: number;
  first_occurrence: number;
  last_occurrence: number;
  trend: 'increasing' | 'decreasing' | 'stable';
}

export interface PerformanceAnalysis {
  total_processing_time: number;
  bottlenecks: string[];
  recommendations: string[];
  metrics: Record<string, number>;
}

export interface AnomalyAnalysis {
  description: string;
  confidence: number;
  line_numbers: number[];
  anomaly_type: string;
}

export interface CorrelationAnalysis {
  related_errors: string[];
  root_cause?: string;
  correlation_strength: number;
  affected_components: string[];
}

export type ErrorCategory =
  | 'code_related'
  | 'infrastructure_related'
  | 'configuration_related'
  | 'external_service_related'
  | 'unknown_related';

// WebSocket types
export interface WebSocketMessage {
  type: 'analysis_status' | 'system_status' | 'ping' | 'pong' | 'subscribe' | 'unsubscribe';
}

export interface AnalysisStatusUpdate extends WebSocketMessage {
  type: 'analysis_status';
  analysis_id: string;
  status: AnalysisStatus;
  progress?: number;
  message?: string;
  result?: AnalysisResult;
  error?: string;
}

export interface SystemStatusUpdate extends WebSocketMessage {
  type: 'system_status';
  online: boolean;
  message: string;
}

// WASM types
export interface LogParseResult {
  total_lines: number;
  error_lines: number;
  warning_lines: number;
  info_lines: number;
  debug_lines: number;
  lines_by_level: LogLinePreview[];
}

export interface LogLinePreview {
  line_number: number;
  level?: string;
  timestamp?: string;
  message: string;
  is_truncated: boolean;
}

// UI State types
export interface FilterState {
  level: LogLevel | 'ALL';
  search: string;
  dateRange?: {
    start: Date;
    end: Date;
  };
}

export interface PaginationState {
  page: number;
  per_page: number;
  total: number;
}

export interface SortState {
  field: string;
  direction: 'asc' | 'desc';
}

export interface ViewState {
  layout: 'split' | 'full';
  showTimestamps: boolean;
  showLineNumbers: boolean;
  wrapLines: boolean;
}

// Theme types
export type Theme = 'light' | 'dark' | 'system';

// API Response types
export interface ApiResponse<T> {
  data: T;
  success: boolean;
  error?: string;
}

export interface AnalysisListResponse {
  analyses: Analysis[];
  pagination: PaginationState;
}

// Performance monitoring types
export interface PerformanceMetrics {
  renderTime: number;
  wasmExecutionTime: number;
  apiResponseTime: number;
  memoryUsage: number;
}

// Error types
export interface AppError {
  code: string;
  message: string;
  details?: Record<string, any>;
  timestamp: Date;
}

// Settings types
export interface Settings {
  default_provider: string;
  api_key: string;
  max_lines: number;
  default_level: string;
  show_timestamps: boolean;
  show_line_numbers: boolean;
  selected_model?: string;
  available_models?: string; // JSON array cache
  models_last_fetched?: string; // ISO datetime
  analysis_timeout_seconds?: number;
}

// Model types
export interface ModelInfo {
  id: string;
  name: string;
  provider: string;
  description?: string;
  context_limit: number;
  supports_streaming: boolean;
  pricing?: {
    input_token_cost: number;
    output_token_cost: number;
    currency: string;
  };
}

export interface ModelListResponse {
  models: ModelInfo[];
  cached_at: string;
  expires_at: string;
}

// Dashboard types
export interface DashboardStats {
  total_projects: number;
  analyses_this_week: number;
  avg_processing_time_minutes?: number;
  critical_errors: number;
}

// Export utilities
export const LogLevels: LogLevel[] = ['ERROR', 'WARN', 'INFO', 'DEBUG'];
export const AnalysisStatuses: AnalysisStatus[] = ['pending', 'running', 'completed', 'failed', 'cancelled'];