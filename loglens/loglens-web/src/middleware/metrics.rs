use axum::{
    extract::{MatchedPath, Request},
    middleware::Next,
    response::Response,
};
use std::{
    collections::HashMap,
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc, RwLock,
    },
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};
use tokio::sync::broadcast;

/// Performance metrics collection and monitoring middleware
/// Tracks request latency, throughput, error rates, and resource usage
pub struct MetricsCollector {
    request_count: AtomicU64,
    error_count: AtomicU64,
    response_times: Arc<RwLock<Vec<Duration>>>,
    endpoint_metrics: Arc<RwLock<HashMap<String, EndpointMetrics>>>,
    quality_metrics: Arc<RwLock<QualityMetrics>>,
    start_time: SystemTime,
    alerts: broadcast::Sender<QualityAlert>,
}

#[derive(Clone, Debug)]
pub struct EndpointMetrics {
    pub path: String,
    pub request_count: u64,
    pub error_count: u64,
    pub total_duration: Duration,
    pub min_duration: Duration,
    pub max_duration: Duration,
    pub avg_duration: Duration,
    pub p95_duration: Duration,
    pub p99_duration: Duration,
    pub last_accessed: SystemTime,
}

#[derive(Clone, Debug)]
pub struct QualityMetrics {
    pub analysis_accuracy: f64,
    pub analysis_completion_rate: f64,
    pub average_confidence_score: f64,
    pub error_classification_accuracy: f64,
    pub system_availability: f64,
    pub memory_usage_mb: u64,
    pub cpu_usage_percent: f64,
    pub active_connections: u64,
    pub database_query_avg_time: Duration,
    pub cache_hit_rate: f64,
}

#[derive(Clone, Debug)]
pub struct QualityAlert {
    pub alert_type: AlertType,
    pub message: String,
    pub severity: AlertSeverity,
    pub timestamp: SystemTime,
    pub metrics: HashMap<String, serde_json::Value>,
}

#[derive(Clone, Debug)]
pub enum AlertType {
    HighResponseTime,
    HighErrorRate,
    LowAccuracy,
    ResourceExhaustion,
    SystemOverload,
    QualityDegradation,
}

#[derive(Clone, Debug)]
pub enum AlertSeverity {
    Low,
    Medium,
    High,
    Critical,
}

impl Default for MetricsCollector {
    fn default() -> Self {
        Self::new()
    }
}

impl MetricsCollector {
    pub fn new() -> Self {
        let (alerts_tx, _) = broadcast::channel(1000);

        Self {
            request_count: AtomicU64::new(0),
            error_count: AtomicU64::new(0),
            response_times: Arc::new(RwLock::new(Vec::new())),
            endpoint_metrics: Arc::new(RwLock::new(HashMap::new())),
            quality_metrics: Arc::new(RwLock::new(QualityMetrics::default())),
            start_time: SystemTime::now(),
            alerts: alerts_tx,
        }
    }

    pub fn start_background_tasks(self: Arc<Self>) {
        let collector_clone = self.clone();
        tokio::spawn(async move {
            collector_clone.quality_monitoring_task().await;
        });

        let collector_clone = self.clone();
        tokio::spawn(async move {
            collector_clone.metrics_cleanup_task().await;
        });

        let collector_clone = self.clone();
        tokio::spawn(async move {
            collector_clone.system_monitoring_task().await;
        });
    }

    async fn quality_monitoring_task(&self) {
        let mut interval = tokio::time::interval(Duration::from_secs(60));

        loop {
            interval.tick().await;

            if let Err(e) = self.check_quality_thresholds().await {
                tracing::error!("Quality monitoring error: {}", e);
            }
        }
    }

    async fn metrics_cleanup_task(&self) {
        let mut interval = tokio::time::interval(Duration::from_secs(300)); // 5 minutes

        loop {
            interval.tick().await;

            // Clean up old response time data (keep last 1000 entries)
            {
                let mut response_times = self.response_times.write().unwrap();
                let len = response_times.len();
                if len > 1000 {
                    response_times.drain(0..len - 1000);
                }
            }

            // Clean up old endpoint metrics (keep active endpoints)
            {
                let mut endpoint_metrics = self.endpoint_metrics.write().unwrap();
                let now = SystemTime::now();
                endpoint_metrics.retain(|_, metrics| {
                    now.duration_since(metrics.last_accessed)
                        .unwrap_or_else(|_| Duration::from_secs(0))
                        .as_secs() < 3600 // Keep if accessed within last hour
                });
            }
        }
    }

    async fn system_monitoring_task(&self) {
        let mut interval = tokio::time::interval(Duration::from_secs(30));

        loop {
            interval.tick().await;

            if let Err(e) = self.update_system_metrics().await {
                tracing::error!("System monitoring error: {}", e);
            }
        }
    }

    async fn check_quality_thresholds(&self) -> Result<(), anyhow::Error> {
        let metrics = self.quality_metrics.read().unwrap().clone();
        let mut alerts = Vec::new();

        // Check response time threshold
        let avg_response_time = self.get_average_response_time();
        if avg_response_time > Duration::from_millis(5000) {
            alerts.push(QualityAlert {
                alert_type: AlertType::HighResponseTime,
                message: format!("Average response time is {}ms, exceeding 5000ms threshold", avg_response_time.as_millis()),
                severity: AlertSeverity::High,
                timestamp: SystemTime::now(),
                metrics: [("avg_response_time_ms".to_string(), serde_json::Value::Number(serde_json::Number::from(avg_response_time.as_millis() as u64)))].into(),
            });
        }

        // Check error rate threshold
        let error_rate = self.get_error_rate();
        if error_rate > 0.05 { // 5% error rate threshold
            alerts.push(QualityAlert {
                alert_type: AlertType::HighErrorRate,
                message: format!("Error rate is {:.2}%, exceeding 5% threshold", error_rate * 100.0),
                severity: AlertSeverity::High,
                timestamp: SystemTime::now(),
                metrics: [("error_rate".to_string(), serde_json::Value::Number(serde_json::Number::from_f64(error_rate).unwrap()))].into(),
            });
        }

        // Check analysis accuracy threshold
        if metrics.analysis_accuracy < 0.80 { // 80% accuracy threshold
            alerts.push(QualityAlert {
                alert_type: AlertType::LowAccuracy,
                message: format!("Analysis accuracy is {:.2}%, below 80% threshold", metrics.analysis_accuracy * 100.0),
                severity: AlertSeverity::Medium,
                timestamp: SystemTime::now(),
                metrics: [("analysis_accuracy".to_string(), serde_json::Value::Number(serde_json::Number::from_f64(metrics.analysis_accuracy).unwrap()))].into(),
            });
        }

        // Check system resource thresholds
        if metrics.memory_usage_mb > 2048 { // 2GB memory threshold
            alerts.push(QualityAlert {
                alert_type: AlertType::ResourceExhaustion,
                message: format!("Memory usage is {}MB, exceeding 2048MB threshold", metrics.memory_usage_mb),
                severity: AlertSeverity::High,
                timestamp: SystemTime::now(),
                metrics: [("memory_usage_mb".to_string(), serde_json::Value::Number(metrics.memory_usage_mb.into()))].into(),
            });
        }

        if metrics.cpu_usage_percent > 80.0 { // 80% CPU threshold
            alerts.push(QualityAlert {
                alert_type: AlertType::SystemOverload,
                message: format!("CPU usage is {:.1}%, exceeding 80% threshold", metrics.cpu_usage_percent),
                severity: AlertSeverity::Medium,
                timestamp: SystemTime::now(),
                metrics: [("cpu_usage_percent".to_string(), serde_json::Value::Number(serde_json::Number::from_f64(metrics.cpu_usage_percent).unwrap()))].into(),
            });
        }

        // Send alerts
        for alert in alerts {
            if self.alerts.send(alert.clone()).is_err() {
                tracing::warn!("Failed to send quality alert: {:?}", alert);
            }
        }

        Ok(())
    }

    async fn update_system_metrics(&self) -> Result<(), anyhow::Error> {
        let memory_usage = self.get_memory_usage().await;
        let cpu_usage = self.get_cpu_usage().await;
        let active_connections = self.get_active_connections().await;

        {
            let mut quality_metrics = self.quality_metrics.write().unwrap();
            quality_metrics.memory_usage_mb = memory_usage;
            quality_metrics.cpu_usage_percent = cpu_usage;
            quality_metrics.active_connections = active_connections;
        }

        Ok(())
    }

    async fn get_memory_usage(&self) -> u64 {
        // Simple memory usage estimation
        // In a real implementation, use process information or system metrics
        std::process::id() as u64 * 1024 // Placeholder
    }

    async fn get_cpu_usage(&self) -> f64 {
        // Simple CPU usage estimation
        // In a real implementation, use system metrics
        (SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() % 100) as f64 / 10.0
    }

    async fn get_active_connections(&self) -> u64 {
        // Track active WebSocket connections
        // In a real implementation, maintain a connection counter
        42 // Placeholder
    }

    pub fn record_request(&self, path: &str, duration: Duration, status_code: u16) {
        self.request_count.fetch_add(1, Ordering::Relaxed);

        if status_code >= 400 {
            self.error_count.fetch_add(1, Ordering::Relaxed);
        }

        // Record response time
        {
            let mut response_times = self.response_times.write().unwrap();
            response_times.push(duration);
        }

        // Update endpoint metrics
        {
            let mut endpoint_metrics = self.endpoint_metrics.write().unwrap();
            let metrics = endpoint_metrics.entry(path.to_string()).or_insert_with(|| {
                EndpointMetrics {
                    path: path.to_string(),
                    request_count: 0,
                    error_count: 0,
                    total_duration: Duration::ZERO,
                    min_duration: duration,
                    max_duration: duration,
                    avg_duration: duration,
                    p95_duration: duration,
                    p99_duration: duration,
                    last_accessed: SystemTime::now(),
                }
            });

            metrics.request_count += 1;
            if status_code >= 400 {
                metrics.error_count += 1;
            }
            metrics.total_duration += duration;
            metrics.min_duration = metrics.min_duration.min(duration);
            metrics.max_duration = metrics.max_duration.max(duration);
            metrics.avg_duration = metrics.total_duration / metrics.request_count as u32;
            metrics.last_accessed = SystemTime::now();

            // Calculate percentiles (simplified approach)
            if metrics.request_count > 20 {
                metrics.p95_duration = duration.mul_f64(1.2); // Approximation
                metrics.p99_duration = duration.mul_f64(1.4); // Approximation
            }
        }
    }

    pub fn record_analysis_result(&self, accuracy: f64, confidence: f64, success: bool) {
        let mut quality_metrics = self.quality_metrics.write().unwrap();

        // Update moving averages (simplified)
        quality_metrics.analysis_accuracy = (quality_metrics.analysis_accuracy * 0.9) + (accuracy * 0.1);
        quality_metrics.average_confidence_score = (quality_metrics.average_confidence_score * 0.9) + (confidence * 0.1);

        if success {
            quality_metrics.analysis_completion_rate = (quality_metrics.analysis_completion_rate * 0.9) + 0.1;
        } else {
            quality_metrics.analysis_completion_rate *= 0.9;
        }
    }

    pub fn get_metrics_summary(&self) -> MetricsSummary {
        let request_count = self.request_count.load(Ordering::Relaxed);
        let error_count = self.error_count.load(Ordering::Relaxed);
        let error_rate = if request_count > 0 {
            error_count as f64 / request_count as f64
        } else {
            0.0
        };

        let avg_response_time = self.get_average_response_time();
        let uptime = SystemTime::now().duration_since(self.start_time).unwrap_or_default();

        let quality_metrics = self.quality_metrics.read().unwrap().clone();
        let endpoint_metrics: Vec<EndpointMetrics> = self.endpoint_metrics.read().unwrap().values().cloned().collect();

        MetricsSummary {
            request_count,
            error_count,
            error_rate,
            avg_response_time,
            uptime,
            quality_metrics,
            endpoint_metrics,
        }
    }

    pub fn get_average_response_time(&self) -> Duration {
        let response_times = self.response_times.read().unwrap();
        if response_times.is_empty() {
            return Duration::ZERO;
        }

        let total: Duration = response_times.iter().sum();
        total / response_times.len() as u32
    }

    pub fn get_error_rate(&self) -> f64 {
        let request_count = self.request_count.load(Ordering::Relaxed);
        let error_count = self.error_count.load(Ordering::Relaxed);

        if request_count > 0 {
            error_count as f64 / request_count as f64
        } else {
            0.0
        }
    }

    pub fn subscribe_to_alerts(&self) -> broadcast::Receiver<QualityAlert> {
        self.alerts.subscribe()
    }
}

#[derive(Clone, Debug)]
pub struct MetricsSummary {
    pub request_count: u64,
    pub error_count: u64,
    pub error_rate: f64,
    pub avg_response_time: Duration,
    pub uptime: Duration,
    pub quality_metrics: QualityMetrics,
    pub endpoint_metrics: Vec<EndpointMetrics>,
}

impl Default for QualityMetrics {
    fn default() -> Self {
        Self {
            analysis_accuracy: 0.85,
            analysis_completion_rate: 0.95,
            average_confidence_score: 0.75,
            error_classification_accuracy: 0.80,
            system_availability: 0.99,
            memory_usage_mb: 512,
            cpu_usage_percent: 25.0,
            active_connections: 0,
            database_query_avg_time: Duration::from_millis(50),
            cache_hit_rate: 0.85,
        }
    }
}

/// Axum middleware for metrics collection
pub async fn metrics_middleware(
    request: Request,
    next: Next,
) -> Response {
    let start = Instant::now();
    let path = request.extensions()
        .get::<MatchedPath>()
        .map(|p| p.as_str())
        .unwrap_or("unknown")
        .to_string();

    let response = next.run(request).await;
    let duration = start.elapsed();
    let status = response.status().as_u16();

    // Record metrics (this would need access to the MetricsCollector instance)
    // In practice, you'd store the collector in app state and access it here
    tracing::info!(
        "Request: {} {} - {}ms - Status: {}",
        path,
        duration.as_millis(),
        status,
        if status >= 400 { "ERROR" } else { "OK" }
    );

    response
}

/// Metrics endpoint handler
pub async fn metrics_handler(
    collector: Arc<MetricsCollector>,
) -> Result<axum::response::Json<MetricsSummary>, axum::http::StatusCode> {
    Ok(axum::response::Json(collector.get_metrics_summary()))
}

/// Health check with quality metrics
pub async fn health_with_metrics_handler(
    collector: Arc<MetricsCollector>,
) -> Result<axum::response::Json<HealthResponse>, axum::http::StatusCode> {
    let metrics = collector.get_metrics_summary();

    let status = if metrics.error_rate < 0.05
        && metrics.avg_response_time < Duration::from_millis(5000)
        && metrics.quality_metrics.system_availability > 0.95 {
        "healthy"
    } else {
        "degraded"
    };

    Ok(axum::response::Json(HealthResponse {
        status: status.to_string(),
        uptime_seconds: metrics.uptime.as_secs(),
        request_count: metrics.request_count,
        error_rate: metrics.error_rate,
        avg_response_time_ms: metrics.avg_response_time.as_millis() as u64,
        quality_score: calculate_quality_score(&metrics.quality_metrics),
    }))
}

fn calculate_quality_score(metrics: &QualityMetrics) -> f64 {
    // Weighted quality score calculation
    let weights = [
        (metrics.analysis_accuracy, 0.30),
        (metrics.analysis_completion_rate, 0.20),
        (metrics.average_confidence_score, 0.20),
        (metrics.error_classification_accuracy, 0.15),
        (metrics.system_availability, 0.15),
    ];

    weights.iter().map(|(value, weight)| value * weight).sum()
}

#[derive(serde::Serialize)]
pub struct HealthResponse {
    pub status: String,
    pub uptime_seconds: u64,
    pub request_count: u64,
    pub error_rate: f64,
    pub avg_response_time_ms: u64,
    pub quality_score: f64,
}