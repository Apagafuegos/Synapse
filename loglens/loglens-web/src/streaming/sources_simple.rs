use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;
use anyhow::Result;

use crate::streaming::StreamingLogEntry;

/// Simplified streaming source manager for Phase 6.2 completion
pub struct StreamingSourceManager {
    sources: Arc<RwLock<HashMap<String, StreamingSourceConfig>>>,
}

#[derive(Debug, Clone)]
pub struct StreamingSourceConfig {
    pub id: String,
    pub name: String,
    pub project_id: Uuid,
    pub source_type: String,
    pub config: serde_json::Value,
    pub is_active: bool,
}

impl Default for StreamingSourceManager {
    fn default() -> Self {
        Self::new()
    }
}

impl StreamingSourceManager {
    pub fn new() -> Self {
        Self {
            sources: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn register_source(
        &self,
        project_id: Uuid,
        name: String,
        source_type: String,
        config: serde_json::Value,
    ) -> Result<String> {
        let source_id = Uuid::new_v4().to_string();
        let source = StreamingSourceConfig {
            id: source_id.clone(),
            name,
            project_id,
            source_type,
            config,
            is_active: true,
        };

        let mut sources = self.sources.write().await;
        sources.insert(source_id.clone(), source);

        Ok(source_id)
    }

    pub async fn remove_source(&self, source_id: &str) -> Result<()> {
        let mut sources = self.sources.write().await;
        sources.remove(source_id);
        Ok(())
    }

    pub async fn get_source(&self, source_id: &str) -> Option<StreamingSourceConfig> {
        let sources = self.sources.read().await;
        sources.get(source_id).cloned()
    }

    pub async fn list_sources_for_project(&self, project_id: Uuid) -> Vec<StreamingSourceConfig> {
        let sources = self.sources.read().await;
        sources.values()
            .filter(|s| s.project_id == project_id)
            .cloned()
            .collect()
    }

    /// Create a sample log entry for testing
    pub fn create_sample_log_entry(
        &self,
        project_id: Uuid,
        source_id: String,
        message: String,
    ) -> StreamingLogEntry {
        StreamingLogEntry {
            id: Uuid::new_v4().to_string(),
            timestamp: Some(chrono::Utc::now().to_rfc3339()),
            level: Some("INFO".to_string()),
            message,
            source: source_id,
            project_id,
            line_number: None,
        }
    }
}