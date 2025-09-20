//! Log Buffer for Process Monitoring
//! 
//! Provides real-time log buffering with automatic flushing and size management
//! for continuous AI analysis integration.

use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use std::thread;
use crate::model::LogEntry;
use chrono::Utc;
use anyhow::Result;
use std::sync::mpsc::{self, Sender, Receiver};
use serde::{Deserialize, Serialize};

/// Buffer configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BufferConfig {
    pub max_size: usize,
    pub flush_interval_ms: u64,
    pub auto_flush: bool,
    pub keep_recent_lines: usize,
    pub compression_threshold: usize,
}

/// Buffer event types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BufferEvent {
    LineAdded { count: usize },
    BufferFlushed { count: usize },
    BufferFull { size: usize },
    TriggerMatched { pattern: String, line_count: usize },
}

/// Flushed buffer content with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BufferFlush {
    pub timestamp: chrono::DateTime<Utc>,
    pub entries: Vec<LogEntry>,
    pub trigger_patterns: Vec<String>,
    pub total_lines: usize,
    pub buffer_size_at_flush: usize,
}

/// Real-time log buffer with AI analysis integration
pub struct LogBuffer {
    config: BufferConfig,
    buffer: VecDeque<LogEntry>,
    recent_lines: VecDeque<LogEntry>,
    total_lines_received: usize,
    total_lines_flushed: usize,
    trigger_patterns: Vec<String>,
    event_sender: Sender<BufferEvent>,
    event_receiver: Option<Receiver<BufferEvent>>,
    last_flush_time: Instant,
    flush_thread: Option<thread::JoinHandle<()>>,
}

impl LogBuffer {
    /// Create a new log buffer with configuration
    pub fn new(mut config: BufferConfig) -> Self {
        let (sender, receiver) = mpsc::channel();
        
        let buffer = Self {
            config: config.clone(),
            buffer: VecDeque::with_capacity(config.max_size),
            recent_lines: VecDeque::with_capacity(config.keep_recent_lines),
            total_lines_received: 0,
            total_lines_flushed: 0,
            trigger_patterns: Vec::new(),
            event_sender: sender.clone(),
            event_receiver: Some(receiver),
            last_flush_time: Instant::now(),
            flush_thread: None,
        };
        
        // Start auto-flush thread if enabled
        let interval_ms = config.flush_interval_ms;
        if config.auto_flush {
            Self::start_auto_flush_thread(&sender, interval_ms);
        }
        
        buffer
    }
    
    /// Add a log entry to the buffer
    pub fn add_entry(&mut self, entry: LogEntry) -> Result<()> {
        self.total_lines_received += 1;
        
        // Add to main buffer
        if self.buffer.len() >= self.config.max_size {
            // Remove oldest entry if buffer is full
            self.buffer.pop_front();
            self.send_event(BufferEvent::BufferFull { 
                size: self.buffer.len() 
            });
        }
        self.buffer.push_back(entry.clone());
        
        // Add to recent lines buffer
        if self.recent_lines.len() >= self.config.keep_recent_lines {
            self.recent_lines.pop_front();
        }
        self.recent_lines.push_back(entry.clone());
        
        // Send line added event
        self.send_event(BufferEvent::LineAdded { count: 1 });
        
        // Check if auto-flush is needed based on buffer size
        if self.buffer.len() >= self.config.compression_threshold {
            self.flush()?;
        }
        
        Ok(())
    }
    
    /// Add multiple log entries to the buffer
    pub fn add_entries(&mut self, entries: Vec<LogEntry>) -> Result<usize> {
        let count = entries.len();
        self.total_lines_received += count;
        
        for entry in entries {
            // Add to main buffer
            if self.buffer.len() >= self.config.max_size {
                self.buffer.pop_front();
            }
            self.buffer.push_back(entry.clone());
            
            // Add to recent lines buffer
            if self.recent_lines.len() >= self.config.keep_recent_lines {
                self.recent_lines.pop_front();
            }
            self.recent_lines.push_back(entry.clone());
            
            // Check for trigger patterns
            self.check_triggers(&entry);
        }
        
        // Send line added event
        self.send_event(BufferEvent::LineAdded { count });
        
        // Check if auto-flush is needed
        if self.buffer.len() >= self.config.compression_threshold {
            self.flush()?;
        }
        
        Ok(count)
    }
    
    /// Manually flush the buffer
    pub fn flush(&mut self) -> Result<BufferFlush> {
        let entries: Vec<LogEntry> = self.buffer.drain(..).collect();
        let flush_size = entries.len();
        
        if flush_size > 0 {
            self.total_lines_flushed += flush_size;
            self.last_flush_time = Instant::now();
            
            let flush_data = BufferFlush {
                timestamp: Utc::now(),
                entries: entries.clone(),
                trigger_patterns: self.trigger_patterns.clone(),
                total_lines: flush_size,
                buffer_size_at_flush: flush_size,
            };
            
            // Clear trigger patterns after flush
            self.trigger_patterns.clear();
            
            // Send flush event
            self.send_event(BufferEvent::BufferFlushed { 
                count: flush_size 
            });
            
            Ok(flush_data)
        } else {
            Err(anyhow::anyhow!("Buffer is empty, nothing to flush"))
        }
    }
    
    /// Get all current entries in the buffer
    pub fn get_entries(&self) -> Vec<LogEntry> {
        self.buffer.iter().cloned().collect()
    }
    
    /// Get recent entries (last N lines)
    pub fn get_recent_entries(&self, count: usize) -> Vec<LogEntry> {
        let start_idx = if count > self.recent_lines.len() {
            0
        } else {
            self.recent_lines.len() - count
        };
        
        self.recent_lines.range(start_idx..)
            .cloned()
            .collect()
    }
    
    /// Get buffer statistics
    pub fn get_stats(&self) -> BufferStats {
        BufferStats {
            current_size: self.buffer.len(),
            max_size: self.config.max_size,
            total_lines_received: self.total_lines_received,
            total_lines_flushed: self.total_lines_flushed,
            recent_lines_count: self.recent_lines.len(),
            buffer_usage_percent: (self.buffer.len() as f64 / self.config.max_size as f64) * 100.0,
            time_since_last_flush: self.last_flush_time.elapsed(),
            trigger_patterns_count: self.trigger_patterns.len(),
        }
    }
    
    /// Add trigger pattern to watch for
    pub fn add_trigger_pattern(&mut self, pattern: String) {
        if !self.trigger_patterns.contains(&pattern) {
            self.trigger_patterns.push(pattern);
        }
    }
    
    /// Remove trigger pattern
    pub fn remove_trigger_pattern(&mut self, pattern: &str) {
        self.trigger_patterns.retain(|p| p != pattern);
    }
    
    /// Get active trigger patterns
    pub fn get_trigger_patterns(&self) -> Vec<String> {
        self.trigger_patterns.clone()
    }
    
    /// Try to receive buffer events
    pub fn try_receive_events(&mut self) -> Vec<BufferEvent> {
        if let Some(ref receiver) = self.event_receiver {
            let mut events = Vec::new();
            
            while let Ok(event) = receiver.try_recv() {
                events.push(event);
            }
            
            events
        } else {
            Vec::new()
        }
    }
    
    // Helper function to start auto-flush thread
pub fn start_auto_flush_thread(sender: &Sender<BufferEvent>, interval_ms: u64) {
    let sender = sender.clone();
    
    thread::spawn(move || {
        loop {
            thread::sleep(Duration::from_millis(interval_ms));
            
            // Send flush trigger event
            let _ = sender.send(BufferEvent::BufferFlushed { count: 0 });
        }
    });
}
    
    /// Check if buffer should be flushed based on size
    pub fn should_flush_by_size(&self) -> bool {
        self.buffer.len() >= self.config.compression_threshold
    }
    
    /// Clear the buffer
    pub fn clear(&mut self) {
        self.buffer.clear();
        self.trigger_patterns.clear();
        self.last_flush_time = Instant::now();
    }
    
    /// Get current buffer size
    pub fn len(&self) -> usize {
        self.buffer.len()
    }
    
    /// Check if buffer is empty
    pub fn is_empty(&self) -> bool {
        self.buffer.is_empty()
    }
    
    /// Get remaining capacity
    pub fn remaining_capacity(&self) -> usize {
        self.config.max_size.saturating_sub(self.buffer.len())
    }
    
    /// Start auto-flush thread
    fn start_auto_flush(&mut self) {
        let interval_ms = self.config.flush_interval_ms;
        
        let sender = self.event_sender.clone();
        
        Self::start_auto_flush_thread(&sender, interval_ms);
    }
    
    /// Check for trigger patterns in log entry
    fn check_triggers(&mut self, entry: &LogEntry) {
        let message = entry.message.to_lowercase();
        let raw_line = entry.raw_line.to_lowercase();
        
        for pattern in &self.trigger_patterns {
            let pattern_lower = pattern.to_lowercase();
            
            if message.contains(&pattern_lower) || raw_line.contains(&pattern_lower) {
                self.send_event(BufferEvent::TriggerMatched { 
                    pattern: pattern.clone(), 
                    line_count: 1 
                });
            }
        }
    }
    
    /// Send buffer event
    fn send_event(&self, event: BufferEvent) {
        let _ = self.event_sender.send(event);
    }
}

/// Buffer statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BufferStats {
    pub current_size: usize,
    pub max_size: usize,
    pub total_lines_received: usize,
    pub total_lines_flushed: usize,
    pub recent_lines_count: usize,
    pub buffer_usage_percent: f64,
    pub time_since_last_flush: Duration,
    pub trigger_patterns_count: usize,
}

impl Drop for LogBuffer {
    fn drop(&mut self) {
        // Flush remaining data before dropping
        if let Err(e) = self.flush() {
            eprintln!("Error flushing buffer during drop: {}", e);
        }
    }
}

/// Thread-safe wrapper for LogBuffer
pub struct SharedLogBuffer {
    buffer: Arc<Mutex<LogBuffer>>,
}

impl SharedLogBuffer {
    /// Create a new shared log buffer
    pub fn new(config: BufferConfig) -> Self {
        Self {
            buffer: Arc::new(Mutex::new(LogBuffer::new(config))),
        }
    }
    
    /// Add a log entry (thread-safe)
    pub fn add_entry(&self, entry: LogEntry) -> Result<()> {
        let mut buffer = self.buffer.lock()
            .map_err(|e| anyhow::anyhow!("Failed to lock buffer: {}", e))?;
        buffer.add_entry(entry)
    }
    
    /// Add multiple log entries (thread-safe)
    pub fn add_entries(&self, entries: Vec<LogEntry>) -> Result<usize> {
        let mut buffer = self.buffer.lock()
            .map_err(|e| anyhow::anyhow!("Failed to lock buffer: {}", e))?;
        buffer.add_entries(entries)
    }
    
    /// Flush the buffer (thread-safe)
    pub fn flush(&self) -> Result<BufferFlush> {
        let mut buffer = self.buffer.lock()
            .map_err(|e| anyhow::anyhow!("Failed to lock buffer: {}", e))?;
        buffer.flush()
    }
    
    /// Get buffer statistics (thread-safe)
    pub fn get_stats(&self) -> Result<BufferStats> {
        let buffer = self.buffer.lock()
            .map_err(|e| anyhow::anyhow!("Failed to lock buffer: {}", e))?;
        Ok(buffer.get_stats())
    }
    
    /// Add trigger pattern (thread-safe)
    pub fn add_trigger_pattern(&self, pattern: String) -> Result<()> {
        let mut buffer = self.buffer.lock()
            .map_err(|e| anyhow::anyhow!("Failed to lock buffer: {}", e))?;
        buffer.add_trigger_pattern(pattern);
        Ok(())
    }
    
    /// Get recent entries (thread-safe)
    pub fn get_recent_entries(&self, count: usize) -> Result<Vec<LogEntry>> {
        let buffer = self.buffer.lock()
            .map_err(|e| anyhow::anyhow!("Failed to lock buffer: {}", e))?;
        Ok(buffer.get_recent_entries(count))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_log_buffer_creation() {
        let config = BufferConfig {
            max_size: 100,
            flush_interval_ms: 1000,
            auto_flush: false,
            keep_recent_lines: 10,
            compression_threshold: 80,
        };
        
        let buffer = LogBuffer::new(config);
        assert_eq!(buffer.len(), 0);
        assert!(buffer.is_empty());
        assert_eq!(buffer.remaining_capacity(), 100);
    }

    #[test]
    fn test_add_entry() -> Result<()> {
        let config = BufferConfig {
            max_size: 10,
            flush_interval_ms: 1000,
            auto_flush: false,
            keep_recent_lines: 5,
            compression_threshold: 8,
        };
        
        let mut buffer = LogBuffer::new(config);
        
        let entry = LogEntry {
            timestamp: Utc::now(),
            level: LogLevel::Info,
            message: "Test message".to_string(),
            fields: std::collections::HashMap::new(),
            raw_line: "Test message".to_string(),
        };
        
        buffer.add_entry(entry)?;
        
        assert_eq!(buffer.len(), 1);
        assert!(!buffer.is_empty());
        assert_eq!(buffer.remaining_capacity(), 9);
        
        Ok(())
    }

    #[test]
    fn test_buffer_full_handling() -> Result<()> {
        let config = BufferConfig {
            max_size: 3,
            flush_interval_ms: 1000,
            auto_flush: false,
            keep_recent_lines: 5,
            compression_threshold: 10,
        };
        
        let mut buffer = LogBuffer::new(config);
        
        for i in 0..5 {
            let entry = LogEntry {
                timestamp: Utc::now(),
                level: LogLevel::Info,
                message: format!("Message {}", i),
                fields: std::collections::HashMap::new(),
                raw_line: format!("Message {}", i),
            };
            
            buffer.add_entry(entry)?;
        }
        
        assert_eq!(buffer.len(), 3); // Should be at max size
        assert_eq!(buffer.remaining_capacity(), 0);
        
        let entries = buffer.get_entries();
        assert_eq!(entries.len(), 3);
        // Should contain the last 3 messages (2, 3, 4)
        assert!(entries[0].message.contains("2"));
        assert!(entries[2].message.contains("4"));
        
        Ok(())
    }

    #[test]
    fn test_flush_buffer() -> Result<()> {
        let config = BufferConfig {
            max_size: 10,
            flush_interval_ms: 1000,
            auto_flush: false,
            keep_recent_lines: 5,
            compression_threshold: 10,
        };
        
        let mut buffer = LogBuffer::new(config);
        
        // Add some entries
        for i in 0..3 {
            let entry = LogEntry {
                timestamp: Utc::now(),
                level: LogLevel::Info,
                message: format!("Message {}", i),
                fields: std::collections::HashMap::new(),
                raw_line: format!("Message {}", i),
            };
            
            buffer.add_entry(entry)?;
        }
        
        assert_eq!(buffer.len(), 3);
        
        // Flush the buffer
        let flush_data = buffer.flush()?;
        
        assert_eq!(flush_data.entries.len(), 3);
        assert_eq!(buffer.len(), 0);
        assert!(buffer.is_empty());
        
        Ok(())
    }

    #[test]
    fn test_trigger_patterns() -> Result<()> {
        let config = BufferConfig {
            max_size: 10,
            flush_interval_ms: 1000,
            auto_flush: false,
            keep_recent_lines: 5,
            compression_threshold: 10,
        };
        
        let mut buffer = LogBuffer::new(config);
        
        // Add trigger patterns
        buffer.add_trigger_pattern("ERROR".to_string());
        buffer.add_trigger_pattern("CRITICAL".to_string());
        
        assert_eq!(buffer.get_trigger_patterns().len(), 2);
        
        // Add an entry that matches a trigger
        let entry = LogEntry {
            timestamp: Utc::now(),
            level: LogLevel::Error,
            message: "ERROR: Something went wrong".to_string(),
            fields: std::collections::HashMap::new(),
            raw_line: "ERROR: Something went wrong".to_string(),
        };
        
        buffer.add_entry(entry)?;
        
        // Check for events (trigger should have matched)
        let events = buffer.try_receive_events();
        assert!(events.iter().any(|e| matches!(e, BufferEvent::TriggerMatched { .. })));
        
        Ok(())
    }
}