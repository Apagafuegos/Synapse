use serde::{Deserialize, Serialize};
use sqlx::{Pool, Sqlite};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

use crate::cache::{CacheKeys, CacheManager};
use crate::models::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryPerformanceStats {
    pub query_type: String,
    pub execution_time_ms: u64,
    pub cache_hit: bool,
    pub rows_affected: u64,
    pub timestamp: String,
}

#[derive(Debug, Default)]
pub struct PerformanceMonitor {
    query_stats: Arc<RwLock<Vec<QueryPerformanceStats>>>,
    cache_manager: Arc<CacheManager>,
}

impl PerformanceMonitor {
    pub fn new(cache_manager: Arc<CacheManager>) -> Self {
        Self {
            query_stats: Arc::new(RwLock::new(Vec::new())),
            cache_manager,
        }
    }

    pub async fn record_query(
        &self,
        query_type: String,
        execution_time: Duration,
        cache_hit: bool,
        rows_affected: u64,
    ) {
        let stat = QueryPerformanceStats {
            query_type,
            execution_time_ms: execution_time.as_millis() as u64,
            cache_hit,
            rows_affected,
            timestamp: chrono::Utc::now().to_rfc3339(),
        };

        let mut stats = self.query_stats.write().await;
        stats.push(stat);

        // Keep only last 1000 stats to prevent memory growth
        let len = stats.len();
        if len > 1000 {
            // Use a different approach: create a new vector with the last 1000 elements
            // This avoids the borrow checker issue by using a temporary variable
            let temp_vec = std::mem::take(&mut *stats);
            let skip_count = len - 1000;
            let new_stats: Vec<QueryPerformanceStats> =
                temp_vec.into_iter().skip(skip_count).collect();
            *stats = new_stats;
        }
    }

    pub async fn get_performance_summary(&self) -> PerformanceSummary {
        let stats = self.query_stats.read().await;
        let cache_stats = self.cache_manager.get_cache_stats();

        let total_queries = stats.len() as u64;
        let cache_hits = stats.iter().filter(|s| s.cache_hit).count() as u64;
        let average_response_time = if !stats.is_empty() {
            stats.iter().map(|s| s.execution_time_ms).sum::<u64>() as f64 / stats.len() as f64
        } else {
            0.0
        };

        let slowest_queries = {
            let mut sorted_stats = stats.clone();
            sorted_stats.sort_by(|a, b| b.execution_time_ms.cmp(&a.execution_time_ms));
            sorted_stats.into_iter().take(10).collect()
        };

        PerformanceSummary {
            total_queries,
            cache_hit_rate: if total_queries > 0 {
                cache_hits as f64 / total_queries as f64
            } else {
                0.0
            },
            average_response_time_ms: average_response_time,
            slowest_queries,
            cache_stats,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PerformanceSummary {
    pub total_queries: u64,
    pub cache_hit_rate: f64,
    pub average_response_time_ms: f64,
    pub slowest_queries: Vec<QueryPerformanceStats>,
    pub cache_stats: HashMap<String, crate::cache::CacheStats>,
}

// Optimized database operations
pub struct OptimizedDbOps {
    pool: Pool<Sqlite>,
    cache: Arc<CacheManager>,
    monitor: Arc<PerformanceMonitor>,
}

impl OptimizedDbOps {
    pub fn new(pool: Pool<Sqlite>, cache: Arc<CacheManager>) -> Self {
        let monitor = Arc::new(PerformanceMonitor::new(Arc::clone(&cache)));
        Self {
            pool,
            cache,
            monitor,
        }
    }

    // Optimized analysis retrieval with caching
    pub async fn get_analysis_cached(
        &self,
        project_id: &str,
        analysis_id: &str,
    ) -> Result<Option<Analysis>, sqlx::Error> {
        let cache_key = CacheKeys::analysis_key(project_id, analysis_id);
        let start = Instant::now();

        // Try cache first
        if let Some(cached_analysis) = self.cache.analysis_cache.get(&cache_key) {
            self.monitor
                .record_query("get_analysis".to_string(), start.elapsed(), true, 1)
                .await;
            return Ok(Some(cached_analysis));
        }

        // Cache miss - query database
        let analysis = sqlx::query_as::<_, Analysis>(
            "SELECT id, project_id, log_file_id, analysis_type, provider, level_filter, status, result, error_message, started_at, completed_at
             FROM analyses WHERE id = ? AND project_id = ?"
        )
        .bind(analysis_id)
        .bind(project_id)
        .fetch_optional(&self.pool)
        .await?;

        let execution_time = start.elapsed();

        if let Some(ref analysis) = analysis {
            // Cache the result
            self.cache
                .analysis_cache
                .put(cache_key, analysis.clone(), None);
        }

        self.monitor
            .record_query(
                "get_analysis".to_string(),
                execution_time,
                false,
                if analysis.is_some() { 1 } else { 0 },
            )
            .await;

        Ok(analysis)
    }

    // Optimized project analyses with pagination and caching
    pub async fn get_project_analyses_cached(
        &self,
        project_id: &str,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<Analysis>, sqlx::Error> {
        let cache_key = format!("project_analyses:{}:{}:{}", project_id, limit, offset);
        let start = Instant::now();

        // Try cache first
        if let Some(cached_analyses) = self.cache.analysis_cache.get(&cache_key) {
            self.monitor
                .record_query("get_project_analyses".to_string(), start.elapsed(), true, 1)
                .await;
            return Ok(vec![cached_analyses]);
        }

        // Cache miss - query database with optimized query
        let analyses = sqlx::query_as::<_, Analysis>(
            "SELECT id, project_id, log_file_id, analysis_type, provider, level_filter, status, result, error_message, started_at, completed_at
             FROM analyses
             WHERE project_id = ?
             ORDER BY started_at DESC
             LIMIT ? OFFSET ?"
        )
        .bind(project_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await?;

        let execution_time = start.elapsed();
        let row_count = analyses.len() as u64;

        self.monitor
            .record_query(
                "get_project_analyses".to_string(),
                execution_time,
                false,
                row_count,
            )
            .await;

        Ok(analyses)
    }

    // Bulk performance metrics retrieval with caching
    pub async fn get_performance_metrics_cached(
        &self,
        analysis_id: &str,
    ) -> Result<Vec<PerformanceMetric>, sqlx::Error> {
        let cache_key = CacheKeys::metrics_key(analysis_id);
        let start = Instant::now();

        // Try cache first
        if let Some(cached_metrics) = self.cache.metrics_cache.get(&cache_key) {
            self.monitor
                .record_query(
                    "get_performance_metrics".to_string(),
                    start.elapsed(),
                    true,
                    cached_metrics.len() as u64,
                )
                .await;
            return Ok(cached_metrics);
        }

        // Cache miss - query database
        let metrics = sqlx::query_as::<_, PerformanceMetric>(
            "SELECT id, analysis_id, metric_name, metric_value, unit, threshold_value, is_bottleneck, created_at
             FROM performance_metrics
             WHERE analysis_id = ?
             ORDER BY is_bottleneck DESC, metric_name"
        )
        .bind(analysis_id)
        .fetch_all(&self.pool)
        .await?;

        let execution_time = start.elapsed();
        let row_count = metrics.len() as u64;

        // Cache the result
        self.cache
            .metrics_cache
            .put(cache_key, metrics.clone(), None);

        self.monitor
            .record_query(
                "get_performance_metrics".to_string(),
                execution_time,
                false,
                row_count,
            )
            .await;

        Ok(metrics)
    }

    // Batch correlation retrieval with caching
    pub async fn get_error_correlations_cached(
        &self,
        project_id: &str,
        analysis_id: &str,
    ) -> Result<Vec<ErrorCorrelation>, sqlx::Error> {
        let cache_key = CacheKeys::correlation_key(project_id, analysis_id);
        let start = Instant::now();

        // Try cache first
        if let Some(cached_correlations) = self.cache.correlation_cache.get(&cache_key) {
            self.monitor
                .record_query(
                    "get_error_correlations".to_string(),
                    start.elapsed(),
                    true,
                    cached_correlations.len() as u64,
                )
                .await;
            return Ok(cached_correlations);
        }

        // Cache miss - query database
        let correlations = sqlx::query_as::<_, ErrorCorrelation>(
            "SELECT id, project_id, primary_error_id, correlated_error_id, correlation_strength, correlation_type, created_at
             FROM error_correlations
             WHERE project_id = ? AND (primary_error_id = ? OR correlated_error_id = ?)
             ORDER BY correlation_strength DESC"
        )
        .bind(project_id)
        .bind(analysis_id)
        .bind(analysis_id)
        .fetch_all(&self.pool)
        .await?;

        let execution_time = start.elapsed();
        let row_count = correlations.len() as u64;

        // Cache the result
        self.cache
            .correlation_cache
            .put(cache_key, correlations.clone(), None);

        self.monitor
            .record_query(
                "get_error_correlations".to_string(),
                execution_time,
                false,
                row_count,
            )
            .await;

        Ok(correlations)
    }

    // Optimized knowledge base search with full-text search preparation
    pub async fn search_knowledge_base_optimized(
        &self,
        project_id: &str,
        search_term: &str,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<KnowledgeBaseEntry>, sqlx::Error> {
        let start = Instant::now();

        // Use indexed search when available
        let entries = if search_term.trim().is_empty() {
            sqlx::query_as::<_, KnowledgeBaseEntry>(
                "SELECT id, project_id, title, problem_description, solution, tags, severity, is_public, usage_count, created_at, updated_at
                 FROM knowledge_base
                 WHERE project_id = ?
                 ORDER BY usage_count DESC, created_at DESC
                 LIMIT ? OFFSET ?"
            )
            .bind(project_id)
            .bind(limit)
            .bind(offset)
            .fetch_all(&self.pool)
            .await?
        } else {
            let search_pattern = format!("%{}%", search_term);
            sqlx::query_as::<_, KnowledgeBaseEntry>(
                "SELECT id, project_id, title, problem_description, solution, tags, severity, is_public, usage_count, created_at, updated_at
                 FROM knowledge_base
                 WHERE project_id = ? AND (
                     title LIKE ? OR
                     problem_description LIKE ? OR
                     solution LIKE ?
                 )
                 ORDER BY
                     CASE
                         WHEN title LIKE ? THEN 1
                         WHEN problem_description LIKE ? THEN 2
                         ELSE 3
                     END,
                     usage_count DESC,
                     created_at DESC
                 LIMIT ? OFFSET ?"
            )
            .bind(project_id)
            .bind(&search_pattern)
            .bind(&search_pattern)
            .bind(&search_pattern)
            .bind(&search_pattern)
            .bind(&search_pattern)
            .bind(limit)
            .bind(offset)
            .fetch_all(&self.pool)
            .await?
        };

        let execution_time = start.elapsed();
        let row_count = entries.len() as u64;

        self.monitor
            .record_query(
                "search_knowledge_base".to_string(),
                execution_time,
                false,
                row_count,
            )
            .await;

        Ok(entries)
    }

    // Batch insert for performance metrics
    pub async fn bulk_insert_performance_metrics(
        &self,
        metrics: Vec<PerformanceMetric>,
    ) -> Result<(), sqlx::Error> {
        if metrics.is_empty() {
            return Ok(());
        }

        let start = Instant::now();

        let mut transaction = self.pool.begin().await?;

        for metric in &metrics {
            sqlx::query(
                "INSERT INTO performance_metrics (id, analysis_id, metric_name, metric_value, unit, threshold_value, is_bottleneck, created_at)
                 VALUES (?, ?, ?, ?, ?, ?, ?, ?)"
            )
            .bind(&metric.id)
            .bind(&metric.analysis_id)
            .bind(&metric.metric_name)
            .bind(metric.metric_value)
            .bind(&metric.unit)
            .bind(metric.threshold_value)
            .bind(metric.is_bottleneck)
            .bind(metric.created_at)
            .execute(&mut *transaction)
            .await?;
        }

        transaction.commit().await?;

        let execution_time = start.elapsed();
        self.monitor
            .record_query(
                "bulk_insert_performance_metrics".to_string(),
                execution_time,
                false,
                metrics.len() as u64,
            )
            .await;

        // Invalidate related cache
        for metric in &metrics {
            let cache_key = CacheKeys::metrics_key(&metric.analysis_id);
            self.cache.metrics_cache.remove(&cache_key);
        }

        Ok(())
    }

    // Cache invalidation for analysis updates
    pub async fn invalidate_analysis_cache(&self, project_id: &str, analysis_id: &str) {
        let analysis_key = CacheKeys::analysis_key(project_id, analysis_id);
        let result_key = CacheKeys::result_key(analysis_id);
        let metrics_key = CacheKeys::metrics_key(analysis_id);
        let correlation_key = CacheKeys::correlation_key(project_id, analysis_id);

        self.cache.analysis_cache.remove(&analysis_key);
        self.cache.result_cache.remove(&result_key);
        self.cache.metrics_cache.remove(&metrics_key);
        self.cache.correlation_cache.remove(&correlation_key);

        tracing::debug!("Invalidated cache for analysis {}", analysis_id);
    }

    // Get performance monitor for health checks
    pub fn get_monitor(&self) -> Arc<PerformanceMonitor> {
        Arc::clone(&self.monitor)
    }
}

// Database index optimization recommendations
pub async fn create_performance_indexes(pool: &Pool<Sqlite>) -> Result<(), sqlx::Error> {
    tracing::info!("Creating performance indexes...");

    // Analyses indexes
    sqlx::query("CREATE INDEX IF NOT EXISTS idx_analyses_project_started ON analyses(project_id, started_at DESC)")
        .execute(pool)
        .await?;

    sqlx::query("CREATE INDEX IF NOT EXISTS idx_analyses_status ON analyses(status)")
        .execute(pool)
        .await?;

    // Performance metrics indexes
    sqlx::query("CREATE INDEX IF NOT EXISTS idx_performance_metrics_analysis ON performance_metrics(analysis_id)")
        .execute(pool)
        .await?;

    sqlx::query("CREATE INDEX IF NOT EXISTS idx_performance_metrics_bottleneck ON performance_metrics(is_bottleneck, analysis_id)")
        .execute(pool)
        .await?;

    // Error correlations indexes
    sqlx::query("CREATE INDEX IF NOT EXISTS idx_error_correlations_primary ON error_correlations(primary_error_id)")
        .execute(pool)
        .await?;

    sqlx::query("CREATE INDEX IF NOT EXISTS idx_error_correlations_correlated ON error_correlations(correlated_error_id)")
        .execute(pool)
        .await?;

    sqlx::query("CREATE INDEX IF NOT EXISTS idx_error_correlations_strength ON error_correlations(correlation_strength DESC)")
        .execute(pool)
        .await?;

    // Knowledge base indexes
    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_knowledge_base_project ON knowledge_base(project_id)",
    )
    .execute(pool)
    .await?;

    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_knowledge_base_usage ON knowledge_base(usage_count DESC)",
    )
    .execute(pool)
    .await?;

    sqlx::query("CREATE INDEX IF NOT EXISTS idx_knowledge_base_search ON knowledge_base(project_id, title, problem_description)")
        .execute(pool)
        .await?;

    // Error patterns indexes
    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_error_patterns_project ON error_patterns(project_id)",
    )
    .execute(pool)
    .await?;

    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_error_patterns_frequency ON error_patterns(frequency DESC)",
    )
    .execute(pool)
    .await?;

    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_error_patterns_category ON error_patterns(category)",
    )
    .execute(pool)
    .await?;

    tracing::info!("Performance indexes created successfully");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cache::CacheManager;

    #[tokio::test]
    async fn test_performance_monitor() {
        let cache_manager = Arc::new(CacheManager::new());
        let monitor = PerformanceMonitor::new(cache_manager);

        monitor
            .record_query(
                "test_query".to_string(),
                Duration::from_millis(100),
                false,
                5,
            )
            .await;

        let summary = monitor.get_performance_summary().await;
        assert_eq!(summary.total_queries, 1);
        assert_eq!(summary.cache_hit_rate, 0.0);
        assert_eq!(summary.average_response_time_ms, 100.0);
    }
}
