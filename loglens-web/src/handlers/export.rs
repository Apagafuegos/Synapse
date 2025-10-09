use axum::{
    extract::{Path, Query, State},
    http::header,
    response::{Json, Response},
};
use serde::{Deserialize, Serialize};
use std::process::Command;
use std::io::Write;
use tempfile::NamedTempFile;

use crate::{error_handling::AppError, models::*, AppState};

#[derive(Deserialize)]
pub struct ExportQuery {
    pub format: String, // "html", "pdf", "json", "csv"
    pub include_charts: Option<bool>,
    pub include_correlations: Option<bool>,
    pub template: Option<String>, // For HTML exports
}

#[derive(Deserialize)]
pub struct ShareRequest {
    pub analysis_id: String,
    pub expires_in_hours: Option<i64>, // Default 24 hours
    pub password_protect: Option<bool>,
    pub allow_download: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ShareResponse {
    pub share_id: String,
    pub share_url: String,
    pub expires_at: String,
    pub is_password_protected: bool,
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ExportMetadata {
    pub analysis_id: String,
    pub project_id: String,
    pub export_format: String,
    pub file_size: Option<u64>,
    pub created_at: String,
    pub download_url: Option<String>,
}

// HTML Report Generation
pub async fn export_html_report(
    State(state): State<AppState>,
    Path((project_id, analysis_id)): Path<(String, String)>,
    Query(params): Query<ExportQuery>,
) -> Result<Response, AppError> {
    tracing::info!("Starting HTML export for analysis {} in project {}", analysis_id, project_id);
    
    // Get analysis data
    let analysis = get_analysis_with_related_data(state, &project_id, &analysis_id).await
        .map_err(|e| {
            tracing::error!("Failed to fetch analysis data for HTML export: {}", e);
            e
        })?;

    // Generate HTML report
    let html_content = generate_html_report(&analysis, params.include_charts.unwrap_or(true));
    
    if html_content.is_empty() {
        tracing::error!("Generated HTML content is empty for analysis {}", analysis_id);
        return Err(AppError::internal("Failed to generate HTML content: empty result"));
    }

    tracing::info!("Successfully generated HTML export for analysis {} (size: {} bytes)", analysis_id, html_content.len());

    Response::builder()
        .header(header::CONTENT_TYPE, "text/html; charset=utf-8")
        .header(
            header::CONTENT_DISPOSITION,
            format!("attachment; filename=\"loglens_analysis_{}.html\"", analysis_id),
        )
        .body(html_content.into())
        .map_err(|e: axum::http::Error| { tracing::error!("HTTP response error: {}", e); AppError::internal(format!("Failed to build response: {}", e)) })
}

// JSON Export
pub async fn export_json_data(
    State(state): State<AppState>,
    Path((project_id, analysis_id)): Path<(String, String)>,
) -> Result<Response, AppError> {
    let analysis = get_analysis_with_related_data(state, &project_id, &analysis_id).await?;

    let json_data =
        serde_json::to_string_pretty(&analysis).map_err(|e: serde_json::Error| { tracing::error!("JSON serialization error: {}", e); AppError::internal(format!("Failed to serialize JSON: {}", e)) })?;

    Response::builder()
        .header(header::CONTENT_TYPE, "application/json")
        .header(
            header::CONTENT_DISPOSITION,
            format!("attachment; filename=\"loglens_analysis_{}.json\"", analysis_id),
        )
        .body(json_data.into())
        .map_err(|e: axum::http::Error| { tracing::error!("HTTP response error: {}", e); AppError::internal(format!("Failed to build response: {}", e)) })
}

// CSV Export
pub async fn export_csv_data(
    State(state): State<AppState>,
    Path((project_id, analysis_id)): Path<(String, String)>,
) -> Result<Response, AppError> {
    let analysis = get_analysis_with_related_data(state, &project_id, &analysis_id).await?;

    let csv_content = generate_csv_report(&analysis, &analysis_id);

    Response::builder()
        .header(header::CONTENT_TYPE, "text/csv")
        .header(
            header::CONTENT_DISPOSITION,
            format!("attachment; filename=\"loglens_analysis_{}.csv\"", analysis_id),
        )
        .body(csv_content.into())
        .map_err(|e: axum::http::Error| { tracing::error!("HTTP response error: {}", e); AppError::internal(format!("Failed to build response: {}", e)) })
}

// Markdown Export
pub async fn export_markdown_report(
    State(state): State<AppState>,
    Path((project_id, analysis_id)): Path<(String, String)>,
) -> Result<Response, AppError> {
    let analysis = get_analysis_with_related_data(state, &project_id, &analysis_id).await?;

    let markdown_content = generate_markdown_report(&analysis, &analysis_id);

    Response::builder()
        .header(header::CONTENT_TYPE, "text/markdown")
        .header(
            header::CONTENT_DISPOSITION,
            format!("attachment; filename=\"loglens_analysis_{}.md\"", analysis_id),
        )
        .body(markdown_content.into())
        .map_err(|e: axum::http::Error| { tracing::error!("HTTP response error: {}", e); AppError::internal(format!("Failed to build response: {}", e)) })
}

// PDF Export
pub async fn export_pdf_report(
    State(state): State<AppState>,
    Path((project_id, analysis_id)): Path<(String, String)>,
) -> Result<Response, AppError> {
    tracing::info!("Starting PDF export for analysis {} in project {}", analysis_id, project_id);
    
    let analysis = get_analysis_with_related_data(state, &project_id, &analysis_id).await
        .map_err(|e| {
            tracing::error!("Failed to fetch analysis data for PDF export: {}", e);
            e
        })?;

    match generate_pdf_report(&analysis, &analysis_id).await {
        Ok(pdf_data) => {
            tracing::info!("Successfully generated PDF export for analysis {} (size: {} bytes)", analysis_id, pdf_data.len());
            Ok(Response::builder()
                .header(header::CONTENT_TYPE, "application/pdf")
                .header(
                    header::CONTENT_DISPOSITION,
                    format!("attachment; filename=\"loglens_analysis_{}.pdf\"", analysis_id),
                )
                .body(pdf_data.into())
                .map_err(|e: axum::http::Error| { 
                    tracing::error!("HTTP response error for PDF: {}", e); 
                    AppError::internal(format!("Failed to build PDF response: {}", e)) 
                })?)
        }
        Err(e) => {
            tracing::error!("Failed to generate PDF for analysis {}: {}", analysis_id, e);
            // Return a more informative error response instead of fallback HTML
            let error_html = format!(
                r#"<!DOCTYPE html>
                <html>
                <head>
                    <title>PDF Export Error - LogLens</title>
                    <style>
                        body {{ font-family: Arial, sans-serif; margin: 40px; background: #f5f5f5; }}
                        .container {{ max-width: 600px; margin: 0 auto; background: white; padding: 30px; border-radius: 8px; box-shadow: 0 2px 10px rgba(0,0,0,0.1); }}
                        h1 {{ color: #e74c3c; }}
                        .error-details {{ background: #f8d7da; padding: 15px; border-radius: 5px; margin: 20px 0; }}
                        .suggestion {{ background: #d1ecf1; padding: 15px; border-radius: 5px; margin: 20px 0; }}
                    </style>
                </head>
                <body>
                    <div class="container">
                        <h1>PDF Export Failed</h1>
                        <p>We encountered an error while generating the PDF report for analysis <strong>{}</strong>.</p>
                        
                        <div class="error-details">
                            <strong>Error Details:</strong><br>
                            {}
                        </div>
                        
                        <div class="suggestion">
                            <strong>Suggested Solutions:</strong><br>
                            1. Try exporting in <strong>HTML format</strong> instead (contains the same content)<br>
                            2. Check if the analysis data is complete<br>
                            3. Contact support if the issue persists
                        </div>
                        
                        <p><em>You can close this window and try a different export format.</em></p>
                    </div>
                </body>
                </html>"#,
                analysis_id, e
            );
            Ok(Response::builder()
                .header(header::CONTENT_TYPE, "text/html; charset=utf-8")
                .status(axum::http::StatusCode::INTERNAL_SERVER_ERROR)
                .body(error_html.into())
                .map_err(|e: axum::http::Error| { 
                    tracing::error!("Failed to build error response: {}", e); 
                    AppError::internal(format!("Failed to build error response: {}", e)) 
                })?)
        }
    }
}

// Helper function to generate CSV report
fn generate_csv_report(analysis: &serde_json::Value, analysis_id: &str) -> String {
    let mut csv_data = String::new();

    // CSV Header
    csv_data.push_str("type,analysis_id,field,value,timestamp\n");

    // Analysis metadata
    let analysis_obj = analysis
        .get("analysis")
        .and_then(|a| a.as_object())
        .cloned()
        .unwrap_or_default();

    // Add metadata rows
    csv_data.push_str(&format!(
        "metadata,{},provider,\"{}\",{}\n",
        analysis_id,
        analysis_obj
            .get("provider")
            .and_then(|p| p.as_str())
            .unwrap_or("unknown"),
        analysis_obj
            .get("started_at")
            .and_then(|s| s.as_str())
            .unwrap_or("unknown")
    ));

    csv_data.push_str(&format!(
        "metadata,{},level_filter,\"{}\",{}\n",
        analysis_id,
        analysis_obj
            .get("level_filter")
            .and_then(|l| l.as_str())
            .unwrap_or("unknown"),
        analysis_obj
            .get("started_at")
            .and_then(|s| s.as_str())
            .unwrap_or("unknown")
    ));

    csv_data.push_str(&format!(
        "metadata,{},status,\"{}\",{}\n",
        analysis_id,
        analysis_obj
            .get("status")
            .and_then(|s| s.as_str())
            .unwrap_or("unknown"),
        analysis_obj
            .get("started_at")
            .and_then(|s| s.as_str())
            .unwrap_or("unknown")
    ));

    // Performance metrics
    if let Some(metrics) = analysis
        .get("performance_metrics")
        .and_then(|m| m.as_array())
    {
        for metric in metrics {
            if let Some(metric_obj) = metric.as_object() {
                csv_data.push_str(&format!(
                    "metric,{},\"{}\",{},\"{}\",{}\n",
                    analysis_id,
                    metric_obj
                        .get("metric_name")
                        .and_then(|n| n.as_str())
                        .unwrap_or("unknown"),
                    metric_obj
                        .get("metric_value")
                        .and_then(|v| v.as_f64())
                        .unwrap_or(0.0),
                    metric_obj
                        .get("unit")
                        .and_then(|u| u.as_str())
                        .unwrap_or("unknown"),
                    metric_obj
                        .get("created_at")
                        .and_then(|c| c.as_str())
                        .unwrap_or("unknown")
                ));
            }
        }
    }

    // Correlations
    if let Some(correlations) = analysis.get("correlations").and_then(|c| c.as_array()) {
        for correlation in correlations {
            if let Some(corr_obj) = correlation.as_object() {
                csv_data.push_str(&format!(
                    "correlation,{},\"{}\",{},\"{}\",{}\n",
                    analysis_id,
                    corr_obj
                        .get("correlated_error_id")
                        .and_then(|e| e.as_str())
                        .unwrap_or("unknown"),
                    corr_obj
                        .get("correlation_strength")
                        .and_then(|s| s.as_f64())
                        .unwrap_or(0.0),
                    corr_obj
                        .get("correlation_type")
                        .and_then(|t| t.as_str())
                        .unwrap_or("unknown"),
                    corr_obj
                        .get("created_at")
                        .and_then(|c| c.as_str())
                        .unwrap_or("unknown")
                ));
            }
        }
    }

    csv_data
}

// Helper function to generate Markdown report
fn generate_markdown_report(analysis: &serde_json::Value, analysis_id: &str) -> String {
    let mut markdown = String::new();

    let analysis_obj = analysis
        .get("analysis")
        .and_then(|a| a.as_object())
        .cloned()
        .unwrap_or_default();

    // Title and metadata
    markdown.push_str("# LogLens Analysis Report\n\n");
    markdown.push_str(&format!("**Analysis ID:** {}\n\n", analysis_id));

    markdown.push_str("## Summary\n\n");
    markdown.push_str(&format!(
        "- **Provider:** {}\n",
        analysis_obj
            .get("provider")
            .and_then(|p| p.as_str())
            .unwrap_or("unknown")
    ));
    markdown.push_str(&format!(
        "- **Level Filter:** {}\n",
        analysis_obj
            .get("level_filter")
            .and_then(|l| l.as_str())
            .unwrap_or("unknown")
    ));
    markdown.push_str(&format!(
        "- **Status:** {}\n",
        analysis_obj
            .get("status")
            .and_then(|s| s.as_str())
            .unwrap_or("unknown")
    ));
    markdown.push_str(&format!(
        "- **Started:** {}\n",
        analysis_obj
            .get("started_at")
            .and_then(|s| s.as_str())
            .unwrap_or("unknown")
    ));
    if let Some(completed) = analysis_obj.get("completed_at").and_then(|c| c.as_str()) {
        markdown.push_str(&format!("- **Completed:** {}\n", completed));
    }

    // Performance metrics
    if let Some(metrics) = analysis
        .get("performance_metrics")
        .and_then(|m| m.as_array())
    {
        if !metrics.is_empty() {
            markdown.push_str("\n## Performance Metrics\n\n");
            markdown.push_str("| Metric | Value | Unit | Status |\n");
            markdown.push_str("|--------|-------|------|--------|\n");

            for metric in metrics {
                if let Some(metric_obj) = metric.as_object() {
                    let is_bottleneck = metric_obj
                        .get("is_bottleneck")
                        .and_then(|b| b.as_bool())
                        .unwrap_or(false);
                    markdown.push_str(&format!(
                        "| {} | {} | {} | {} |\n",
                        metric_obj
                            .get("metric_name")
                            .and_then(|n| n.as_str())
                            .unwrap_or("unknown"),
                        metric_obj
                            .get("metric_value")
                            .and_then(|v| v.as_f64())
                            .unwrap_or(0.0),
                        metric_obj
                            .get("unit")
                            .and_then(|u| u.as_str())
                            .unwrap_or("unknown"),
                        if is_bottleneck { "BOTTLENECK" } else { "OK" }
                    ));
                }
            }
        }
    }

    // Correlations
    if let Some(correlations) = analysis.get("correlations").and_then(|c| c.as_array()) {
        if !correlations.is_empty() {
            markdown.push_str("\n## Error Correlations\n\n");

            for correlation in correlations {
                if let Some(corr_obj) = correlation.as_object() {
                    markdown.push_str(&format!(
                        "- **Correlated with:** {}\n",
                        corr_obj
                            .get("correlated_error_id")
                            .and_then(|e| e.as_str())
                            .unwrap_or("unknown")
                    ));
                    markdown.push_str(&format!(
                        "  - **Strength:** {:.2}\n",
                        corr_obj
                            .get("correlation_strength")
                            .and_then(|s| s.as_f64())
                            .unwrap_or(0.0)
                    ));
                    markdown.push_str(&format!(
                        "  - **Type:** {}\n",
                        corr_obj
                            .get("correlation_type")
                            .and_then(|t| t.as_str())
                            .unwrap_or("unknown")
                    ));
                    markdown.push('\n');
                }
            }
        }
    }

    // Analysis results
    if let Some(result) = analysis_obj.get("result").and_then(|r| r.as_str()) {
        markdown.push_str("\n## Analysis Results\n\n");
        markdown.push_str("```\n");
        markdown.push_str(result);
        markdown.push_str("\n```\n");
    }

    markdown.push_str(&format!(
        "\n---\n*Generated by LogLens at {}*\n",
        chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC")
    ));

    markdown
}

// Helper function to generate PDF report
async fn generate_pdf_report(analysis: &serde_json::Value, analysis_id: &str) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
    tracing::info!("Starting PDF generation for analysis {}", analysis_id);
    
    // Check if wkhtmltopdf is available
    match Command::new("wkhtmltopdf").arg("--version").output() {
        Ok(_) => tracing::info!("wkhtmltopdf is available"),
        Err(e) => {
            tracing::error!("wkhtmltopdf not found: {}", e);
            return Err(format!("wkhtmltopdf binary not found: {}. Please install wkhtmltopdf.", e).into());
        }
    }

    // First generate HTML content
    let html_content = generate_html_report(analysis, true);
    
    if html_content.is_empty() {
        return Err("Failed to generate HTML content for PDF conversion".into());
    }

    // Create a temporary HTML file
    let mut temp_file = NamedTempFile::with_suffix(".html")?;
    temp_file.write_all(html_content.as_bytes())?;
    let temp_path = temp_file.path();
    
    tracing::debug!("Created temporary HTML file at: {:?}", temp_path);

    // Use wkhtmltopdf to convert HTML to PDF
    let output = Command::new("wkhtmltopdf")
        .arg("--page-size")
        .arg("A4")
        .arg("--orientation")
        .arg("Portrait")
        .arg("--margin-top")
        .arg("20mm")
        .arg("--margin-bottom")
        .arg("20mm")
        .arg("--margin-left")
        .arg("15mm")
        .arg("--margin-right")
        .arg("15mm")
        .arg("--enable-local-file-access")
        .arg("--encoding")
        .arg("UTF-8")
        .arg("--no-stop-slow-scripts")
        .arg(temp_path)
        .arg("-")
        .output()?;

    if output.status.success() {
        let pdf_size = output.stdout.len();
        tracing::info!("Successfully generated PDF for analysis {} (size: {} bytes)", analysis_id, pdf_size);
        Ok(output.stdout)
    } else {
        let error_msg = String::from_utf8_lossy(&output.stderr);
        let stdout_msg = String::from_utf8_lossy(&output.stdout);
        tracing::error!("wkhtmltopdf failed with status {}: stderr: {}, stdout: {}", 
                      output.status, error_msg, stdout_msg);
        Err(format!("wkhtmltopdf failed (exit code: {}): {}", 
                   output.status.code().unwrap_or(-1), error_msg).into())
    }
}

// Shareable Analysis Links
pub async fn create_share_link(
    State(state): State<AppState>,
    Path(project_id): Path<String>,
    Json(req): Json<ShareRequest>,
) -> Result<Json<ShareResponse>, AppError> {
    // Verify analysis exists and belongs to project
    let _analysis = sqlx::query_as::<_, Analysis>(
        "SELECT id, project_id, log_file_id, analysis_type, provider, level_filter, status, result, error_message, started_at, completed_at
         FROM analyses WHERE id = ? AND project_id = ?"
    )
    .bind(&req.analysis_id)
    .bind(&project_id)
    .fetch_optional(state.db.pool())
    .await
    .map_err(|e: sqlx::Error| { tracing::error!("Database error: {}", e); AppError::Database(e) })?
    .ok_or_else(|| AppError::not_found("Resource not found"))?;

    let share_id = uuid::Uuid::new_v4().to_string();
    let expires_in_hours = req.expires_in_hours.unwrap_or(24);
    let expires_at = chrono::Utc::now() + chrono::Duration::hours(expires_in_hours);

    // Store share information (in a real implementation, this would go to a shares table)
    // For now, we'll create a knowledge base entry as a placeholder
    let solution_data = serde_json::json!({
        "analysis_id": req.analysis_id,
        "expires_at": expires_at,
        "password_protected": req.password_protect.unwrap_or(false),
        "allow_download": req.allow_download.unwrap_or(true)
    });

    let solution_json = serde_json::to_string(&solution_data).unwrap_or_default();
    let title = format!("Share Link for Analysis {}", req.analysis_id);
    let tags_json = serde_json::to_string(&vec!["share", "analysis"]).unwrap_or_default();

    sqlx::query(
        "INSERT INTO knowledge_base (id, project_id, title, problem_description, solution, tags, severity, is_public, usage_count, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)"
    )
    .bind(&share_id)
    .bind(&project_id)
    .bind(&title)
    .bind(format!("Shared analysis: {}", req.analysis_id))
    .bind(&solution_json)
    .bind(&tags_json)
    .bind("low")
    .bind(true)
    .bind(0)
    .bind(chrono::Utc::now())
    .bind(chrono::Utc::now())
    .execute(state.db.pool())
    .await
    .map_err(|e: sqlx::Error| { tracing::error!("Database error: {}", e); AppError::Database(e) })?;

    let response = ShareResponse {
        share_id: share_id.clone(),
        share_url: format!("http://localhost:3000/shared/{}", share_id),
        expires_at: expires_at.to_rfc3339(),
        is_password_protected: req.password_protect.unwrap_or(false),
        created_at: chrono::Utc::now().to_rfc3339(),
    };

    Ok(Json(response))
}

// Get shared analysis
pub async fn get_shared_analysis(
    State(state): State<AppState>,
    Path(share_id): Path<String>,
) -> Result<Response, AppError> {
    // Get share information from knowledge base (placeholder implementation)
    let share_info = sqlx::query_as::<_, KnowledgeBaseEntry>(
        "SELECT id, project_id, title, problem_description, solution, tags, severity, is_public, usage_count, created_at, updated_at
         FROM knowledge_base WHERE id = $1 AND tags LIKE '%\"share\"%'"
    )
    .bind(&share_id)
    .fetch_optional(state.db.pool())
    .await
    .map_err(|e: sqlx::Error| { tracing::error!("Database error: {}", e); AppError::Database(e) })?
    .ok_or_else(|| AppError::not_found("Resource not found"))?;

    // Parse share data
    let share_data: serde_json::Value = serde_json::from_str(&share_info.solution)
        .map_err(|e: serde_json::Error| { tracing::error!("JSON serialization error: {}", e); AppError::internal(format!("Failed to serialize JSON: {}", e)) })?;

    let analysis_id = share_data
        .get("analysis_id")
        .and_then(|id| id.as_str())
        .ok_or_else(|| AppError::not_found("Resource not found"))?;

    let project_id = share_info.project_id.clone();

    // Get analysis data
    let analysis = get_analysis_with_related_data(state, &project_id, analysis_id).await?;

    // Generate HTML view for shared analysis
    let html_content = generate_shared_html_view(&analysis, &share_info);

    Response::builder()
        .header(header::CONTENT_TYPE, "text/html; charset=utf-8")
        .body(html_content.into())
        .map_err(|e: axum::http::Error| { tracing::error!("HTTP response error: {}", e); AppError::internal(format!("Failed to build response: {}", e)) })
}

// Export history
pub async fn get_export_history(
    State(_state): State<AppState>,
    Path(_project_id): Path<String>,
) -> Result<Json<Vec<ExportMetadata>>, AppError> {
    // This would query an exports table in a real implementation
    // For now, return empty vector as placeholder
    Ok(Json(Vec::new()))
}

// Helper functions
async fn get_analysis_with_related_data(
    state: AppState,
    project_id: &str,
    analysis_id: &str,
) -> Result<serde_json::Value, AppError> {
    let _analysis = sqlx::query_as::<_, Analysis>(
        "SELECT id, project_id, log_file_id, analysis_type, provider, level_filter, status, result, error_message, started_at, completed_at
         FROM analyses WHERE id = ? AND project_id = ?"
    )
    .bind(analysis_id)
    .bind(project_id)
    .fetch_optional(state.db.pool())
    .await
    .map_err(|e: sqlx::Error| { tracing::error!("Database error: {}", e); AppError::Database(e) })?
    .ok_or_else(|| AppError::not_found("Resource not found"))?;

    let correlations = sqlx::query_as::<_, ErrorCorrelation>(
        "SELECT id, project_id, primary_error_id, correlated_error_id, correlation_strength, correlation_type, created_at
         FROM error_correlations 
         WHERE project_id = ? AND (primary_error_id = ? OR correlated_error_id = ?)
         ORDER BY correlation_strength DESC"
    )
    .bind(project_id)
    .bind(analysis_id)
    .bind(analysis_id)
    .fetch_all(state.db.pool())
    .await
    .map_err(|e: sqlx::Error| { tracing::error!("Database error: {}", e); AppError::Database(e) })?;

    let metrics = sqlx::query_as::<_, PerformanceMetric>(
        "SELECT id, analysis_id, metric_name, metric_value, unit, threshold_value, is_bottleneck, created_at
         FROM performance_metrics WHERE analysis_id = ?"
    )
    .bind(analysis_id)
    .fetch_all(state.db.pool())
    .await
    .map_err(|e: sqlx::Error| { tracing::error!("Database error: {}", e); AppError::Database(e) })?;

    let analysis_json = serde_json::json!({
        "analysis": _analysis,
        "correlations": correlations,
        "performance_metrics": metrics,
        "exported_at": chrono::Utc::now()
    });

    Ok(analysis_json)
}

fn generate_html_report(analysis: &serde_json::Value, _include_charts: bool) -> String {
    let mut html = String::new();

    let analysis_obj = analysis
        .get("analysis")
        .and_then(|a| a.as_object())
        .cloned()
        .unwrap_or_default();

    html.push_str(&format!(
        r#"
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Analysis Report - {}</title>
    <style>
        body {{ font-family: 'Segoe UI', Tahoma, Geneva, Verdana, sans-serif; margin: 0; padding: 20px; background: #f5f5f5; }}
        .container {{ max-width: 1200px; margin: 0 auto; background: white; padding: 30px; border-radius: 10px; box-shadow: 0 2px 10px rgba(0,0,0,0.1); }}
        h1 {{ color: #2c3e50; border-bottom: 3px solid #3498db; padding-bottom: 10px; }}
        h2 {{ color: #34495e; margin-top: 30px; }}
        .summary {{ background: #ecf0f1; padding: 20px; border-radius: 8px; margin: 20px 0; }}
        .metric {{ display: inline-block; margin: 10px; padding: 15px; background: #3498db; color: white; border-radius: 5px; }}
        .correlation {{ background: #fff3cd; border: 1px solid #ffeaa7; padding: 15px; margin: 10px 0; border-radius: 5px; }}
        .error {{ background: #f8d7da; border: 1px solid #f5c6cb; padding: 15px; margin: 10px 0; border-radius: 5px; }}
        .success {{ background: #d4edda; border: 1px solid #c3e6cb; padding: 15px; margin: 10px 0; border-radius: 5px; }}
        table {{ width: 100%; border-collapse: collapse; margin: 20px 0; }}
        th, td {{ padding: 12px; text-align: left; border-bottom: 1px solid #ddd; }}
        th {{ background-color: #3498db; color: white; }}
        .bottleneck {{ background: #e74c3c; color: white; }}
    </style>
</head>
<body>
    <div class="container">
        <h1>Analysis Report</h1>
        
        <div class="summary">
            <h2>Summary</h2>
            <p><strong>Analysis ID:</strong> {}</p>
            <p><strong>Provider:</strong> {}</p>
            <p><strong>Level Filter:</strong> {}</p>
            <p><strong>Status:</strong> {}</p>
            <p><strong>Started:</strong> {}</p>
            <p><strong>Completed:</strong> {}</p>
        </div>
"#,
        analysis_obj.get("id").and_then(|i| i.as_str()).unwrap_or("unknown"), // title
        analysis_obj.get("id").and_then(|i| i.as_str()).unwrap_or("unknown"),
        analysis_obj.get("provider").and_then(|p| p.as_str()).unwrap_or("unknown"),
        analysis_obj.get("level_filter").and_then(|l| l.as_str()).unwrap_or("unknown"),
        analysis_obj.get("status").and_then(|s| s.as_str()).unwrap_or("unknown"),
        analysis_obj.get("started_at").and_then(|s| s.as_str()).unwrap_or("unknown"),
        analysis_obj.get("completed_at").and_then(|c| c.as_str()).unwrap_or("N/A")
    ));

    // Performance metrics
    if let Some(metrics) = analysis
        .get("performance_metrics")
        .and_then(|m| m.as_array())
    {
        if !metrics.is_empty() {
            html.push_str("<h2>Performance Metrics</h2>");
            html.push_str("<table><tr><th>Metric</th><th>Value</th><th>Unit</th><th>Threshold</th><th>Status</th></tr>");

            for metric in metrics {
                if let Some(metric_obj) = metric.as_object() {
                    let is_bottleneck = metric_obj
                        .get("is_bottleneck")
                        .and_then(|b| b.as_bool())
                        .unwrap_or(false);
                    let status_class = if is_bottleneck {
                        "bottleneck"
                    } else {
                        "success"
                    };
                    html.push_str(&format!(
                        "<tr><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td class='{}'>{}</td></tr>",
                        metric_obj.get("metric_name").and_then(|n| n.as_str()).unwrap_or("unknown"),
                        metric_obj.get("metric_value").and_then(|v| v.as_f64()).unwrap_or(0.0),
                        metric_obj.get("unit").and_then(|u| u.as_str()).unwrap_or("unknown"),
                        metric_obj.get("threshold_value").map_or("N/A".to_string(), |v| v.to_string()),
                        status_class,
                        if is_bottleneck { "BOTTLENECK" } else { "OK" }
                    ));
                }
            }
            html.push_str("</table>");
        }
    }

    // Correlations
    if let Some(correlations) = analysis.get("correlations").and_then(|c| c.as_array()) {
        if !correlations.is_empty() {
            html.push_str("<h2>Error Correlations</h2>");

            for correlation in correlations {
                if let Some(corr_obj) = correlation.as_object() {
                    html.push_str(&format!(
                        "<div class='correlation'>
                            <strong>Correlated with:</strong> {}<br>
                            <strong>Strength:</strong> {:.2}<br>
                            <strong>Type:</strong> {}<br>
                            <strong>Detected:</strong> {}
                        </div>",
                        corr_obj
                            .get("correlated_error_id")
                            .and_then(|e| e.as_str())
                            .unwrap_or("unknown"),
                        corr_obj
                            .get("correlation_strength")
                            .and_then(|s| s.as_f64())
                            .unwrap_or(0.0),
                        corr_obj
                            .get("correlation_type")
                            .and_then(|t| t.as_str())
                            .unwrap_or("unknown"),
                        corr_obj
                            .get("created_at")
                            .and_then(|c| c.as_str())
                            .unwrap_or("unknown")
                    ));
                }
            }
        }
    }

    // Analysis result
    if let Some(result) = analysis_obj.get("result").and_then(|r| r.as_str()) {
        html.push_str("<h2>Analysis Results</h2>");
        html.push_str(&format!("<div class='error'><pre>{}</pre></div>", result));
    }

    // Error message
    if let Some(error) = analysis_obj.get("error_message").and_then(|e| e.as_str()) {
        html.push_str("<h2>Error Information</h2>");
        html.push_str(&format!(
            "<div class='error'><strong>Error:</strong> {}</div>",
            error
        ));
    }

    html.push_str("</div></body></html>");
    html
}

fn generate_shared_html_view(
    analysis: &serde_json::Value,
    share_info: &KnowledgeBaseEntry,
) -> String {
    // Simplified shared view - in real implementation, would be more polished
    format!(
        r#"
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Shared Analysis - LogLens</title>
    <style>
        body {{ font-family: 'Segoe UI', Tahoma, Geneva, Verdana, sans-serif; margin: 0; padding: 20px; background: #f5f5f5; }}
        .container {{ max-width: 1200px; margin: 0 auto; background: white; padding: 30px; border-radius: 10px; box-shadow: 0 2px 10px rgba(0,0,0,0.1); }}
        h1 {{ color: #2c3e50; border-bottom: 3px solid #3498db; padding-bottom: 10px; }}
        .shared-info {{ background: #e8f4fd; padding: 15px; border-radius: 5px; margin-bottom: 20px; }}
        pre {{ background: #f8f9fa; padding: 15px; border-radius: 5px; overflow-x: auto; }}
    </style>
</head>
<body>
    <div class="container">
        <h1>Shared Analysis</h1>
        <div class="shared-info">
            <p><strong>Shared by:</strong> LogLens</p>
            <p><strong>Created:</strong> {}</p>
            <p><strong>Access:</strong> Public</p>
        </div>
        <h2>Analysis Data</h2>
        <pre>{}</pre>
    </div>
</body>
</html>
"#,
        share_info.created_at,
        serde_json::to_string_pretty(analysis).unwrap_or_default()
    )
}
