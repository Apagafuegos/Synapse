//! Process Executor
//! 
//! Handles command execution with real-time stdio capture and monitoring

use std::process::{Command, Stdio, Child, ChildStdout, ChildStderr};
use std::thread;
use std::sync::mpsc::{self, Sender, Receiver};
use std::io::{self, BufRead, BufReader};
use std::time::{Duration, Instant};
use anyhow::{Context, Result};
use crate::model::LogEntry;
use crate::model::LogLevel;
use chrono::Utc;
use std::collections::VecDeque;
use serde::{Deserialize, Serialize};

/// Process execution configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessExecutionConfig {
    pub command: String,
    pub args: Vec<String>,
    pub working_dir: Option<String>,
    pub env_vars: Option<std::collections::HashMap<String, String>>,
    pub timeout_seconds: Option<u64>,
    pub buffer_size: usize,
    pub auto_flush_interval_ms: u64,
}

/// Process output line with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessOutputLine {
    pub timestamp: chrono::DateTime<Utc>,
    pub stream_type: StreamType,
    pub content: String,
    pub line_number: usize,
}

/// Stream type for process output
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum StreamType {
    Stdout,
    Stderr,
    Mixed,
}

/// Process execution status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ProcessStatus {
    Starting,
    Running,
    Completed { exit_code: i32 },
    Failed { error: String },
    Timeout,
    Killed,
}

/// Process manager for executing and monitoring commands
pub struct ProcessManager {
    config: ProcessExecutionConfig,
    status: ProcessStatus,
    start_time: Option<Instant>,
    process: Option<Child>,
    output_sender: Sender<ProcessOutputLine>,
    output_receiver: Option<Receiver<ProcessOutputLine>>,
    log_buffer: VecDeque<ProcessOutputLine>,
    total_lines: usize,
    error_count: usize,
}

impl ProcessManager {
    /// Create a new process manager
    pub fn new(config: ProcessExecutionConfig) -> Self {
        let (sender, receiver) = mpsc::channel();
        
        Self {
            config: config.clone(),
            status: ProcessStatus::Starting,
            start_time: None,
            process: None,
            output_sender: sender,
            output_receiver: Some(receiver),
            log_buffer: VecDeque::with_capacity(config.buffer_size),
            total_lines: 0,
            error_count: 0,
        }
    }
    
    /// Execute the configured command
    pub fn execute(&mut self) -> Result<()> {
        self.status = ProcessStatus::Starting;
        self.start_time = Some(Instant::now());
        
        let mut cmd = Command::new(&self.config.command);
        
        // Set arguments
        cmd.args(&self.config.args);
        
        // Set working directory if specified
        if let Some(ref work_dir) = self.config.working_dir {
            cmd.current_dir(work_dir);
        }
        
        // Set environment variables if specified
        if let Some(ref env_vars) = self.config.env_vars {
            for (key, value) in env_vars {
                cmd.env(key, value);
            }
        }
        
        // Configure stdio for capture
        cmd.stdin(Stdio::null())
           .stdout(Stdio::piped())
           .stderr(Stdio::piped());
        
        // Start the process
        let mut child = cmd.spawn()
            .context(format!("Failed to execute command: {} {}", self.config.command, self.config.args.join(" ")))?;
        
        self.status = ProcessStatus::Running;
        self.process = Some(child);
        
        // Spawn threads to capture stdout and stderr
        if let Some(ref mut process) = self.process {
            let stdout = process.stdout.take().expect("Failed to capture stdout");
            let stderr = process.stderr.take().expect("Failed to capture stderr");
            
            let sender = self.output_sender.clone();
            
            // Spawn stdout reader thread
            let stdout_sender = sender.clone();
            thread::spawn(move || {
                Self::capture_stream(stdout, stdout_sender, StreamType::Stdout);
            });
            
            // Spawn stderr reader thread
            thread::spawn(move || {
                Self::capture_stream(stderr, sender, StreamType::Stderr);
            });
        }
        
        Ok(())
    }
    
    /// Wait for process completion with optional timeout
    pub fn wait_for_completion(&mut self) -> Result<()> {
        if let Some(ref mut process) = self.process {
            let timeout_duration = self.config.timeout_seconds
                .map(|secs| Duration::from_secs(secs));
            
            match timeout_duration {
                Some(timeout) => {
                    match process.wait_timeout(timeout) {
                        Ok(Some(exit_status)) => {
                            self.status = ProcessStatus::Completed { 
                                exit_code: exit_status.code().unwrap_or(-1) 
                            };
                        }
                        Ok(None) => {
                            self.status = ProcessStatus::Timeout;
                            process.kill().context("Failed to kill timed-out process")?;
                        }
                        Err(e) => {
                            self.status = ProcessStatus::Failed { 
                                error: format!("Failed to wait for process: {}", e) 
                            };
                        }
                    }
                }
                None => {
                    match process.wait() {
                        Ok(exit_status) => {
                            self.status = ProcessStatus::Completed { 
                                exit_code: exit_status.code().unwrap_or(-1) 
                            };
                        }
                        Err(e) => {
                            self.status = ProcessStatus::Failed { 
                                error: format!("Failed to wait for process: {}", e) 
                            };
                        }
                    }
                }
            }
        }
        
        Ok(())
    }
    
    /// Get the current process status
    pub fn status(&self) -> &ProcessStatus {
        &self.status
    }
    
    /// Check if process is still running
    pub fn is_running(&self) -> bool {
        matches!(self.status, ProcessStatus::Running)
    }
    
    /// Get the execution time
    pub fn execution_time(&self) -> Option<Duration> {
        self.start_time.map(|start| start.elapsed())
    }
    
    /// Get buffered output lines
    pub fn get_buffered_output(&self) -> Vec<ProcessOutputLine> {
        self.log_buffer.iter().cloned().collect()
    }
    
    /// Get recent output lines (last N lines)
    pub fn get_recent_output(&self, count: usize) -> Vec<ProcessOutputLine> {
        let start_idx = if count > self.log_buffer.len() {
            0
        } else {
            self.log_buffer.len() - count
        };
        
        self.log_buffer.range(start_idx..)
            .cloned()
            .collect()
    }
    
    /// Get total lines processed
    pub fn total_lines(&self) -> usize {
        self.total_lines
    }
    
    /// Get error count (stderr lines)
    pub fn error_count(&self) -> usize {
        self.error_count
    }
    
    /// Convert buffered output to LogEntry format for AI analysis
    pub fn to_log_entries(&self) -> Vec<LogEntry> {
        self.log_buffer.iter()
            .filter_map(|line| {
                let log_level = if line.stream_type == StreamType::Stderr {
                    LogLevel::Error
                } else {
                    // Simple heuristic - could be improved with pattern matching
                    if line.content.to_lowercase().contains("error") {
                        LogLevel::Error
                    } else if line.content.to_lowercase().contains("warn") {
                        LogLevel::Warn
                    } else if line.content.to_lowercase().contains("info") {
                        LogLevel::Info
                    } else {
                        LogLevel::Debug
                    }
                };
                
                Some(LogEntry {
                    timestamp: line.timestamp,
                    level: log_level,
                    message: line.content.clone(),
                    fields: std::collections::HashMap::new(),
                    raw_line: line.content.clone(),
                })
            })
            .collect()
    }
    
    /// Kill the running process
    pub fn kill(&mut self) -> Result<()> {
        let was_running = self.is_running();
        if let Some(ref mut process) = self.process {
            if was_running {
                process.kill().context("Failed to kill process")?;
                self.status = ProcessStatus::Killed;
            }
        }
        Ok(())
    }
    
    /// Try receive pending output lines
    pub fn try_receive_output(&mut self) -> Vec<ProcessOutputLine> {
        if let Some(ref receiver) = self.output_receiver {
            let mut lines = Vec::new();
            
            while let Ok(line) = receiver.try_recv() {
                // Add to buffer, maintaining size limit
                if self.log_buffer.len() >= self.config.buffer_size {
                    self.log_buffer.pop_front();
                }
                self.log_buffer.push_back(line.clone());
                
                // Update counters
                self.total_lines += 1;
                if line.stream_type == StreamType::Stderr {
                    self.error_count += 1;
                }
                
                lines.push(line);
            }
            
            lines
        } else {
            Vec::new()
        }
    }
    
    /// Capture output from a stream (stdout/stderr)
    fn capture_stream(
        stream: impl std::io::Read + Send + 'static,
        sender: Sender<ProcessOutputLine>,
        stream_type: StreamType,
    ) {
        let reader = BufReader::new(stream);
        let mut line_number = 0;
        
        for line in reader.lines() {
            match line {
                Ok(content) => {
                    line_number += 1;
                    let output_line = ProcessOutputLine {
                        timestamp: Utc::now(),
                        stream_type: stream_type.clone(),
                        content,
                        line_number,
                    };
                    
                    // Send to main thread, ignore errors if receiver is dropped
                    let _ = sender.send(output_line);
                }
                Err(e) => {
                    eprintln!("Error reading from stream: {}", e);
                    break;
                }
            }
        }
    }
}

/// Extension trait for wait_timeout functionality
trait WaitTimeoutExt {
    fn wait_timeout(&mut self, timeout: Duration) -> io::Result<Option<std::process::ExitStatus>>;
}

impl WaitTimeoutExt for std::process::Child {
    fn wait_timeout(&mut self, timeout: Duration) -> io::Result<Option<std::process::ExitStatus>> {
        let start = Instant::now();
        
        loop {
            match self.try_wait()? {
                Some(status) => return Ok(Some(status)),
                None => {
                    if start.elapsed() >= timeout {
                        return Ok(None);
                    }
                    thread::sleep(Duration::from_millis(50));
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process_manager_creation() {
        let config = ProcessExecutionConfig {
            command: "echo".to_string(),
            args: vec!["Hello, World!".to_string()],
            working_dir: None,
            env_vars: None,
            timeout_seconds: Some(5),
            buffer_size: 100,
            auto_flush_interval_ms: 1000,
        };
        
        let manager = ProcessManager::new(config);
        assert!(matches!(manager.status(), ProcessStatus::Starting));
        assert_eq!(manager.total_lines(), 0);
        assert_eq!(manager.error_count(), 0);
    }

    #[test]
    fn test_process_execution() -> Result<()> {
        let config = ProcessExecutionConfig {
            command: "echo".to_string(),
            args: vec!["test output".to_string()],
            working_dir: None,
            env_vars: None,
            timeout_seconds: Some(5),
            buffer_size: 100,
            auto_flush_interval_ms: 1000,
        };
        
        let mut manager = ProcessManager::new(config);
        manager.execute()?;
        manager.wait_for_completion()?;
        
        let _output = manager.try_receive_output();
        
        assert!(!matches!(manager.status(), ProcessStatus::Running));
        assert!(manager.execution_time().is_some());
        
        Ok(())
    }

    #[test]
    fn test_log_entry_conversion() {
        // This would need more sophisticated testing with actual process output
        // For now, we just test the structure exists
        let config = ProcessExecutionConfig {
            command: "echo".to_string(),
            args: vec!["INFO: Test message".to_string()],
            working_dir: None,
            env_vars: None,
            timeout_seconds: Some(1),
            buffer_size: 10,
            auto_flush_interval_ms: 100,
        };
        
        let manager = ProcessManager::new(config);
        let log_entries = manager.to_log_entries();
        assert_eq!(log_entries.len(), 0); // No output yet
    }
}