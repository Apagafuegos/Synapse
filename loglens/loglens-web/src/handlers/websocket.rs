use axum::extract::ws::{Message, WebSocket};
use axum::{
    extract::{Path, Query, State, WebSocketUpgrade},
    http::StatusCode,
    response::Response,
};
use futures::{sink::SinkExt, stream::StreamExt};
use loglens_core::{
    analyzer::Analyzer, create_provider, filter_logs_by_level, parse_log_lines, slim_logs,
    AnalysisResponse,
};
use serde::{Deserialize, Serialize};
use std::{sync::Arc, time::Instant};
use tokio::sync::broadcast;
use tokio::sync::RwLock;
use uuid::Uuid;
use serde_json::json;
use tokio::time::Duration;

use crate::{models::*, AppState};

/// WebSocket handler for real-time log analysis
/// Provides live progress updates, cancellation support, and streaming results
pub async fn websocket_analysis_handler(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
    Path((project_id, file_id)): Path<(String, String)>,
    Query(params): Query<AnalysisWebSocketParams>,
) -> Result<Response, StatusCode> {
    // Validate project and file exist
    let _project = sqlx::query_as::<_, Project>(
        "SELECT id, name, description, created_at, updated_at FROM projects WHERE id = ?",
    )
    .bind(&project_id)
    .fetch_one(state.db.pool())
    .await
    .map_err(|_| StatusCode::NOT_FOUND)?;

    let log_file = sqlx::query_as::<_, LogFile>(
        "SELECT id, project_id, filename, file_size, created_at FROM log_files WHERE id = ? AND project_id = ?",
    )
    .bind(&file_id)
    .bind(&project_id)
    .fetch_one(state.db.pool())
    .await
    .map_err(|_| StatusCode::NOT_FOUND)?;

    Ok(ws.on_upgrade(move |socket| websocket_analysis_task(socket, state, log_file, params)))
}

#[derive(Debug, Deserialize)]
pub struct AnalysisWebSocketParams {
    pub level: String,
    pub provider: String,
    pub api_key: Option<String>,
    pub user_context: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(tag = "type", content = "data")]
pub enum WebSocketMessage {
    /// Progress update with current status
    Progress {
        stage: String,
        progress: f32, // 0.0 to 1.0
        message: String,
        elapsed_ms: u64,
    },
    /// Error occurred during analysis
    Error {
        error: String,
        stage: String,
        elapsed_ms: u64,
    },
    /// Analysis completed successfully
    Complete {
        analysis: AnalysisResponse,
        analysis_id: String,
        elapsed_ms: u64,
        stats: AnalysisStats,
    },
    /// Analysis was cancelled
    Cancelled { reason: String, elapsed_ms: u64 },
    /// Heartbeat to keep connection alive
    Heartbeat { timestamp: u64 },
}

#[derive(Debug, Serialize)]
pub struct AnalysisStats {
    pub total_lines: usize,
    pub parsed_entries: usize,
    pub filtered_entries: usize,
    pub slimmed_entries: usize,
    pub processing_time_ms: u64,
    pub ai_analysis_time_ms: u64,
}

async fn websocket_analysis_task(
    socket: WebSocket,
    state: AppState,
    log_file: LogFile,
    params: AnalysisWebSocketParams,
) {
    let start_time = Instant::now();
    let (mut sender, mut receiver) = socket.split();
    let analysis_id = Uuid::new_v4().to_string();

    // Send initial progress
    let _ = send_progress_split(
        &mut sender,
        "starting",
        0.0,
        "Initializing analysis",
        start_time.elapsed().as_millis() as u64,
    )
    .await;

    // Create cancellation channel
    let (cancel_tx, mut cancel_rx) = broadcast::channel(1);

    // Handle incoming messages for cancellation
    let cancel_tx_clone = cancel_tx.clone();
    tokio::spawn(async move {
        while let Some(msg) = receiver.next().await {
            match msg {
                Ok(Message::Text(text)) => {
                    if text == "cancel" {
                        let _ = cancel_tx_clone.send("user_requested".to_string());
                        break;
                    }
                }
                Ok(Message::Close(_)) => {
                    let _ = cancel_tx_clone.send("connection_closed".to_string());
                    break;
                }
                _ => {}
            }
        }
    });

    // Start analysis
    let analysis_result = perform_streaming_analysis_split(
        &state,
        &log_file,
        &params,
        &analysis_id,
        start_time,
        &mut sender,
        &mut cancel_rx,
    )
    .await;

    match analysis_result {
        Ok(analysis_data) => {
            // Store analysis result in database first
            let _ = store_analysis_result(&state, &log_file, &analysis_id, &analysis_data).await;

            let complete_msg = WebSocketMessage::Complete {
                analysis: analysis_data.analysis,
                analysis_id: analysis_id.clone(),
                elapsed_ms: start_time.elapsed().as_millis() as u64,
                stats: analysis_data.stats,
            };

            let _ = sender
                .send(Message::Text(serde_json::to_string(&complete_msg).unwrap()))
                .await;
        }
        Err(error) => {
            let error_msg = WebSocketMessage::Error {
                error: error.to_string(),
                stage: "analysis".to_string(),
                elapsed_ms: start_time.elapsed().as_millis() as u64,
            };

            let _ = sender
                .send(Message::Text(serde_json::to_string(&error_msg).unwrap()))
                .await;
        }
    }

    let _ = sender.send(Message::Close(None)).await;
}

struct AnalysisData {
    analysis: AnalysisResponse,
    stats: AnalysisStats,
}

async fn perform_streaming_analysis_split(
    state: &AppState,
    log_file: &LogFile,
    params: &AnalysisWebSocketParams,
    analysis_id: &str,
    start_time: Instant,
    sender: &mut futures::stream::SplitSink<WebSocket, Message>,
    cancel_rx: &mut broadcast::Receiver<String>,
) -> Result<AnalysisData, anyhow::Error> {
    // Check for cancellation
    if cancel_rx.try_recv().is_ok() {
        send_cancelled_split(sender, "cancelled_before_start".to_string(), start_time).await;
        return Err(anyhow::anyhow!("Analysis cancelled"));
    }

    // Step 1: Read log file
    {
        let _ = send_progress_split(
            sender,
            "reading_file",
            0.1,
            &format!("Reading log file: {}", log_file.filename),
            start_time.elapsed().as_millis() as u64,
        )
        .await;
    }

    let file_path = &log_file.upload_path;
    let raw_lines = match tokio::fs::read_to_string(&file_path).await {
        Ok(content) => content.lines().map(String::from).collect::<Vec<_>>(),
        Err(_) => {
            // Try reading from database if file not found
            sqlx::query_scalar::<_, String>(
                "SELECT content FROM log_file_content WHERE file_id = ?",
            )
            .bind(&log_file.id)
            .fetch_one(state.db.pool())
            .await
            .map_err(|e| anyhow::anyhow!("Failed to read log file: {}", e))?
            .lines()
            .map(String::from)
            .collect()
        }
    };

    // Check for cancellation
    if cancel_rx.try_recv().is_ok() {
        send_cancelled_split(sender, "cancelled_after_reading".to_string(), start_time).await;
        return Err(anyhow::anyhow!("Analysis cancelled"));
    }

    // Step 2: Parse logs
    {
        let _ = send_progress_split(
            sender,
            "parsing",
            0.2,
            &format!("Parsing {} log lines", raw_lines.len()),
            start_time.elapsed().as_millis() as u64,
        )
        .await;
    }

    let parsed_entries = parse_log_lines(&raw_lines);

    // Step 3: Filter logs
    {
        let _ = send_progress_split(
            sender,
            "filtering",
            0.3,
            &format!("Filtering by {} level", params.level),
            start_time.elapsed().as_millis() as u64,
        )
        .await;
    }

    let filtered_entries = filter_logs_by_level(parsed_entries.clone(), &params.level)?;

    if filtered_entries.is_empty() {
        return Err(anyhow::anyhow!(
            "No log entries found matching level: {}",
            params.level
        ));
    }

    // Check for cancellation
    if cancel_rx.try_recv().is_ok() {
        send_cancelled_split(sender, "cancelled_after_filtering".to_string(), start_time).await;
        return Err(anyhow::anyhow!("Analysis cancelled"));
    }

    // Step 4: Slim logs
    {
        let _ = send_progress_split(
            sender,
            "slimming",
            0.4,
            &format!(
                "Optimizing {} entries for AI analysis",
                filtered_entries.len()
            ),
            start_time.elapsed().as_millis() as u64,
        )
        .await;
    }

    let slimmed_entries = slim_logs(filtered_entries.clone());

    // Step 5: AI Analysis
    {
        let _ = send_progress_split(
            sender,
            "ai_analysis",
            0.5,
            &format!("Analyzing with {} provider", params.provider),
            start_time.elapsed().as_millis() as u64,
        )
        .await;
    }

    let ai_start_time = Instant::now();

    // Get API key
    let api_key = match &params.api_key {
        Some(key) => key.clone(),
        None => state
            .config
            .get_api_key(&params.provider)
            .ok_or_else(|| anyhow::anyhow!("API key required for provider {}", params.provider))?,
    };

    // Create provider and analyzer
    let provider = create_provider(&params.provider, &api_key)?;
    let mut analyzer = Analyzer::new(provider);

    // Check for cancellation before expensive AI call
    if cancel_rx.try_recv().is_ok() {
        send_cancelled_split(sender, "cancelled_before_ai".to_string(), start_time).await;
        return Err(anyhow::anyhow!("Analysis cancelled"));
    }

    // Perform AI analysis
    let analysis = analyzer.analyze_logs(slimmed_entries.clone()).await?;
    let ai_analysis_time = ai_start_time.elapsed().as_millis() as u64;

    // Step 6: Finalization
    {
        let _ = send_progress_split(
            sender,
            "finalizing",
            0.9,
            "Finalizing analysis results",
            start_time.elapsed().as_millis() as u64,
        )
        .await;
    }

    let stats = AnalysisStats {
        total_lines: raw_lines.len(),
        parsed_entries: parsed_entries.len(),
        filtered_entries: filtered_entries.len(),
        slimmed_entries: slimmed_entries.len(),
        processing_time_ms: start_time.elapsed().as_millis() as u64,
        ai_analysis_time_ms: ai_analysis_time,
    };

    Ok(AnalysisData { analysis, stats })
}

async fn send_progress(
    socket: &mut WebSocket,
    stage: &str,
    progress: f32,
    message: &str,
    elapsed_ms: u64,
) -> Result<(), anyhow::Error> {
    let progress_msg = WebSocketMessage::Progress {
        stage: stage.to_string(),
        progress,
        message: message.to_string(),
        elapsed_ms,
    };

    socket
        .send(Message::Text(serde_json::to_string(&progress_msg)?))
        .await
        .map_err(|e| anyhow::anyhow!("Failed to send progress: {}", e))?;

    Ok(())
}

async fn send_progress_split(
    sender: &mut futures::stream::SplitSink<WebSocket, Message>,
    stage: &str,
    progress: f32,
    message: &str,
    elapsed_ms: u64,
) -> Result<(), anyhow::Error> {
    let progress_msg = WebSocketMessage::Progress {
        stage: stage.to_string(),
        progress,
        message: message.to_string(),
        elapsed_ms,
    };

    sender
        .send(Message::Text(serde_json::to_string(&progress_msg)?))
        .await
        .map_err(|e| anyhow::anyhow!("Failed to send progress: {}", e))?;

    Ok(())
}

async fn send_cancelled_split(
    sender: &mut futures::stream::SplitSink<WebSocket, Message>,
    reason: String,
    start_time: Instant,
) {
    let cancel_msg = WebSocketMessage::Cancelled {
        reason,
        elapsed_ms: start_time.elapsed().as_millis() as u64,
    };

    let _ = sender
        .send(Message::Text(serde_json::to_string(&cancel_msg).unwrap()))
        .await;
}

async fn send_cancelled(socket: Arc<RwLock<WebSocket>>, reason: String, start_time: Instant) {
    let cancel_msg = WebSocketMessage::Cancelled {
        reason,
        elapsed_ms: start_time.elapsed().as_millis() as u64,
    };

    let mut socket_guard = socket.write().await;
    let _ = socket_guard
        .send(Message::Text(serde_json::to_string(&cancel_msg).unwrap()))
        .await;
}

async fn store_analysis_result(
    state: &AppState,
    log_file: &LogFile,
    analysis_id: &str,
    analysis_data: &AnalysisData,
) -> Result<(), anyhow::Error> {
    let analysis_json = serde_json::to_string(&analysis_data.analysis)?;
    let stats_json = serde_json::to_string(&analysis_data.stats)?;

    sqlx::query!(
        r#"
        INSERT INTO analyses (
            id, project_id, log_file_id,
            analysis_type, provider, level_filter,
            status, result, error_message,
            started_at, completed_at
        ) VALUES (?, ?, ?, 'file', ?, ?, 'completed', ?, NULL, datetime('now'), datetime('now'))
        "#,
        analysis_id,
        log_file.project_id,
        log_file.id,
        "loglens", // provider
        "INFO",    // level_filter
        analysis_json
    )
    .execute(state.db.pool())
    .await?;

    Ok(())
}

/// WebSocket endpoint for analysis monitoring
pub async fn websocket_monitor_handler(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
    Path(analysis_id): Path<String>,
) -> Result<Response, StatusCode> {
    // Validate analysis exists
    let _analysis = sqlx::query_as::<_, Analysis>(
        "SELECT id, project_id, log_file_id, status, created_at FROM analyses WHERE id = ?",
    )
    .bind(&analysis_id)
    .fetch_one(state.db.pool())
    .await
    .map_err(|_| StatusCode::NOT_FOUND)?;

    Ok(ws.on_upgrade(move |socket| websocket_monitor_task(socket, state, analysis_id)))
}

async fn websocket_monitor_task(mut socket: WebSocket, state: AppState, analysis_id: String) {
    let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(1));

    loop {
        tokio::select! {
            _ = interval.tick() => {
                // Check analysis status
                match check_analysis_status(&state, &analysis_id).await {
                    Ok(Some(status_msg)) => {
                        if let Err(_) = socket.send(Message::Text(status_msg)).await {
                            break;
                        }
                    }
                    Ok(None) => {
                        // Analysis completed or cancelled
                        break;
                    }
                    Err(_) => {
                        let error_msg = WebSocketMessage::Error {
                            error: "Failed to check analysis status".to_string(),
                            stage: "monitoring".to_string(),
                            elapsed_ms: 0,
                        };
                        let _ = socket.send(Message::Text(serde_json::to_string(&error_msg).unwrap())).await;
                        break;
                    }
                }

                // Send heartbeat
                let heartbeat_msg = WebSocketMessage::Heartbeat {
                    timestamp: chrono::Utc::now().timestamp_millis() as u64,
                };
                if let Err(_) = socket.send(Message::Text(serde_json::to_string(&heartbeat_msg).unwrap())).await {
                    break;
                }
            }
            msg = socket.next() => {
                match msg {
                    Some(Ok(Message::Close(_))) => break,
                    Some(Err(_)) => break,
                    _ => {}
                }
            }
        }
    }

    let _ = socket.close().await;
}

async fn check_analysis_status(
    state: &AppState,
    analysis_id: &str,
) -> Result<Option<String>, anyhow::Error> {
    let analysis = sqlx::query_as::<_, Analysis>(
        "SELECT id, project_id, log_file_id, status, created_at FROM analyses WHERE id = ?",
    )
    .bind(analysis_id)
    .fetch_optional(state.db.pool())
    .await?;

    match analysis {
        Some(analysis) => {
            if analysis.status == "completed" || analysis.status == "failed" {
                Ok(None) // Analysis is done
            } else {
                let progress_msg = WebSocketMessage::Progress {
                    stage: analysis.status.to_string(),
                    progress: 0.5, // Intermediate progress
                    message: format!("Analysis {} is {}", analysis_id, analysis.status),
                    elapsed_ms: 0,
                };
                Ok(Some(serde_json::to_string(&progress_msg)?))
            }
        }
        None => Err(anyhow::anyhow!("Analysis not found")),
    }
}

// Extension to WebConfig for API key management
impl crate::config::WebConfig {
    pub fn get_api_key(&self, provider: &str) -> Option<String> {
        match provider {
            "openrouter" => std::env::var("OPENROUTER_API_KEY").ok(),
            "openai" => std::env::var("OPENAI_API_KEY").ok(),
            "claude" => std::env::var("ANTHROPIC_API_KEY").ok(),
            "gemini" => std::env::var("GOOGLE_API_KEY").ok(),
            "mock" => Some("mock_key".to_string()),
            _ => None,
        }
    }
}

// Status WebSocket handler for system status updates
pub async fn status_ws_handler(
    ws: WebSocketUpgrade,
    State(_state): State<AppState>,
) -> Result<Response, StatusCode> {
    Ok(ws.on_upgrade(handle_status_ws))
}

async fn handle_status_ws(mut socket: WebSocket) {
    // Send initial status
    let initial_msg = serde_json::to_string(&json!({
        "type": "system_status",
        "online": true,
        "message": "Connected to LogLens server"
    })).unwrap();
    let _ = socket.send(Message::Text(initial_msg)).await;

    let mut interval = tokio::time::interval(Duration::from_secs(30));

    loop {
        tokio::select! {
            _ = interval.tick() => {
                let status_msg = serde_json::to_string(&json!({
                    "type": "system_status",
                    "online": true,
                    "message": "System operational"
                })).unwrap();
                if let Err(_) = socket.send(Message::Text(status_msg)).await {
                    break;
                }
            }
            msg = socket.recv() => {
                match msg {
                    Some(Ok(Message::Close(_))) => break,
                    Some(Err(_)) => break,
                    _ => {},
                }
            }
        }
    }
    
    let _ = socket.close().await;
}
