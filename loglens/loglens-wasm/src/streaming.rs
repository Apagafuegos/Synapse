use js_sys::Date;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{MessageEvent, WebSocket};
use web_sys::console;

/// Maximum number of log entries to keep in memory for real-time display
const MAX_STREAMING_ENTRIES: usize = 1000;

/// Real-time streaming message types from the WebSocket
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum StreamingMessage {
    #[serde(rename = "log_batch")]
    LogBatch { entries: Vec<StreamingLogEntry>, batch_id: String },
    #[serde(rename = "heartbeat")]
    Heartbeat { timestamp: String },
    #[serde(rename = "error")]
    Error { message: String, code: Option<String> },
    #[serde(rename = "subscription_status")]
    SubscriptionStatus { subscribed: bool, project_id: String },
}

/// Individual log entry in streaming format
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamingLogEntry {
    pub id: String,
    pub timestamp: String,
    pub level: String,
    pub message: String,
    pub source: String,
    pub project_id: String,
    pub metadata: Option<HashMap<String, serde_json::Value>>,
}

/// Real-time analytics data structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiveAnalytics {
    pub total_entries: usize,
    pub entries_by_level: HashMap<String, usize>,
    pub entries_by_source: HashMap<String, usize>,
    pub recent_entries: VecDeque<StreamingLogEntry>,
    pub errors_per_minute: Vec<f64>,
    pub warnings_per_minute: Vec<f64>,
    pub last_update: f64,
    pub connection_status: String,
}

impl Default for LiveAnalytics {
    fn default() -> Self {
        Self {
            total_entries: 0,
            entries_by_level: HashMap::new(),
            entries_by_source: HashMap::new(),
            recent_entries: VecDeque::new(),
            errors_per_minute: Vec::new(),
            warnings_per_minute: Vec::new(),
            last_update: Date::now(),
            connection_status: "disconnected".to_string(),
        }
    }
}

/// WebSocket streaming client for real-time log data
#[wasm_bindgen]
pub struct StreamingClient {
    websocket: Option<WebSocket>,
    analytics: LiveAnalytics,
    project_id: String,
    on_message_callback: Option<js_sys::Function>,
    on_analytics_update_callback: Option<js_sys::Function>,
    connection_url: String,
}

#[wasm_bindgen]
impl StreamingClient {
    /// Create a new streaming client for a project
    #[wasm_bindgen(constructor)]
    pub fn new(project_id: &str, base_url: &str) -> StreamingClient {
        let connection_url = format!("{}/api/projects/{}/stream", base_url, project_id);
        
        StreamingClient {
            websocket: None,
            analytics: LiveAnalytics::default(),
            project_id: project_id.to_string(),
            on_message_callback: None,
            on_analytics_update_callback: None,
            connection_url,
        }
    }

    /// Connect to the streaming WebSocket
    #[wasm_bindgen]
    pub fn connect(&mut self) -> Result<(), JsValue> {
        let ws = WebSocket::new(&self.connection_url)?;
        
        // Set up message handler
        let analytics_ptr = &mut self.analytics as *mut LiveAnalytics;
        let on_message_callback = self.on_message_callback.clone();
        let on_analytics_callback = self.on_analytics_update_callback.clone();
        
        let onmessage_callback = Closure::wrap(Box::new(move |e: MessageEvent| {
            if let Ok(txt) = e.data().dyn_into::<js_sys::JsString>() {
                let message_str = txt.as_string().unwrap_or_default();
                
                // Parse the streaming message
                if let Ok(message) = serde_json::from_str::<StreamingMessage>(&message_str) {
                    unsafe {
                        if let Some(analytics) = analytics_ptr.as_mut() {
                            Self::process_message(analytics, message);
                            
                            // Trigger analytics update callback
                            if let Some(callback) = &on_analytics_callback {
                                if let Ok(analytics_js) = serde_wasm_bindgen::to_value(analytics) {
                                    let _ = callback.call1(&JsValue::NULL, &analytics_js);
                                }
                            }
                        }
                    }
                }
                
                // Trigger raw message callback
                if let Some(callback) = &on_message_callback {
                    let _ = callback.call1(&JsValue::NULL, &JsValue::from_str(&message_str));
                }
            }
        }) as Box<dyn FnMut(_)>);
        
        ws.set_onmessage(Some(onmessage_callback.as_ref().unchecked_ref()));
        onmessage_callback.forget();
        
        // Set up connection handlers
        let onopen_callback = Closure::wrap(Box::new(move |_event: web_sys::Event| {
            console::log_1(&"WebSocket connected successfully".into());
        }) as Box<dyn FnMut(web_sys::Event)>);
        ws.set_onopen(Some(onopen_callback.as_ref().unchecked_ref()));
        onopen_callback.forget();
        
        let onerror_callback = Closure::wrap(Box::new(move |_event: web_sys::Event| {
            console::log_1(&"WebSocket connection error".into());
        }) as Box<dyn FnMut(web_sys::Event)>);
        ws.set_onerror(Some(onerror_callback.as_ref().unchecked_ref()));
        onerror_callback.forget();
        
        self.websocket = Some(ws);
        self.analytics.connection_status = "connected".to_string();
        
        Ok(())
    }

    /// Disconnect from the WebSocket
    #[wasm_bindgen]
    pub fn disconnect(&mut self) {
        if let Some(ws) = &self.websocket {
            let _ = ws.close();
        }
        self.websocket = None;
        self.analytics.connection_status = "disconnected".to_string();
    }

    /// Set callback for raw WebSocket messages
    #[wasm_bindgen]
    pub fn set_on_message(&mut self, callback: js_sys::Function) {
        self.on_message_callback = Some(callback);
    }

    /// Set callback for analytics updates
    #[wasm_bindgen]
    pub fn set_on_analytics_update(&mut self, callback: js_sys::Function) {
        self.on_analytics_update_callback = Some(callback);
    }

    /// Get current analytics data
    #[wasm_bindgen]
    pub fn get_analytics(&self) -> Result<JsValue, JsValue> {
        Ok(serde_wasm_bindgen::to_value(&self.analytics)?)
    }

    /// Get recent log entries
    #[wasm_bindgen]
    pub fn get_recent_entries(&self, limit: Option<usize>) -> Result<JsValue, JsValue> {
        let limit = limit.unwrap_or(100);
        let recent: Vec<_> = self.analytics.recent_entries
            .iter()
            .take(limit)
            .collect();
        Ok(serde_wasm_bindgen::to_value(&recent)?)
    }

    /// Clear all analytics data
    #[wasm_bindgen]
    pub fn clear_analytics(&mut self) {
        self.analytics = LiveAnalytics::default();
        self.analytics.connection_status = 
            if self.websocket.is_some() { "connected" } else { "disconnected" }.to_string();
    }

    /// Send a client message to the server (e.g., for filtering)
    #[wasm_bindgen]
    pub fn send_message(&self, message: &str) -> Result<(), JsValue> {
        if let Some(ws) = &self.websocket {
            ws.send_with_str(message)?;
        }
        Ok(())
    }

    /// Get connection status
    #[wasm_bindgen]
    pub fn get_connection_status(&self) -> String {
        self.analytics.connection_status.clone()
    }
}

impl StreamingClient {
    /// Process incoming streaming message and update analytics
    fn process_message(analytics: &mut LiveAnalytics, message: StreamingMessage) {
        match message {
            StreamingMessage::LogBatch { entries, .. } => {
                for entry in entries {
                    // Update counters
                    analytics.total_entries += 1;
                    *analytics.entries_by_level.entry(entry.level.clone()).or_insert(0) += 1;
                    *analytics.entries_by_source.entry(entry.source.clone()).or_insert(0) += 1;
                    
                    // Add to recent entries
                    analytics.recent_entries.push_front(entry);
                    
                    // Keep only the most recent entries
                    if analytics.recent_entries.len() > MAX_STREAMING_ENTRIES {
                        analytics.recent_entries.pop_back();
                    }
                }
                
                // Update time-based metrics
                Self::update_time_based_metrics(analytics);
            }
            StreamingMessage::Heartbeat { .. } => {
                analytics.connection_status = "connected".to_string();
            }
            StreamingMessage::Error { message, .. } => {
                console::log_1(&format!("Streaming error: {}", message).into());
                analytics.connection_status = "error".to_string();
            }
            StreamingMessage::SubscriptionStatus { subscribed, .. } => {
                analytics.connection_status = if subscribed { "subscribed" } else { "unsubscribed" }.to_string();
            }
        }
        
        analytics.last_update = Date::now();
    }

    /// Update time-based metrics (errors per minute, etc.)
    fn update_time_based_metrics(analytics: &mut LiveAnalytics) {
        let now = Date::now();
        let minute_ago = now - 60000.0; // 1 minute in milliseconds
        
        // Count errors and warnings in the last minute
        let mut errors_in_minute = 0;
        let mut warnings_in_minute = 0;
        
        for entry in &analytics.recent_entries {
            if let Ok(timestamp) = entry.timestamp.parse::<f64>() {
                if timestamp >= minute_ago {
                    match entry.level.to_uppercase().as_str() {
                        "ERROR" => errors_in_minute += 1,
                        "WARN" | "WARNING" => warnings_in_minute += 1,
                        _ => {}
                    }
                } else {
                    break; // Entries are ordered by recency
                }
            }
        }
        
        analytics.errors_per_minute.push(errors_in_minute as f64);
        analytics.warnings_per_minute.push(warnings_in_minute as f64);
        
        // Keep only the last 60 data points (1 hour of minute-by-minute data)
        if analytics.errors_per_minute.len() > 60 {
            analytics.errors_per_minute.remove(0);
        }
        if analytics.warnings_per_minute.len() > 60 {
            analytics.warnings_per_minute.remove(0);
        }
    }
}

/// Filter configuration for real-time log streams
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamingFilter {
    pub levels: Vec<String>,
    pub sources: Vec<String>,
    pub keywords: Vec<String>,
    pub exclude_keywords: Vec<String>,
    pub time_range_minutes: Option<u32>,
}

/// Real-time search functionality
#[wasm_bindgen]
pub struct LiveSearch {
    filters: StreamingFilter,
    regex_cache: HashMap<String, js_sys::RegExp>,
}

#[wasm_bindgen]
impl LiveSearch {
    /// Create a new live search instance
    #[wasm_bindgen(constructor)]
    pub fn new() -> LiveSearch {
        LiveSearch {
            filters: StreamingFilter {
                levels: vec![],
                sources: vec![],
                keywords: vec![],
                exclude_keywords: vec![],
                time_range_minutes: None,
            },
            regex_cache: HashMap::new(),
        }
    }

    /// Set log level filters
    #[wasm_bindgen]
    pub fn set_level_filter(&mut self, levels: Vec<JsValue>) {
        self.filters.levels = levels.into_iter()
            .filter_map(|v| v.as_string())
            .collect();
    }

    /// Set source filters
    #[wasm_bindgen]
    pub fn set_source_filter(&mut self, sources: Vec<JsValue>) {
        self.filters.sources = sources.into_iter()
            .filter_map(|v| v.as_string())
            .collect();
    }

    /// Set keyword filters
    #[wasm_bindgen]
    pub fn set_keyword_filter(&mut self, keywords: Vec<JsValue>) {
        self.filters.keywords = keywords.into_iter()
            .filter_map(|v| v.as_string())
            .collect();
    }

    /// Set exclude keyword filters
    #[wasm_bindgen]
    pub fn set_exclude_filter(&mut self, exclude_keywords: Vec<JsValue>) {
        self.filters.exclude_keywords = exclude_keywords.into_iter()
            .filter_map(|v| v.as_string())
            .collect();
    }

    /// Filter streaming entries based on current criteria
    #[wasm_bindgen]
    pub fn filter_entries(&self, entries: JsValue) -> Result<JsValue, JsValue> {
        let entries: Vec<StreamingLogEntry> = serde_wasm_bindgen::from_value(entries)?;
        
        let filtered: Vec<StreamingLogEntry> = entries.into_iter()
            .filter(|entry| self.matches_filters(entry))
            .collect();
        
        Ok(serde_wasm_bindgen::to_value(&filtered)?)
    }

    /// Check if an entry matches the current filters
    fn matches_filters(&self, entry: &StreamingLogEntry) -> bool {
        // Level filter
        if !self.filters.levels.is_empty() && !self.filters.levels.contains(&entry.level) {
            return false;
        }
        
        // Source filter
        if !self.filters.sources.is_empty() && !self.filters.sources.contains(&entry.source) {
            return false;
        }
        
        // Keyword filter
        if !self.filters.keywords.is_empty() {
            let message_lower = entry.message.to_lowercase();
            if !self.filters.keywords.iter().any(|keyword| {
                message_lower.contains(&keyword.to_lowercase())
            }) {
                return false;
            }
        }
        
        // Exclude keyword filter
        if !self.filters.exclude_keywords.is_empty() {
            let message_lower = entry.message.to_lowercase();
            if self.filters.exclude_keywords.iter().any(|keyword| {
                message_lower.contains(&keyword.to_lowercase())
            }) {
                return false;
            }
        }
        
        true
    }

    /// Get current filter configuration
    #[wasm_bindgen]
    pub fn get_filters(&self) -> Result<JsValue, JsValue> {
        Ok(serde_wasm_bindgen::to_value(&self.filters)?)
    }
}

/// Real-time chart data generator
#[wasm_bindgen]
pub struct ChartDataGenerator;

#[wasm_bindgen]
impl ChartDataGenerator {
    /// Generate time series data for log levels
    #[wasm_bindgen]
    pub fn generate_level_timeseries(analytics: JsValue) -> Result<JsValue, JsValue> {
        let analytics: LiveAnalytics = serde_wasm_bindgen::from_value(analytics)?;
        
        // Create time series data from recent entries
        let mut time_buckets = HashMap::new();
        let now = Date::now();
        
        for entry in &analytics.recent_entries {
            if let Ok(timestamp) = entry.timestamp.parse::<f64>() {
                let bucket = ((now - timestamp) / 60000.0).floor() as i32; // Minutes ago
                if bucket < 60 { // Last hour only
                    let bucket_data = time_buckets.entry(bucket).or_insert_with(HashMap::new);
                    *bucket_data.entry(&entry.level).or_insert(0) += 1;
                }
            }
        }
        
        Ok(serde_wasm_bindgen::to_value(&time_buckets)?)
    }

    /// Generate pie chart data for log levels
    #[wasm_bindgen]
    pub fn generate_level_pie_data(analytics: JsValue) -> Result<JsValue, JsValue> {
        let analytics: LiveAnalytics = serde_wasm_bindgen::from_value(analytics)?;
        
        let pie_data: Vec<_> = analytics.entries_by_level.iter()
            .map(|(level, count)| {
                serde_json::json!({
                    "label": level,
                    "value": count,
                    "percentage": (*count as f64 / analytics.total_entries as f64 * 100.0).round()
                })
            })
            .collect();
            
        Ok(serde_wasm_bindgen::to_value(&pie_data)?)
    }

    /// Generate source distribution data
    #[wasm_bindgen]
    pub fn generate_source_distribution(analytics: JsValue) -> Result<JsValue, JsValue> {
        let analytics: LiveAnalytics = serde_wasm_bindgen::from_value(analytics)?;
        
        let source_data: Vec<_> = analytics.entries_by_source.iter()
            .map(|(source, count)| {
                serde_json::json!({
                    "source": source,
                    "count": count,
                    "percentage": (*count as f64 / analytics.total_entries as f64 * 100.0).round()
                })
            })
            .collect();
            
        Ok(serde_wasm_bindgen::to_value(&source_data)?)
    }
}

// Export for JavaScript usage
#[wasm_bindgen]
pub fn init_streaming() {
    console::log_1(&"LogLens streaming module initialized".into());
}