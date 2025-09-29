use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::fs::File;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;
use tokio::sync::mpsc;
use tokio::time::{Duration, Instant};
use tracing::{error, info, warn};
use uuid::Uuid;

use crate::streaming::{StreamingHub, StreamingLogEntry};
use loglens_core::parser::parse_single_log_line;
use regex::Regex;

/// Different types of streaming log sources
#[derive(Debug, Clone)]
pub enum StreamingSourceType {
    /// Monitor a file for changes (tail -f behavior)
    File { path: PathBuf },
    /// Execute a command and stream its output
    Command { command: String, args: Vec<String> },
    /// Listen on a TCP port for incoming log data
    TcpListener { port: u16 },
    /// Read from stdin
    Stdin,
    /// Custom HTTP endpoint for log ingestion
    HttpEndpoint { path: String },
}

/// Configuration for a streaming source
#[derive(Debug, Clone)]
pub struct StreamingSourceConfig {
    pub source_type: StreamingSourceType,
    pub project_id: Uuid,
    pub name: String,
    pub parser_config: ParserConfig,
    pub buffer_size: usize,
    pub batch_timeout: Duration,
    pub restart_on_error: bool,
    pub max_restarts: Option<u32>,
}

#[derive(Debug, Clone)]
pub struct ParserConfig {
    pub log_format: LogFormat,
    pub timestamp_format: Option<String>,
    pub level_field: Option<String>,
    pub message_field: Option<String>,
    pub metadata_fields: Vec<String>,
}

#[derive(Debug, Clone)]
pub enum LogFormat {
    /// Simple text format with basic parsing
    Text,
    /// JSON structured logs
    Json,
    /// Common log format (Apache/Nginx style)
    CommonLog,
    /// Custom regex pattern
    Regex { pattern: String },
    /// Syslog format (RFC 3164/5424)
    Syslog,
}

impl Default for ParserConfig {
    fn default() -> Self {
        Self {
            log_format: LogFormat::Text,
            timestamp_format: None,
            level_field: None,
            message_field: None,
            metadata_fields: Vec::new(),
        }
    }
}

/// Manager for streaming log sources
pub struct StreamingSourceManager {
    sources: HashMap<String, StreamingSourceHandle>,
    streaming_hub: Arc<StreamingHub>,
}

struct StreamingSourceHandle {
    source_id: String,
    config: StreamingSourceConfig,
    #[allow(dead_code)]
    cancel_tx: mpsc::UnboundedSender<()>,
    restart_count: u32,
}

impl StreamingSourceManager {
    pub fn new(streaming_hub: Arc<StreamingHub>) -> Self {
        Self {
            sources: HashMap::new(),
            streaming_hub,
        }
    }

    /// Start a new streaming source
    pub async fn start_source(&mut self, config: StreamingSourceConfig) -> anyhow::Result<String> {
        let source_id = self.streaming_hub
            .register_source(config.project_id, config.name.clone())
            .await;

        let (cancel_tx, cancel_rx) = mpsc::unbounded_channel();
        
        // Clone necessary data for the async task
        let streaming_hub = Arc::clone(&self.streaming_hub);
        let config_clone = config.clone();
        let source_id_clone = source_id.clone();

        // Start the source processing task
        tokio::spawn(async move {
            if let Err(e) = Self::run_source(streaming_hub, config_clone, source_id_clone, cancel_rx).await {
                error!("Streaming source error: {}", e);
            }
        });

        let handle = StreamingSourceHandle {
            source_id: source_id.clone(),
            config,
            cancel_tx,
            restart_count: 0,
        };

        self.sources.insert(source_id.clone(), handle);
        info!("Started streaming source: {}", source_id);
        
        Ok(source_id)
    }

    /// Stop a streaming source
    pub async fn stop_source(&mut self, source_id: &str) -> anyhow::Result<()> {
        if let Some(handle) = self.sources.remove(source_id) {
            let _ = handle.cancel_tx.send(());
            self.streaming_hub.remove_source(handle.config.project_id, source_id).await;
            info!("Stopped streaming source: {}", source_id);
        }
        Ok(())
    }

    /// Get active source count
    pub fn active_source_count(&self) -> usize {
        self.sources.len()
    }

    /// List active sources
    pub fn list_sources(&self) -> Vec<(String, String)> {
        self.sources
            .iter()
            .map(|(id, handle)| (id.clone(), handle.config.name.clone()))
            .collect()
    }

    /// Main source processing loop
    async fn run_source(
        streaming_hub: Arc<StreamingHub>,
        config: StreamingSourceConfig,
        source_id: String,
        mut cancel_rx: mpsc::UnboundedReceiver<()>,
    ) -> anyhow::Result<()> {
        let mut restart_count = 0;
        
        loop {
            // Check for cancellation
            if cancel_rx.try_recv().is_ok() {
                info!("Streaming source {} cancelled", source_id);
                break;
            }

            let result = match &config.source_type {
                StreamingSourceType::File { path } => {
                    Self::handle_file_source(&streaming_hub, &config, &source_id, path, &mut cancel_rx).await
                }
                StreamingSourceType::Command { command, args } => {
                    Self::handle_command_source(&streaming_hub, &config, &source_id, command, args, &mut cancel_rx).await
                }
                StreamingSourceType::Stdin => {
                    Self::handle_stdin_source(&streaming_hub, &config, &source_id, &mut cancel_rx).await
                }
                StreamingSourceType::TcpListener { port } => {
                    Self::handle_tcp_source(&streaming_hub, &config, &source_id, *port, &mut cancel_rx).await
                }
                StreamingSourceType::HttpEndpoint { .. } => {
                    // HTTP endpoint sources are handled by the web server directly
                    tokio::time::sleep(Duration::from_secs(1)).await;
                    continue;
                }
            };

            if let Err(e) = result {
                error!("Source {} error: {}", source_id, e);
                restart_count += 1;

                if !config.restart_on_error {
                    break;
                }

                if let Some(max_restarts) = config.max_restarts {
                    if restart_count >= max_restarts {
                        error!("Source {} exceeded max restarts ({})", source_id, max_restarts);
                        break;
                    }
                }

                // Exponential backoff for restarts
                let delay = Duration::from_secs(2_u64.pow(restart_count.min(5)));
                warn!("Restarting source {} in {:?} (attempt {})", source_id, delay, restart_count);
                tokio::time::sleep(delay).await;
            } else {
                // Source completed successfully
                break;
            }
        }

        Ok(())
    }

    /// Handle file tailing source
    async fn handle_file_source(
        streaming_hub: &Arc<StreamingHub>,
        config: &StreamingSourceConfig,
        source_id: &str,
        file_path: &PathBuf,
        cancel_rx: &mut mpsc::UnboundedReceiver<()>,
    ) -> anyhow::Result<()> {
        let file = File::open(file_path).await?;
        let mut reader = BufReader::new(file);
        let mut line = String::new();
        let mut buffer = Vec::new();
        let mut last_flush = Instant::now();

        info!("Monitoring file: {:?}", file_path);

        loop {
            tokio::select! {
                _ = cancel_rx.recv() => {
                    info!("File source cancelled");
                    break;
                }
                
                result = reader.read_line(&mut line) => {
                    match result {
                        Ok(0) => {
                            // End of file, wait a bit and continue (tail behavior)
                            tokio::time::sleep(Duration::from_millis(100)).await;
                            continue;
                        }
                        Ok(_) => {
                            if let Some(entry) = Self::parse_log_line(&line, config, source_id) {
                                buffer.push(entry);
                                line.clear();

                                // Check if we should flush the buffer
                                if buffer.len() >= config.buffer_size ||
                                   last_flush.elapsed() >= config.batch_timeout {
                                    streaming_hub.add_logs(config.project_id, source_id, buffer.drain(..).collect()).await?;
                                    last_flush = Instant::now();
                                }
                            }
                        }
                        Err(e) => return Err(e.into()),
                    }
                }
            }
        }

        // Flush remaining buffer
        if !buffer.is_empty() {
            streaming_hub.add_logs(config.project_id, source_id, buffer).await?;
        }

        Ok(())
    }

    /// Handle command output source
    async fn handle_command_source(
        streaming_hub: &Arc<StreamingHub>,
        config: &StreamingSourceConfig,
        source_id: &str,
        command: &str,
        args: &[String],
        cancel_rx: &mut mpsc::UnboundedReceiver<()>,
    ) -> anyhow::Result<()> {
        let mut child = Command::new(command)
            .args(args)
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()?;

        let stdout = child.stdout.take().unwrap();
        let mut reader = BufReader::new(stdout);
        let mut line = String::new();
        let mut buffer = Vec::new();
        let mut last_flush = Instant::now();

        info!("Started command: {} {}", command, args.join(" "));

        loop {
            tokio::select! {
                _ = cancel_rx.recv() => {
                    info!("Command source cancelled, killing process");
                    let _ = child.kill().await;
                    break;
                }
                
                result = reader.read_line(&mut line) => {
                    match result {
                        Ok(0) => {
                            // Process ended
                            if let Ok(status) = child.try_wait() {
                                if let Some(exit_status) = status {
                                    info!("Command finished with status: {}", exit_status);
                                    break;
                                }
                            }
                            tokio::time::sleep(Duration::from_millis(10)).await;
                        }
                        Ok(_) => {
                            if let Some(entry) = Self::parse_log_line(&line, config, source_id) {
                                buffer.push(entry);
                                line.clear();

                                // Check if we should flush the buffer
                                if buffer.len() >= config.buffer_size ||
                                   last_flush.elapsed() >= config.batch_timeout {
                                    streaming_hub.add_logs(config.project_id, source_id, buffer.drain(..).collect()).await?;
                                    last_flush = Instant::now();
                                }
                            }
                        }
                        Err(e) => return Err(e.into()),
                    }
                }
            }
        }

        // Flush remaining buffer
        if !buffer.is_empty() {
            streaming_hub.add_logs(config.project_id, source_id, buffer).await?;
        }

        Ok(())
    }

    /// Handle stdin source
    async fn handle_stdin_source(
        streaming_hub: &Arc<StreamingHub>,
        config: &StreamingSourceConfig,
        source_id: &str,
        cancel_rx: &mut mpsc::UnboundedReceiver<()>,
    ) -> anyhow::Result<()> {
        let stdin = tokio::io::stdin();
        let mut reader = BufReader::new(stdin);
        let mut line = String::new();
        let mut buffer = Vec::new();
        let mut last_flush = Instant::now();

        info!("Reading from stdin");

        loop {
            tokio::select! {
                _ = cancel_rx.recv() => {
                    info!("Stdin source cancelled");
                    break;
                }
                
                result = reader.read_line(&mut line) => {
                    match result {
                        Ok(0) => break, // EOF
                        Ok(_) => {
                            if let Some(entry) = Self::parse_log_line(&line, config, source_id) {
                                buffer.push(entry);
                                line.clear();

                                // Check if we should flush the buffer
                                if buffer.len() >= config.buffer_size ||
                                   last_flush.elapsed() >= config.batch_timeout {
                                    streaming_hub.add_logs(config.project_id, source_id, buffer.drain(..).collect()).await?;
                                    last_flush = Instant::now();
                                }
                            }
                        }
                        Err(e) => return Err(e.into()),
                    }
                }
            }
        }

        // Flush remaining buffer
        if !buffer.is_empty() {
            streaming_hub.add_logs(config.project_id, source_id, buffer).await?;
        }

        Ok(())
    }

    /// Handle TCP listener source
    async fn handle_tcp_source(
        streaming_hub: &Arc<StreamingHub>,
        config: &StreamingSourceConfig,
        source_id: &str,
        port: u16,
        cancel_rx: &mut mpsc::UnboundedReceiver<()>,
    ) -> anyhow::Result<()> {
        use tokio::net::TcpListener;
        
        let listener = TcpListener::bind(format!("0.0.0.0:{}", port)).await?;
        info!("TCP listener started on port {}", port);

        loop {
            tokio::select! {
                _ = cancel_rx.recv() => {
                    info!("TCP source cancelled");
                    break;
                }
                
                result = listener.accept() => {
                    match result {
                        Ok((socket, addr)) => {
                            info!("TCP connection from {}", addr);
                            let streaming_hub = Arc::clone(streaming_hub);
                            let config = config.clone();
                            let source_id = source_id.to_string();
                            
                            // Handle each connection in a separate task
                            tokio::spawn(async move {
                                if let Err(e) = Self::handle_tcp_connection(socket, streaming_hub, config, source_id).await {
                                    error!("TCP connection error: {}", e);
                                }
                            });
                        }
                        Err(e) => {
                            error!("TCP accept error: {}", e);
                            tokio::time::sleep(Duration::from_millis(100)).await;
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// Handle individual TCP connection
    async fn handle_tcp_connection(
        socket: tokio::net::TcpStream,
        streaming_hub: Arc<StreamingHub>,
        config: StreamingSourceConfig,
        source_id: String,
    ) -> anyhow::Result<()> {
        let mut reader = BufReader::new(socket);
        let mut line = String::new();
        let mut buffer = Vec::new();
        let mut last_flush = Instant::now();

        loop {
            match reader.read_line(&mut line).await {
                Ok(0) => break, // Connection closed
                Ok(_) => {
                    if let Some(entry) = Self::parse_log_line(&line, &config, &source_id) {
                        buffer.push(entry);
                        line.clear();

                        // Check if we should flush the buffer
                        if buffer.len() >= config.buffer_size ||
                           last_flush.elapsed() >= config.batch_timeout {
                            streaming_hub.add_logs(config.project_id, &source_id, buffer.drain(..).collect()).await?;
                            last_flush = Instant::now();
                        }
                    }
                }
                Err(e) => return Err(e.into()),
            }
        }

        // Flush remaining buffer
        if !buffer.is_empty() {
            streaming_hub.add_logs(config.project_id, &source_id, buffer).await?;
        }

        Ok(())
    }

    /// Parse a log line according to the configured format
    fn parse_log_line(
        line: &str,
        config: &StreamingSourceConfig,
        source_id: &str,
    ) -> Option<StreamingLogEntry> {
        let line = line.trim();
        if line.is_empty() {
            return None;
        }

        match config.parser_config.log_format {
            LogFormat::Text => {
                Self::parse_text_log(line, config, source_id)
            }
            LogFormat::Json => {
                Self::parse_json_log(line, config, source_id)
            }
            LogFormat::Syslog => {
                Self::parse_syslog(line, config, source_id)
            }
            LogFormat::CommonLog => {
                Self::parse_common_log(line, config, source_id)
            }
            LogFormat::Regex { ref pattern } => {
                Self::parse_regex_log(line, pattern, config, source_id)
            }
        }
    }

    fn parse_text_log(
        line: &str,
        config: &StreamingSourceConfig,
        source_id: &str,
    ) -> Option<StreamingLogEntry> {
        // Use the existing parser from loglens-core
        let timestamp_regex = Regex::new(r"(\d{4}-\d{2}-\d{2}[T\s]\d{2}:\d{2}:\d{2}(?:\.\d{3})?(?:Z|[+-]\d{2}:\d{2})?)").unwrap();
        let level_regex = Regex::new(r"\b(ERROR|WARN|INFO|DEBUG|TRACE|FATAL)\b").unwrap();
        
        let log_entry = parse_single_log_line(line, &timestamp_regex);
        
        Some(StreamingLogEntry {
            id: Uuid::new_v4().to_string(),
            timestamp: log_entry.timestamp,
            level: log_entry.level,
            message: log_entry.message,
            source: source_id.to_string(),
            project_id: config.project_id,
            line_number: log_entry.line_number,
        })
    }

    fn parse_json_log(
        line: &str,
        config: &StreamingSourceConfig,
        source_id: &str,
    ) -> Option<StreamingLogEntry> {
        if let Ok(json_value) = serde_json::from_str::<serde_json::Value>(line) {
            let message = json_value.get("message")
                .or_else(|| json_value.get("msg"))
                .and_then(|v| v.as_str())
                .unwrap_or(line)
                .to_string();

            let level = json_value.get("level")
                .or_else(|| json_value.get("severity"))
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());

            let timestamp = json_value.get("timestamp")
                .or_else(|| json_value.get("time"))
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());

            Some(StreamingLogEntry {
                id: Uuid::new_v4().to_string(),
                timestamp,
                level,
                message,
                source: source_id.to_string(),
                project_id: config.project_id,
                line_number: None,
            })
        } else {
            None
        }
    }

    fn parse_syslog(
        line: &str,
        config: &StreamingSourceConfig,
        source_id: &str,
    ) -> Option<StreamingLogEntry> {
        // Basic syslog parsing - could be enhanced with proper RFC 3164/5424 parsing
        let timestamp_regex = Regex::new(r"(\d{4}-\d{2}-\d{2}[T\s]\d{2}:\d{2}:\d{2}(?:\.\d{3})?(?:Z|[+-]\d{2}:\d{2})?)").unwrap();
        let level_regex = Regex::new(r"\b(ERROR|WARN|INFO|DEBUG|TRACE|FATAL)\b").unwrap();
        
        let log_entry = parse_single_log_line(line, &timestamp_regex);
        
        Some(StreamingLogEntry {
            id: Uuid::new_v4().to_string(),
            timestamp: log_entry.timestamp,
            level: log_entry.level,
            message: log_entry.message,
            source: source_id.to_string(),
            project_id: config.project_id,
            line_number: log_entry.line_number,
        })
    }

    fn parse_common_log(
        _line: &str,
        config: &StreamingSourceConfig,
        source_id: &str,
    ) -> Option<StreamingLogEntry> {
        // Common log format parsing (Apache/Nginx)
        // This is a simplified version - could be enhanced
        Some(StreamingLogEntry {
            id: Uuid::new_v4().to_string(),
            timestamp: Some(chrono::Utc::now().to_rfc3339()),
            level: Some("INFO".to_string()),
            message: _line.to_string(),
            source: source_id.to_string(),
            project_id: config.project_id,
            line_number: None,
        })
    }

    fn parse_regex_log(
        line: &str,
        pattern: &str,
        config: &StreamingSourceConfig,
        source_id: &str,
    ) -> Option<StreamingLogEntry> {
        if let Ok(regex) = regex::Regex::new(pattern) {
            if let Some(captures) = regex.captures(line) {
                let message = captures.name("message")
                    .map(|m| m.as_str())
                    .unwrap_or(line)
                    .to_string();

                let level = captures.name("level")
                    .map(|m| m.as_str())
                    .map(|s| s.to_string());

                let timestamp = captures.name("timestamp")
                    .map(|m| m.as_str())
                    .map(|s| s.to_string());

                return Some(StreamingLogEntry {
                    id: Uuid::new_v4().to_string(),
                    timestamp,
                    level,
                    message,
                    source: source_id.to_string(),
                    project_id: config.project_id,
                    line_number: None,
                });
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use tokio::io::AsyncWriteExt;

    #[tokio::test]
    async fn test_streaming_source_manager() {
        let hub = Arc::new(StreamingHub::new());
        let mut manager = StreamingSourceManager::new(hub);
        
        assert_eq!(manager.active_source_count(), 0);
    }

    #[tokio::test]
    async fn test_text_log_parsing() {
        let config = StreamingSourceConfig {
            source_type: StreamingSourceType::Stdin,
            project_id: Uuid::new_v4(),
            name: "test".to_string(),
            parser_config: ParserConfig::default(),
            buffer_size: 10,
            batch_timeout: Duration::from_secs(1),
            restart_on_error: false,
            max_restarts: None,
        };

        let entry = StreamingSourceManager::parse_log_line(
            "2023-01-01T12:00:00Z [ERROR] Test message",
            &config,
            "test-source"
        );

        assert!(entry.is_some());
        let entry = entry.unwrap();
        assert_eq!(entry.level, Some("ERROR".to_string()));
        assert!(entry.message.contains("Test message"));
    }

    #[tokio::test]
    async fn test_json_log_parsing() {
        let mut config = StreamingSourceConfig {
            source_type: StreamingSourceType::Stdin,
            project_id: Uuid::new_v4(),
            name: "test".to_string(),
            parser_config: ParserConfig::default(),
            buffer_size: 10,
            batch_timeout: Duration::from_secs(1),
            restart_on_error: false,
            max_restarts: None,
        };
        config.parser_config.log_format = LogFormat::Json;

        let json_line = r#"{"timestamp": "2023-01-01T12:00:00Z", "level": "error", "message": "Test JSON message", "service": "test-service"}"#;
        
        let entry = StreamingSourceManager::parse_log_line(
            json_line,
            &config,
            "test-source"
        );

        assert!(entry.is_some());
        let entry = entry.unwrap();
        assert_eq!(entry.level, Some("error".to_string()));
        assert_eq!(entry.message, "Test JSON message");
    }
}