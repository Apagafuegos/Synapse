use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tokio::time::timeout;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CircuitBreakerState {
    Closed,   // Normal operation
    Open,     // Circuit is open, failing fast
    HalfOpen, // Testing if service is back
}

#[derive(Debug)]
pub struct CircuitBreakerConfig {
    pub failure_threshold: u32,
    pub success_threshold: u32,
    pub timeout_duration: Duration,
    pub reset_timeout: Duration,
}

impl Default for CircuitBreakerConfig {
    fn default() -> Self {
        Self {
            failure_threshold: 5,
            success_threshold: 3,
            timeout_duration: Duration::from_secs(10),
            reset_timeout: Duration::from_secs(60),
        }
    }
}

#[derive(Debug)]
struct CircuitBreakerMetrics {
    failure_count: AtomicU64,
    success_count: AtomicU64,
    last_failure_time: Mutex<Option<Instant>>,
    state: Mutex<CircuitBreakerState>,
}

#[derive(Debug)]
pub struct CircuitBreaker {
    config: CircuitBreakerConfig,
    metrics: CircuitBreakerMetrics,
    name: String,
}

#[derive(Debug)]
pub enum CircuitBreakerError {
    CircuitOpen(String),
    Timeout(String),
    ServiceError(String),
}

impl std::fmt::Display for CircuitBreakerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CircuitBreakerError::CircuitOpen(service) => {
                write!(f, "Circuit breaker is open for service: {}", service)
            }
            CircuitBreakerError::Timeout(service) => {
                write!(f, "Timeout calling service: {}", service)
            }
            CircuitBreakerError::ServiceError(err) => {
                write!(f, "Service error: {}", err)
            }
        }
    }
}

impl std::error::Error for CircuitBreakerError {}

impl CircuitBreaker {
    pub fn new(name: String, config: CircuitBreakerConfig) -> Self {
        Self {
            name,
            config,
            metrics: CircuitBreakerMetrics {
                failure_count: AtomicU64::new(0),
                success_count: AtomicU64::new(0),
                last_failure_time: Mutex::new(None),
                state: Mutex::new(CircuitBreakerState::Closed),
            },
        }
    }

    pub fn with_default_config(name: String) -> Self {
        Self::new(name, CircuitBreakerConfig::default())
    }

    pub async fn call<F, Fut, T, E>(&self, operation: F) -> Result<T, CircuitBreakerError>
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = Result<T, E>>,
        E: std::error::Error + Send + Sync + 'static,
    {
        // Check if circuit should be opened or can be attempted
        if !self.can_attempt() {
            return Err(CircuitBreakerError::CircuitOpen(self.name.clone()));
        }

        // Execute operation with timeout
        let result = timeout(self.config.timeout_duration, operation()).await;

        match result {
            Ok(Ok(success)) => {
                self.on_success();
                Ok(success)
            }
            Ok(Err(error)) => {
                self.on_failure();
                Err(CircuitBreakerError::ServiceError(error.to_string()))
            }
            Err(_) => {
                self.on_failure();
                Err(CircuitBreakerError::Timeout(self.name.clone()))
            }
        }
    }

    fn can_attempt(&self) -> bool {
        let state = *self.metrics.state.lock().unwrap();

        match state {
            CircuitBreakerState::Closed => true,
            CircuitBreakerState::Open => {
                // Check if we should transition to half-open
                if let Some(last_failure) = *self.metrics.last_failure_time.lock().unwrap() {
                    if last_failure.elapsed() >= self.config.reset_timeout {
                        tracing::info!("Circuit breaker {} transitioning to half-open", self.name);
                        *self.metrics.state.lock().unwrap() = CircuitBreakerState::HalfOpen;
                        self.metrics.success_count.store(0, Ordering::Relaxed);
                        return true;
                    }
                }
                false
            }
            CircuitBreakerState::HalfOpen => true,
        }
    }

    fn on_success(&self) {
        let state = *self.metrics.state.lock().unwrap();

        match state {
            CircuitBreakerState::Closed => {
                // Reset failure count on success in closed state
                self.metrics.failure_count.store(0, Ordering::Relaxed);
            }
            CircuitBreakerState::HalfOpen => {
                let success_count = self.metrics.success_count.fetch_add(1, Ordering::Relaxed) + 1;
                if success_count >= self.config.success_threshold as u64 {
                    tracing::info!("Circuit breaker {} transitioning to closed", self.name);
                    *self.metrics.state.lock().unwrap() = CircuitBreakerState::Closed;
                    self.metrics.failure_count.store(0, Ordering::Relaxed);
                    self.metrics.success_count.store(0, Ordering::Relaxed);
                }
            }
            CircuitBreakerState::Open => {
                // This shouldn't happen, but reset if it does
                tracing::warn!(
                    "Circuit breaker {} got success while open - resetting",
                    self.name
                );
                *self.metrics.state.lock().unwrap() = CircuitBreakerState::Closed;
                self.metrics.failure_count.store(0, Ordering::Relaxed);
                self.metrics.success_count.store(0, Ordering::Relaxed);
            }
        }
    }

    fn on_failure(&self) {
        let state = *self.metrics.state.lock().unwrap();
        let failure_count = self.metrics.failure_count.fetch_add(1, Ordering::Relaxed) + 1;

        match state {
            CircuitBreakerState::Closed => {
                if failure_count >= self.config.failure_threshold as u64 {
                    tracing::warn!(
                        "Circuit breaker {} opening due to {} failures",
                        self.name,
                        failure_count
                    );
                    *self.metrics.state.lock().unwrap() = CircuitBreakerState::Open;
                    *self.metrics.last_failure_time.lock().unwrap() = Some(Instant::now());
                }
            }
            CircuitBreakerState::HalfOpen => {
                tracing::warn!(
                    "Circuit breaker {} reopening after failure in half-open state",
                    self.name
                );
                *self.metrics.state.lock().unwrap() = CircuitBreakerState::Open;
                *self.metrics.last_failure_time.lock().unwrap() = Some(Instant::now());
                self.metrics.failure_count.store(1, Ordering::Relaxed);
                self.metrics.success_count.store(0, Ordering::Relaxed);
            }
            CircuitBreakerState::Open => {
                // Update last failure time
                *self.metrics.last_failure_time.lock().unwrap() = Some(Instant::now());
            }
        }
    }

    pub fn get_state(&self) -> CircuitBreakerState {
        *self.metrics.state.lock().unwrap()
    }

    pub fn get_failure_count(&self) -> u64 {
        self.metrics.failure_count.load(Ordering::Relaxed)
    }

    pub fn get_success_count(&self) -> u64 {
        self.metrics.success_count.load(Ordering::Relaxed)
    }

    pub fn reset(&self) {
        tracing::info!("Manually resetting circuit breaker {}", self.name);
        *self.metrics.state.lock().unwrap() = CircuitBreakerState::Closed;
        self.metrics.failure_count.store(0, Ordering::Relaxed);
        self.metrics.success_count.store(0, Ordering::Relaxed);
        *self.metrics.last_failure_time.lock().unwrap() = None;
    }
}

#[derive(Debug)]
pub struct CircuitBreakerRegistry {
    breakers: Mutex<std::collections::HashMap<String, Arc<CircuitBreaker>>>,
}

impl CircuitBreakerRegistry {
    pub fn new() -> Self {
        Self {
            breakers: Mutex::new(std::collections::HashMap::new()),
        }
    }

    pub fn get_or_create(
        &self,
        name: &str,
        config: Option<CircuitBreakerConfig>,
    ) -> Arc<CircuitBreaker> {
        let mut breakers = self.breakers.lock().unwrap();

        if let Some(breaker) = breakers.get(name) {
            return breaker.clone();
        }

        let breaker = Arc::new(CircuitBreaker::new(
            name.to_string(),
            config.unwrap_or_default(),
        ));
        breakers.insert(name.to_string(), breaker.clone());

        tracing::info!("Created new circuit breaker: {}", name);
        breaker
    }

    pub fn get_breaker(&self, name: &str) -> Option<Arc<CircuitBreaker>> {
        self.breakers.lock().unwrap().get(name).cloned()
    }

    pub fn list_breakers(&self) -> Vec<(String, CircuitBreakerState, u64, u64)> {
        self.breakers
            .lock()
            .unwrap()
            .iter()
            .map(|(name, breaker)| {
                (
                    name.clone(),
                    breaker.get_state(),
                    breaker.get_failure_count(),
                    breaker.get_success_count(),
                )
            })
            .collect()
    }
}

impl Default for CircuitBreakerRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::sleep;

    // Simple error type for testing
    #[derive(Debug)]
    struct TestError(String);

    impl std::fmt::Display for TestError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{}", self.0)
        }
    }

    impl std::error::Error for TestError {}

    #[tokio::test]
    async fn test_circuit_breaker_closed_state() {
        let breaker = CircuitBreaker::with_default_config("test".to_string());

        // Successful calls should keep circuit closed
        let result = breaker.call(|| async { Ok::<_, TestError>("success") }).await;
        assert!(result.is_ok());
        assert_eq!(breaker.get_state(), CircuitBreakerState::Closed);
    }

    #[tokio::test]
    async fn test_circuit_breaker_opens_on_failures() {
        let config = CircuitBreakerConfig {
            failure_threshold: 2,
            ..Default::default()
        };
        let breaker = CircuitBreaker::new("test".to_string(), config);

        // The error must be `Send + Sync` to be safely passed between threads.
        let result = breaker
            .call(|| async { Err::<String, TestError>(TestError("error".to_string())) })
            .await;
        assert!(result.is_err());
        assert_eq!(breaker.get_state(), CircuitBreakerState::Closed);

        let result = breaker
            .call(|| async { Err::<String, TestError>(TestError("error".to_string())) })
            .await;
        assert!(result.is_err());
        assert_eq!(breaker.get_state(), CircuitBreakerState::Open);

        let result = breaker
            .call(|| async { Ok::<_, TestError>("success") })
            .await;
        assert!(matches!(result, Err(CircuitBreakerError::CircuitOpen(_))));
    }

    #[tokio::test]
    async fn test_circuit_breaker_half_open_recovery() {
        let config = CircuitBreakerConfig {
            failure_threshold: 1,
            success_threshold: 2,
            reset_timeout: Duration::from_millis(10),
            ..Default::default()
        };
        let breaker = CircuitBreaker::new("test".to_string(), config);

        // Trigger circuit open
        let _ = breaker.call(|| async { Err::<String, TestError>(TestError("error".to_string())) }).await;
        assert_eq!(breaker.get_state(), CircuitBreakerState::Open);

        // Wait for reset timeout
        sleep(Duration::from_millis(15)).await;

        // First success in half-open state
        let result = breaker.call(|| async { Ok::<_, TestError>("success") }).await;
        assert!(result.is_ok());
        assert_eq!(breaker.get_state(), CircuitBreakerState::HalfOpen);

        // Second success should close circuit
        let result = breaker.call(|| async { Ok::<_, TestError>("success") }).await;
        assert!(result.is_ok());
        assert_eq!(breaker.get_state(), CircuitBreakerState::Closed);
    }

    #[tokio::test]
    async fn test_registry() {
        let registry = CircuitBreakerRegistry::new();

        let breaker1 = registry.get_or_create("test1", None);
        let breaker2 = registry.get_or_create("test1", None);

        // Should return the same instance
        assert!(Arc::ptr_eq(&breaker1, &breaker2));

        let breakers = registry.list_breakers();
        assert_eq!(breakers.len(), 1);
        assert_eq!(breakers[0].0, "test1");
    }
}
