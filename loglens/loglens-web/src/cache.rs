use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::{Duration, SystemTime};
use serde::{Deserialize, Serialize};
use tokio::time::interval;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheEntry<T> {
    pub data: T,
    pub created_at: SystemTime,
    pub expires_at: Option<SystemTime>,
    pub access_count: u64,
    pub last_accessed: SystemTime,
}

impl<T> CacheEntry<T> {
    pub fn new(data: T, ttl: Option<Duration>) -> Self {
        let now = SystemTime::now();
        Self {
            data,
            created_at: now,
            expires_at: ttl.map(|d| now + d),
            access_count: 0,
            last_accessed: now,
        }
    }

    pub fn is_expired(&self) -> bool {
        self.expires_at.is_some_and(|exp| SystemTime::now().duration_since(exp).is_ok())
    }

    pub fn access(&mut self) -> &T {
        self.access_count += 1;
        self.last_accessed = SystemTime::now();
        &self.data
    }
}

#[derive(Debug)]
pub struct LRUCache<T> {
    data: Arc<RwLock<HashMap<String, CacheEntry<T>>>>,
    max_size: usize,
    default_ttl: Option<Duration>,
    stats: Arc<RwLock<CacheStats>>,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct CacheStats {
    pub hits: u64,
    pub misses: u64,
    pub evictions: u64,
    pub total_entries: u64,
    pub current_size: usize,
}

impl CacheStats {
    pub fn hit_rate(&self) -> f64 {
        if self.hits + self.misses == 0 {
            0.0
        } else {
            self.hits as f64 / (self.hits + self.misses) as f64
        }
    }
}

impl<T: Clone + Send + Sync + 'static> LRUCache<T> {
    pub fn new(max_size: usize, default_ttl: Option<Duration>) -> Self {
        Self {
            data: Arc::new(RwLock::new(HashMap::new())),
            max_size,
            default_ttl,
            stats: Arc::new(RwLock::new(CacheStats::default())),
        }
    }

    pub fn get(&self, key: &str) -> Option<T> {
        let mut data = self.data.write().unwrap();
        let mut stats = self.stats.write().unwrap();
        
        if let Some(entry) = data.get_mut(key) {
            if entry.is_expired() {
                data.remove(key);
                stats.misses += 1;
                stats.current_size = data.len();
                None
            } else {
                stats.hits += 1;
                Some(entry.access().clone())
            }
        } else {
            stats.misses += 1;
            None
        }
    }

    pub fn put(&self, key: String, value: T, ttl: Option<Duration>) {
        let mut data = self.data.write().unwrap();
        let mut stats = self.stats.write().unwrap();

        let effective_ttl = ttl.or(self.default_ttl);
        let entry = CacheEntry::new(value, effective_ttl);

        // If cache is at capacity, remove least recently used
        if data.len() >= self.max_size && !data.contains_key(&key) {
            if let Some(lru_key) = self.find_lru_key(&data) {
                data.remove(&lru_key);
                stats.evictions += 1;
            }
        }

        let is_new = !data.contains_key(&key);
        data.insert(key, entry);
        
        if is_new {
            stats.total_entries += 1;
        }
        stats.current_size = data.len();
    }

    pub fn remove(&self, key: &str) -> bool {
        let mut data = self.data.write().unwrap();
        let mut stats = self.stats.write().unwrap();
        
        let removed = data.remove(key).is_some();
        stats.current_size = data.len();
        removed
    }

    pub fn clear(&self) {
        let mut data = self.data.write().unwrap();
        let mut stats = self.stats.write().unwrap();
        
        data.clear();
        stats.current_size = 0;
    }

    pub fn stats(&self) -> CacheStats {
        self.stats.read().unwrap().clone()
    }

    pub fn cleanup_expired(&self) -> usize {
        let mut data = self.data.write().unwrap();
        let mut stats = self.stats.write().unwrap();
        
        let initial_size = data.len();
        data.retain(|_, entry| !entry.is_expired());
        let removed = initial_size - data.len();
        
        stats.current_size = data.len();
        removed
    }

    fn find_lru_key(&self, data: &HashMap<String, CacheEntry<T>>) -> Option<String> {
        data.iter()
            .min_by_key(|(_, entry)| entry.last_accessed)
            .map(|(key, _)| key.clone())
    }

    pub fn start_cleanup_task(cache: Arc<Self>) {
        let cache_clone = Arc::clone(&cache);
        tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(300)); // 5 minutes
            
            loop {
                interval.tick().await;
                let removed = cache_clone.cleanup_expired();
                if removed > 0 {
                    tracing::debug!("Cache cleanup removed {} expired entries", removed);
                }
            }
        });
    }
}

// Specialized caches for different data types
pub type AnalysisCache = LRUCache<crate::models::Analysis>;
pub type ResultCache = LRUCache<String>; // For analysis results
pub type MetricsCache = LRUCache<Vec<crate::models::PerformanceMetric>>;
pub type CorrelationCache = LRUCache<Vec<crate::models::ErrorCorrelation>>;

#[derive(Debug)]
pub struct CacheManager {
    pub analysis_cache: Arc<AnalysisCache>,
    pub result_cache: Arc<ResultCache>,
    pub metrics_cache: Arc<MetricsCache>,
    pub correlation_cache: Arc<CorrelationCache>,
}

impl CacheManager {
    pub fn new() -> Self {
        let manager = Self {
            analysis_cache: Arc::new(AnalysisCache::new(1000, Some(Duration::from_secs(3600)))), // 1 hour TTL
            result_cache: Arc::new(ResultCache::new(500, Some(Duration::from_secs(1800)))), // 30 min TTL
            metrics_cache: Arc::new(MetricsCache::new(2000, Some(Duration::from_secs(600)))), // 10 min TTL
            correlation_cache: Arc::new(CorrelationCache::new(1500, Some(Duration::from_secs(900)))), // 15 min TTL
        };

        // Start cleanup tasks
        AnalysisCache::start_cleanup_task(Arc::clone(&manager.analysis_cache));
        ResultCache::start_cleanup_task(Arc::clone(&manager.result_cache));
        MetricsCache::start_cleanup_task(Arc::clone(&manager.metrics_cache));
        CorrelationCache::start_cleanup_task(Arc::clone(&manager.correlation_cache));

        manager
    }

    pub fn get_cache_stats(&self) -> HashMap<String, CacheStats> {
        let mut stats = HashMap::new();
        stats.insert("analysis".to_string(), self.analysis_cache.stats());
        stats.insert("result".to_string(), self.result_cache.stats());
        stats.insert("metrics".to_string(), self.metrics_cache.stats());
        stats.insert("correlation".to_string(), self.correlation_cache.stats());
        stats
    }

    pub fn clear_all_caches(&self) {
        self.analysis_cache.clear();
        self.result_cache.clear();
        self.metrics_cache.clear();
        self.correlation_cache.clear();
        tracing::info!("All caches cleared");
    }
}

impl Default for CacheManager {
    fn default() -> Self {
        Self::new()
    }
}

// Cache keys
pub struct CacheKeys;

impl CacheKeys {
    pub fn analysis_key(project_id: &str, analysis_id: &str) -> String {
        format!("analysis:{}:{}", project_id, analysis_id)
    }

    pub fn result_key(analysis_id: &str) -> String {
        format!("result:{}", analysis_id)
    }

    pub fn metrics_key(analysis_id: &str) -> String {
        format!("metrics:{}", analysis_id)
    }

    pub fn correlation_key(project_id: &str, analysis_id: &str) -> String {
        format!("correlation:{}:{}", project_id, analysis_id)
    }

    pub fn project_analyses_key(project_id: &str) -> String {
        format!("project_analyses:{}", project_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn test_cache_basic_operations() {
        let cache = LRUCache::new(3, Some(Duration::from_secs(1)));
        
        // Test put and get
        cache.put("key1".to_string(), "value1".to_string(), None);
        assert_eq!(cache.get("key1"), Some("value1".to_string()));
        
        // Test miss
        assert_eq!(cache.get("key2"), None);
        
        // Test stats
        let stats = cache.stats();
        assert_eq!(stats.hits, 1);
        assert_eq!(stats.misses, 1);
    }

    #[test]
    fn test_cache_expiration() {
        let cache = LRUCache::new(10, Some(Duration::from_millis(100)));
        
        cache.put("key1".to_string(), "value1".to_string(), None);
        assert_eq!(cache.get("key1"), Some("value1".to_string()));
        
        thread::sleep(Duration::from_millis(150));
        assert_eq!(cache.get("key1"), None);
    }

    #[test]
    fn test_cache_lru_eviction() {
        let cache = LRUCache::new(2, None);
        
        cache.put("key1".to_string(), "value1".to_string(), None);
        cache.put("key2".to_string(), "value2".to_string(), None);
        cache.put("key3".to_string(), "value3".to_string(), None);
        
        // key1 should be evicted
        assert_eq!(cache.get("key1"), None);
        assert_eq!(cache.get("key2"), Some("value2".to_string()));
        assert_eq!(cache.get("key3"), Some("value3".to_string()));
    }
}