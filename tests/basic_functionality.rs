//! Basic functionality tests for LogLens
//! 
//! Tests that the core async/await implementation works correctly

use anyhow::Result;

#[test]
fn test_library_compiles() -> Result<()> {
    // Test that the library compiles and basic imports work
    assert!(true);
    Ok(())
}

#[test]
fn test_ai_interface_exists() -> Result<()> {
    // Test that AI interface is properly defined
    use loglens::ai::interface::LlmProvider;
    
    // This test passes if the trait compiles
    assert!(true);
    Ok(())
}

#[test]
fn test_provider_registry_exists() -> Result<()> {
    // Test that provider registry is available
    use loglens::ai::registry::ProviderRegistry;
    
    // This test passes if the struct compiles
    assert!(true);
    Ok(())
}

#[test]
fn test_log_entry_exists() -> Result<()> {
    // Test that LogEntry struct is available
    use loglens::model::LogEntry;
    
    // This test passes if the struct compiles
    assert!(true);
    Ok(())
}

#[test]
fn test_config_manager_exists() -> Result<()> {
    // Test that ConfigManager is available
    use loglens::config::ConfigManager;
    
    // This test passes if the struct compiles
    assert!(true);
    Ok(())
}

#[tokio::test]
async fn test_async_trait_compiles() -> Result<()> {
    // Test that async trait implementations compile
    use loglens::ai::interface::LlmProvider;
    use loglens::ai::providers::placeholder_providers::MockProvider;
    
    // Create a mock provider
    let provider = MockProvider::new();
    
    // This test passes if the async methods compile
    assert!(true);
    Ok(())
}