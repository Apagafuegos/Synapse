use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::extract::{Path, State};
use axum::response::Response;
use futures::{sink::SinkExt, stream::StreamExt};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};
use tokio::time::{Duration, Instant};
use tracing::{error, info, warn};
use uuid::Uuid;

pub mod sources;
pub mod sources_simple;


use crate::AppState;
use synapse_core::LogEntry;

/// Maximum number of log entries to buffer before forcing a batch send
const MAX_BUFFER_SIZE: usize = 100;
/// Maximum time to wait before sending a batch (in seconds)
const MAX_BUFFER_TIME: u64 = 2;
/// Channel capacity for broadcasting log entries
const BROADCAST_CAPACITY: usize = 1000;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamingLogEntry {
    pub id: String,
    pub timestamp: Option<String>,
    pub level: Option<String>,
    pub message: String,
    pub source: String,
    pub project_id: Uuid,
    pub line_number: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogBatch {
    pub batch_id: String,
    pub timestamp: String,
    pub entries: Vec<StreamingLogEntry>,
    pub source: String,
    pub project_id: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum StreamingMessage {
    #[serde(rename = "log_batch")]
    LogBatch(LogBatch),
    #[serde(rename = "heartbeat")]
    Heartbeat { timestamp: String },
    #[serde(rename = "error")]
    Error { message: String, code: Option<String> },
    #[serde(rename = "subscription_status")]
    SubscriptionStatus { subscribed: bool, project_id: Uuid },
}

#[derive(Debug, Clone)]
pub struct StreamingSource {
    pub id: String,
    pub name: String,
    pub project_id: Uuid,
    pub buffer: Vec<StreamingLogEntry>,
    pub last_flush: Instant,
    pub is_active: bool,
}

/// Central hub for managing streaming log sources and WebSocket connections
#[derive(Clone)]
pub struct StreamingHub {
    /// Broadcast channel for sending log batches to connected clients
    pub sender: broadcast::Sender<StreamingMessage>,
    /// Map of streaming sources by project ID
    pub sources: Arc<RwLock<HashMap<Uuid, Vec<StreamingSource>>>>,
    /// Active WebSocket connections by project ID
    pub connections: Arc<RwLock<HashMap<Uuid, usize>>>,
}

impl Default for StreamingHub {
    fn default() -> Self {
        Self::new()
    }
}

impl StreamingHub {
    pub fn new() -> Self {
        let (sender, _) = broadcast::channel(BROADCAST_CAPACITY);
        
        Self {
            sender,
            sources: Arc::new(RwLock::new(HashMap::new())),
            connections: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Register a new streaming source for a project
    pub async fn register_source(&self, project_id: Uuid, source_name: String) -> String {
        let source_id = Uuid::new_v4().to_string();
        let source = StreamingSource {
            id: source_id.clone(),
            name: source_name,
            project_id,
            buffer: Vec::new(),
            last_flush: Instant::now(),
            is_active: true,
        };

        let mut sources = self.sources.write().await;
        sources.entry(project_id).or_insert_with(Vec::new).push(source);
        
        info!("Registered streaming source {} for project {}", source_id, project_id);
        source_id
    }

    /// Add log entries to a streaming source's buffer
    pub async fn add_logs(&self, project_id: Uuid, source_id: &str, entries: Vec<StreamingLogEntry>) -> anyhow::Result<()> {
        let mut sources = self.sources.write().await;
        
        if let Some(project_sources) = sources.get_mut(&project_id) {
            if let Some(source) = project_sources.iter_mut().find(|s| s.id == source_id) {
                source.buffer.extend(entries);
                
                // Check if we should flush the buffer
                let should_flush = source.buffer.len() >= MAX_BUFFER_SIZE ||
                    source.last_flush.elapsed() >= Duration::from_secs(MAX_BUFFER_TIME);
                
                if should_flush {
                    self.flush_source_buffer(source).await?;
                }
            }
        }
        
        Ok(())
    }

    /// Flush a source's buffer by sending it as a batch
    async fn flush_source_buffer(&self, source: &mut StreamingSource) -> anyhow::Result<()> {
        if source.buffer.is_empty() {
            return Ok(());
        }

        let batch = LogBatch {
            batch_id: Uuid::new_v4().to_string(),
            timestamp: chrono::Utc::now().to_rfc3339(),
            entries: source.buffer.drain(..).collect(),
            source: source.name.clone(),
            project_id: source.project_id,
        };

        let message = StreamingMessage::LogBatch(batch);
        
        if let Err(e) = self.sender.send(message) {
            warn!("Failed to broadcast log batch: {}", e);
        }

        source.last_flush = Instant::now();
        Ok(())
    }

    /// Force flush all buffers for a project
    pub async fn flush_project_buffers(&self, project_id: Uuid) -> anyhow::Result<()> {
        let mut sources = self.sources.write().await;
        
        if let Some(project_sources) = sources.get_mut(&project_id) {
            for source in project_sources.iter_mut() {
                self.flush_source_buffer(source).await?;
            }
        }
        
        Ok(())
    }

    /// Remove a streaming source
    pub async fn remove_source(&self, project_id: Uuid, source_id: &str) {
        let mut sources = self.sources.write().await;
        
        if let Some(project_sources) = sources.get_mut(&project_id) {
            project_sources.retain(|s| s.id != source_id);
            if project_sources.is_empty() {
                sources.remove(&project_id);
            }
        }
        
        info!("Removed streaming source {} for project {}", source_id, project_id);
    }

    /// Get connection count for a project
    pub async fn get_connection_count(&self, project_id: Uuid) -> usize {
        let connections = self.connections.read().await;
        *connections.get(&project_id).unwrap_or(&0)
    }

    /// Increment connection count for a project
    pub async fn increment_connections(&self, project_id: Uuid) {
        let mut connections = self.connections.write().await;
        let count = connections.entry(project_id).or_insert(0);
        *count += 1;
        info!("Incremented connections for project {}: {}", project_id, *count);
    }

    /// Decrement connection count for a project
    pub async fn decrement_connections(&self, project_id: Uuid) {
        let mut connections = self.connections.write().await;
        if let Some(count) = connections.get_mut(&project_id) {
            if *count > 0 {
                *count -= 1;
                info!("Decremented connections for project {}: {}", project_id, *count);
                
                if *count == 0 {
                    connections.remove(&project_id);
                }
            }
        }
    }

    /// Send heartbeat to all connected clients
    pub async fn send_heartbeat(&self) {
        let heartbeat = StreamingMessage::Heartbeat {
            timestamp: chrono::Utc::now().to_rfc3339(),
        };
        
        if let Err(e) = self.sender.send(heartbeat) {
            warn!("Failed to send heartbeat: {}", e);
        }
    }
}

/// WebSocket handler for streaming log data
pub async fn websocket_handler(
    ws: WebSocketUpgrade,
    Path(project_id): Path<Uuid>,
    State(state): State<AppState>,
) -> Response {
    ws.on_upgrade(move |socket| handle_websocket(socket, project_id, state))
}

async fn handle_websocket(socket: WebSocket, project_id: Uuid, state: AppState) {
    info!("WebSocket connected for project {}", project_id);
    
    // Get streaming hub from state (we'll add this to AppState)
    let streaming_hub = &state.streaming_hub;
    
    // Increment connection count
    streaming_hub.increment_connections(project_id).await;
    
    let mut receiver = streaming_hub.sender.subscribe();
    let (mut sender, mut receiver_ws) = socket.split();
    
    // Send subscription confirmation
    let subscription_msg = StreamingMessage::SubscriptionStatus {
        subscribed: true,
        project_id,
    };
    
    if let Ok(msg_json) = serde_json::to_string(&subscription_msg) {
        if let Err(e) = sender.send(Message::Text(msg_json)).await {
            error!("Failed to send subscription confirmation: {}", e);
            streaming_hub.decrement_connections(project_id).await;
            return;
        }
    }

    // Handle incoming WebSocket messages and broadcast messages
    tokio::select! {
        // Handle incoming WebSocket messages (client to server)
        _ = async {
            while let Some(msg) = receiver_ws.next().await {
                match msg {
                    Ok(Message::Text(text)) => {
                        // Handle client commands (e.g., change filters, pause/resume)
                        if let Err(e) = handle_client_message(&text, project_id, &state).await {
                            warn!("Failed to handle client message: {}", e);
                        }
                    }
                    Ok(Message::Close(_)) => {
                        info!("WebSocket connection closed by client for project {}", project_id);
                        break;
                    }
                    Err(e) => {
                        error!("WebSocket error for project {}: {}", project_id, e);
                        break;
                    }
                    _ => {}
                }
            }
        } => {}
        
        // Handle broadcast messages (server to client)
        _ = async {
            while let Ok(msg) = receiver.recv().await {
                // Only send messages relevant to this project
                let should_send = match &msg {
                    StreamingMessage::LogBatch(batch) => batch.project_id == project_id,
                    StreamingMessage::Heartbeat { .. } => true,
                    StreamingMessage::Error { .. } => true,
                    StreamingMessage::SubscriptionStatus { project_id: msg_project_id, .. } => 
                        *msg_project_id == project_id,
                };
                
                if should_send {
                    if let Ok(msg_json) = serde_json::to_string(&msg) {
                        if let Err(e) = sender.send(Message::Text(msg_json)).await {
                            error!("Failed to send message to WebSocket: {}", e);
                            break;
                        }
                    }
                }
            }
        } => {}
    }
    
    // Cleanup
    streaming_hub.decrement_connections(project_id).await;
    info!("WebSocket disconnected for project {}", project_id);
}

#[derive(Debug, Deserialize)]
struct ClientMessage {
    #[serde(rename = "type")]
    message_type: String,
    #[serde(flatten)]
    data: serde_json::Value,
}

async fn handle_client_message(
    message: &str,
    project_id: Uuid,
    _state: &AppState,
) -> anyhow::Result<()> {
    let client_msg: ClientMessage = serde_json::from_str(message)?;
    
    match client_msg.message_type.as_str() {
        "ping" => {
            // Respond with pong (heartbeat)
            info!("Received ping from client for project {}", project_id);
        }
        "filter_change" => {
            // Handle filter changes (log level, keywords, etc.)
            info!("Filter change request for project {}: {:?}", project_id, client_msg.data);
        }
        "pause_stream" => {
            // Pause streaming for this client
            info!("Stream pause request for project {}", project_id);
        }
        "resume_stream" => {
            // Resume streaming for this client
            info!("Stream resume request for project {}", project_id);
        }
        _ => {
            warn!("Unknown client message type: {}", client_msg.message_type);
        }
    }
    
    Ok(())
}

/// Convert a LogEntry to StreamingLogEntry
impl From<LogEntry> for StreamingLogEntry {
    fn from(entry: LogEntry) -> Self {
        StreamingLogEntry {
            id: Uuid::new_v4().to_string(),
            timestamp: entry.timestamp,
            level: entry.level,
            message: entry.message,
            source: "unknown".to_string(), // Will be set by the source
            project_id: Uuid::nil(), // Will be set by the source
            line_number: entry.line_number,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_streaming_hub_creation() {
        let hub = StreamingHub::new();
        assert_eq!(hub.get_connection_count(Uuid::new_v4()).await, 0);
    }

    #[tokio::test]
    async fn test_source_registration() {
        let hub = StreamingHub::new();
        let project_id = Uuid::new_v4();
        let source_id = hub.register_source(project_id, "test-source".to_string()).await;
        
        let sources = hub.sources.read().await;
        assert!(sources.contains_key(&project_id));
        assert_eq!(sources.get(&project_id).unwrap().len(), 1);
        assert_eq!(sources.get(&project_id).unwrap()[0].id, source_id);
    }

    #[tokio::test]
    async fn test_connection_counting() {
        let hub = StreamingHub::new();
        let project_id = Uuid::new_v4();
        
        hub.increment_connections(project_id).await;
        assert_eq!(hub.get_connection_count(project_id).await, 1);
        
        hub.increment_connections(project_id).await;
        assert_eq!(hub.get_connection_count(project_id).await, 2);
        
        hub.decrement_connections(project_id).await;
        assert_eq!(hub.get_connection_count(project_id).await, 1);
        
        hub.decrement_connections(project_id).await;
        assert_eq!(hub.get_connection_count(project_id).await, 0);
    }
}