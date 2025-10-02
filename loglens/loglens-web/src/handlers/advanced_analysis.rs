use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

use crate::{models::*, AppState};

#[derive(Debug, Serialize, Deserialize)]
pub struct CorrelationAnalysisRequest {
    pub analysis_ids: Vec<String>,
    pub correlation_threshold: Option<f64>,
    pub time_window_minutes: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CorrelationAnalysisResponse {
    pub correlations: Vec<ErrorCorrelation>,
    pub analysis_summary: HashMap<String, AnalysisSummary>,
    pub confidence_score: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AnalysisSummary {
    pub id: String,
    pub error_count: i32,
    pub error_types: Vec<String>,
    pub time_range: (String, String),
    pub severity_distribution: HashMap<String, i32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AnomalyDetectionRequest {
    pub analysis_id: String,
    pub sensitivity: Option<f64>, // 0.0 to 1.0
    pub lookback_hours: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AnomalyDetectionResponse {
    pub anomalies: Vec<AnomalyResult>,
    pub confidence_scores: HashMap<String, f64>,
    pub summary: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AnomalyResult {
    pub pattern: String,
    pub confidence: f64,
    pub severity: String,
    pub description: String,
    pub suggested_action: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MultiLogAnalysisRequest {
    pub project_id: String,
    pub log_files: Vec<String>,
    pub analysis_type: String, // "comparative", "trend", "correlation"
    pub provider: String,
    pub level_filter: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MultiLogAnalysisResponse {
    pub cross_file_patterns: Vec<CrossFilePattern>,
    pub file_comparison: HashMap<String, FileAnalysis>,
    pub summary: MultiLogSummary,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CrossFilePattern {
    pub pattern: String,
    pub files_affected: Vec<String>,
    pub frequency: i32,
    pub severity: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FileAnalysis {
    pub file_id: String,
    pub error_count: i32,
    pub unique_errors: i32,
    pub top_errors: Vec<String>,
    pub performance_metrics: Vec<PerformanceMetric>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MultiLogSummary {
    pub total_files: i32,
    pub total_errors: i32,
    pub common_patterns: Vec<String>,
    pub overall_severity: String,
}

// Cross-error correlation analysis
pub async fn analyze_correlations(
    State(state): State<AppState>,
    Path(project_id): Path<String>,
    Json(req): Json<CorrelationAnalysisRequest>,
) -> Result<Json<CorrelationAnalysisResponse>, StatusCode> {
    if req.analysis_ids.len() < 2 {
        return Err(StatusCode::BAD_REQUEST);
    }

    let threshold = req.correlation_threshold.unwrap_or(0.7);
    let time_window = req.time_window_minutes.unwrap_or(60);

    // Get all analyses
    let mut analyses = Vec::new();
    for analysis_id in &req.analysis_ids {
        if let Some(analysis) = sqlx::query_as::<_, Analysis>(
            "SELECT id, project_id, log_file_id, analysis_type, provider, level_filter, status, result, error_message, started_at, completed_at
             FROM analyses WHERE id = ? AND project_id = ?"
        )
        .bind(analysis_id)
        .bind(&project_id)
        .fetch_optional(state.db.pool())
        .await
        .map_err(|_: sqlx::Error| StatusCode::INTERNAL_SERVER_ERROR)?
        {
            analyses.push(analysis);
        }
    }

    // Perform correlation analysis
    let mut correlations = Vec::new();
    let mut analysis_summaries = HashMap::new();

    for (i, analysis1) in analyses.iter().enumerate() {
        for analysis2 in analyses.iter().skip(i + 1) {
            if let Some(correlation) =
                calculate_correlation(analysis1, analysis2, threshold, time_window)
            {
                correlations.push(correlation);
            }
        }

        // Create analysis summary
        let summary = create_analysis_summary(analysis1);
        analysis_summaries.insert(analysis1.id.clone(), summary);
    }

    // Calculate overall confidence score
    let confidence_score = if correlations.is_empty() {
        0.0
    } else {
        let avg_strength: f64 = correlations
            .iter()
            .map(|c| c.correlation_strength)
            .sum::<f64>()
            / correlations.len() as f64;
        avg_strength
    };

    Ok(Json(CorrelationAnalysisResponse {
        correlations,
        analysis_summary: analysis_summaries,
        confidence_score,
    }))
}

// Anomaly detection with confidence scoring
pub async fn detect_anomalies(
    State(state): State<AppState>,
    Path(project_id): Path<String>,
    Json(req): Json<AnomalyDetectionRequest>,
) -> Result<Json<AnomalyDetectionResponse>, StatusCode> {
    let sensitivity = req.sensitivity.unwrap_or(0.8);
    let lookback_hours = req.lookback_hours.unwrap_or(24);

    // Get the target analysis
    let analysis = sqlx::query_as::<_, Analysis>(
        "SELECT id, project_id, log_file_id, analysis_type, provider, level_filter, status, result, error_message, started_at, completed_at
         FROM analyses WHERE id = ? AND project_id = ?"
    )
    .bind(&req.analysis_id)
    .bind(&project_id)
    .fetch_optional(state.db.pool())
    .await
    .map_err(|_: sqlx::Error| StatusCode::INTERNAL_SERVER_ERROR)?
    .ok_or(StatusCode::NOT_FOUND)?;

    // Get historical analyses for comparison
    let historical_analyses = sqlx::query_as::<_, Analysis>(
        "SELECT id, project_id, log_file_id, analysis_type, provider, level_filter, status, result, error_message, started_at, completed_at
         FROM analyses 
         WHERE project_id = ? AND id != ? AND started_at >= datetime('now', '-{} hours')
         ORDER BY started_at DESC",
    )
    .bind(&project_id)
    .bind(&req.analysis_id)
    .bind(lookback_hours)
    .fetch_all(state.db.pool())
    .await
    .map_err(|_: sqlx::Error| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Detect anomalies
    let anomalies = detect_anomalies_in_analysis(&analysis, &historical_analyses, sensitivity);

    // Calculate confidence scores
    let confidence_scores = calculate_confidence_scores(&anomalies, &historical_analyses);

    let summary = format!(
        "Detected {} anomalies in analysis {} with sensitivity threshold {:.2}",
        anomalies.len(),
        req.analysis_id,
        sensitivity
    );

    Ok(Json(AnomalyDetectionResponse {
        anomalies,
        confidence_scores,
        summary,
    }))
}

// Multi-log file analysis
pub async fn analyze_multiple_logs(
    State(state): State<AppState>,
    Path(project_id): Path<String>,
    Json(req): Json<MultiLogAnalysisRequest>,
) -> Result<Json<MultiLogAnalysisResponse>, StatusCode> {
    if req.log_files.is_empty() {
        return Err(StatusCode::BAD_REQUEST);
    }

    let mut file_analyses = HashMap::new();
    let mut all_errors = Vec::new();
    let mut cross_file_patterns = Vec::new();

    // Analyze each log file
    for file_id in &req.log_files {
        let analyses = sqlx::query_as::<_, Analysis>(
            "SELECT id, project_id, log_file_id, analysis_type, provider, level_filter, status, result, error_message, started_at, completed_at
             FROM analyses WHERE project_id = ? AND log_file_id = ? AND provider = ? AND level_filter = ?
             ORDER BY started_at DESC LIMIT 1"
        )
        .bind(&project_id)
        .bind(file_id)
        .bind(&req.provider)
        .bind(&req.level_filter)
        .fetch_all(state.db.pool())
        .await
        .map_err(|_: sqlx::Error| StatusCode::INTERNAL_SERVER_ERROR)?;

        if let Some(latest_analysis) = analyses.first() {
            // Extract errors from analysis result
            if let Some(result_json) = &latest_analysis.result {
                if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(result_json) {
                    if let Some(errors) = parsed.get("errors").and_then(|e| e.as_array()) {
                        for error in errors {
                            if let Some(error_msg) = error.as_str() {
                                all_errors.push((file_id.clone(), error_msg.to_string()));
                            }
                        }
                    }
                }
            }

            // Get performance metrics
            let metrics = sqlx::query_as::<_, PerformanceMetric>(
                "SELECT id, analysis_id, metric_name, metric_value, unit, threshold_value, is_bottleneck, created_at
                 FROM performance_metrics WHERE analysis_id = ?"
            )
            .bind(&latest_analysis.id)
            .fetch_all(state.db.pool())
            .await
            .map_err(|_: sqlx::Error| StatusCode::INTERNAL_SERVER_ERROR)?;

            file_analyses.insert(
                file_id.clone(),
                FileAnalysis {
                    file_id: file_id.clone(),
                    error_count: all_errors.iter().filter(|(f, _)| f == file_id).count() as i32,
                    unique_errors: HashSet::<String>::from_iter(
                        all_errors
                            .iter()
                            .filter(|(f, _)| f == file_id)
                            .map(|(_, e)| e.clone()),
                    )
                    .len() as i32,
                    top_errors: extract_top_errors(&all_errors, file_id, 5),
                    performance_metrics: metrics,
                },
            );
        }
    }

    // Find cross-file patterns
    cross_file_patterns = find_cross_file_patterns(&all_errors);

    // Create summary
    let summary = MultiLogSummary {
        total_files: req.log_files.len() as i32,
        total_errors: all_errors.len() as i32,
        common_patterns: extract_common_patterns(&cross_file_patterns, 3),
        overall_severity: calculate_overall_severity(&file_analyses),
    };

    Ok(Json(MultiLogAnalysisResponse {
        cross_file_patterns,
        file_comparison: file_analyses,
        summary,
    }))
}

// Helper functions
fn calculate_correlation(
    analysis1: &Analysis,
    analysis2: &Analysis,
    threshold: f64,
    time_window_minutes: i64,
) -> Option<ErrorCorrelation> {
    // Simple correlation based on time proximity and error similarity
    let time_diff = analysis1
        .started_at
        .signed_duration_since(analysis2.started_at)
        .num_minutes()
        .abs();

    if time_diff > time_window_minutes {
        return None;
    }

    // Calculate correlation strength based on multiple factors
    let time_factor = 1.0 - (time_diff as f64 / time_window_minutes as f64);
    let provider_factor = if analysis1.provider == analysis2.provider {
        1.0
    } else {
        0.5
    };
    let level_factor = if analysis1.level_filter == analysis2.level_filter {
        1.0
    } else {
        0.3
    };

    let correlation_strength =
        (time_factor * 0.4 + provider_factor * 0.3 + level_factor * 0.3).min(1.0);

    if correlation_strength >= threshold {
        Some(ErrorCorrelation::new(
            analysis1.project_id.clone(),
            analysis1.id.clone(),
            analysis2.id.clone(),
            correlation_strength,
            "temporal".to_string(),
        ))
    } else {
        None
    }
}

fn create_analysis_summary(analysis: &Analysis) -> AnalysisSummary {
    // Extract error count and types from analysis result
    let (error_count, error_types, severity_distribution) =
        if let Some(result_json) = &analysis.result {
            if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(result_json) {
                let count = parsed
                    .get("error_count")
                    .and_then(|c| c.as_i64())
                    .unwrap_or(0) as i32;
                let types = parsed
                    .get("error_types")
                    .and_then(|t| t.as_array())
                    .map(|arr| {
                        arr.iter()
                            .filter_map(|t| t.as_str().map(|s| s.to_string()))
                            .collect()
                    })
                    .unwrap_or_default();

                let mut severity_dist = HashMap::new();
                if let Some(severities) = parsed
                    .get("severity_distribution")
                    .and_then(|s| s.as_object())
                {
                    for (severity, count) in severities {
                        if let Some(count_val) = count.as_i64() {
                            severity_dist.insert(severity.clone(), count_val as i32);
                        }
                    }
                }

                (count, types, severity_dist)
            } else {
                (0, Vec::new(), HashMap::new())
            }
        } else {
            (0, Vec::new(), HashMap::new())
        };

    AnalysisSummary {
        id: analysis.id.clone(),
        error_count,
        error_types,
        time_range: (
            analysis.started_at.to_rfc3339(),
            analysis
                .completed_at
                .map(|dt| dt.to_rfc3339())
                .unwrap_or_default(),
        ),
        severity_distribution,
    }
}

fn detect_anomalies_in_analysis(
    analysis: &Analysis,
    historical_analyses: &[Analysis],
    sensitivity: f64,
) -> Vec<AnomalyResult> {
    let mut anomalies = Vec::new();

    // Simple anomaly detection based on deviation from historical patterns
    if let Some(result_json) = &analysis.result {
        if let Ok(current) = serde_json::from_str::<serde_json::Value>(result_json) {
            let current_error_count = current
                .get("error_count")
                .and_then(|c| c.as_i64())
                .unwrap_or(0);

            // Calculate historical average
            let historical_counts: Vec<i64> = historical_analyses
                .iter()
                .filter_map(|h| h.result.as_ref())
                .filter_map(|r| serde_json::from_str::<serde_json::Value>(r).ok())
                .map(|p| p.get("error_count").and_then(|c| c.as_i64()).unwrap_or(0))
                .collect();

            if !historical_counts.is_empty() {
                let avg_historical: f64 =
                    historical_counts.iter().sum::<i64>() as f64 / historical_counts.len() as f64;
                let deviation =
                    (current_error_count as f64 - avg_historical).abs() / avg_historical;

                if deviation > sensitivity {
                    anomalies.push(AnomalyResult {
                        pattern: "error_count_anomaly".to_string(),
                        confidence: deviation.min(1.0),
                        severity: if deviation > 2.0 { "high" } else { "medium" }.to_string(),
                        description: format!(
                            "Error count {} is {:.1}% higher than historical average",
                            current_error_count,
                            deviation * 100.0
                        ),
                        suggested_action: "Investigate recent changes or system load".to_string(),
                    });
                }
            }
        }
    }

    anomalies
}

fn calculate_confidence_scores(
    anomalies: &[AnomalyResult],
    historical_analyses: &[Analysis],
) -> HashMap<String, f64> {
    let mut scores = HashMap::new();

    for anomaly in anomalies {
        let confidence = if historical_analyses.is_empty() {
            0.5 // Default confidence when no historical data
        } else {
            // Adjust confidence based on historical data availability
            let data_factor = (historical_analyses.len() as f64 / 10.0).min(1.0);
            anomaly.confidence * data_factor
        };

        scores.insert(anomaly.pattern.clone(), confidence);
    }

    scores
}

fn extract_top_errors(errors: &[(String, String)], file_id: &str, limit: usize) -> Vec<String> {
    let file_errors: Vec<&String> = errors
        .iter()
        .filter(|(f, _)| f == file_id)
        .map(|(_, e)| e)
        .collect();

    let mut error_counts = HashMap::new();
    for error in file_errors {
        *error_counts.entry(error).or_insert(0) += 1;
    }

    let mut sorted_errors: Vec<_> = error_counts.into_iter().collect();
    sorted_errors.sort_by(|a, b| b.1.cmp(&a.1));

    sorted_errors
        .into_iter()
        .take(limit)
        .map(|(error, _)| error.clone())
        .collect()
}

fn find_cross_file_patterns(errors: &[(String, String)]) -> Vec<CrossFilePattern> {
    let mut pattern_counts = HashMap::new();

    // Simple pattern matching - look for common error substrings
    for (_, error) in errors {
        // Extract potential patterns (simplified)
        let patterns = extract_error_patterns(error);
        for pattern in patterns {
            let _entry = pattern_counts.entry(pattern).or_insert(HashSet::new());
            // In a real implementation, we'd track which files have this pattern
        }
    }

    // Convert to cross-file patterns
    pattern_counts
        .into_iter()
        .filter(|(_, files)| files.len() > 1) // Only patterns affecting multiple files
        .map(|(pattern, files)| CrossFilePattern {
            pattern,
            files_affected: files.into_iter().collect(),
            frequency: 1, // Would be calculated from actual occurrences
            severity: "medium".to_string(), // Would be calculated
        })
        .collect()
}

fn extract_error_patterns(error: &str) -> Vec<String> {
    // Simplified pattern extraction - in real implementation, use more sophisticated NLP
    let mut patterns = Vec::new();

    // Look for common error indicators
    if error.contains("connection") || error.contains("timeout") {
        patterns.push("network_error".to_string());
    }
    if error.contains("permission") || error.contains("access") {
        patterns.push("permission_error".to_string());
    }
    if error.contains("null") || error.contains("undefined") {
        patterns.push("null_reference".to_string());
    }
    if error.contains("memory") || error.contains("allocation") {
        patterns.push("memory_error".to_string());
    }

    patterns
}

fn extract_common_patterns(patterns: &[CrossFilePattern], limit: usize) -> Vec<String> {
    patterns
        .iter()
        .take(limit)
        .map(|p| p.pattern.clone())
        .collect()
}

fn calculate_overall_severity(file_analyses: &HashMap<String, FileAnalysis>) -> String {
    let total_errors: i32 = file_analyses.values().map(|f| f.error_count).sum();
    let bottleneck_count: i32 = file_analyses
        .values()
        .map(|f| {
            f.performance_metrics
                .iter()
                .filter(|m| m.is_bottleneck)
                .count() as i32
        })
        .sum();

    if total_errors > 1000 || bottleneck_count > 5 {
        "critical"
    } else if total_errors > 100 || bottleneck_count > 2 {
        "high"
    } else if total_errors > 10 || bottleneck_count > 0 {
        "medium"
    } else {
        "low"
    }
    .to_string()
}
