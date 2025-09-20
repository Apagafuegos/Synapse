use loglens::ai::ProviderRegistry;
use loglens::config::ConfigManager;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔑 Testing OpenRouter API Key Setup");
    println!("=====================================");

    // Try to get API key from environment
    let api_key = std::env::var("OPENROUTER_API_KEY")
        .unwrap_or_else(|_| {
            println!("⚠️  OPENROUTER_API_KEY environment variable not found");
            println!("   Trying config file...");
            String::new()
        });

    if api_key.is_empty() {
        // Try config file
        let mut config_manager = ConfigManager::new()?;
        match config_manager.get_config() {
            Ok(config) => {
                if let Some(openrouter_config) = config.providers.get("openrouter") {
                    println!("✅ Found OpenRouter config in config file");
                    println!("   Model: {}", openrouter_config.model);
                    if let Some(key) = &openrouter_config.api_key {
                        println!("   API Key: {}...", &key[..key.len().min(20)]);
                    } else {
                        println!("   API Key: Not set");
                    }
                } else {
                    println!("❌ No OpenRouter configuration found");
                    println!("\n📋 To set up your API key:");
                    println!("1. Get API key from https://openrouter.ai");
                    println!("2. Set environment variable:");
                    println!("   export OPENROUTER_API_KEY=\"sk-or-v1-your-key\"");
                    println!("3. Or add to ~/.config/loglens/config.toml");
                    return Ok(());
                }
            }
            Err(e) => {
                println!("❌ Failed to read config: {}", e);
                return Ok(());
            }
        }
    } else {
        println!("✅ Found API key in environment variable");
        println!("   API Key: {}...", &api_key[..api_key.len().min(20)]);
    }

    // Test provider registry
    println!("\n🧪 Testing Provider Registry...");
    let config_manager = ConfigManager::new()?;
    let mut registry = ProviderRegistry::new(config_manager.clone())?;

    // Test OpenRouter provider specifically
    println!("\n🔍 Testing OpenRouter Provider...");
    match registry.test_provider("openrouter").await {
        Ok(health) => {
            println!("✅ OpenRouter provider test completed:");
            println!("   Status: {}", if health.is_healthy { "Healthy ✅" } else { "Unhealthy ❌" });
            if let Some(response_time) = health.response_time_ms {
                println!("   Response Time: {}ms", response_time);
            }
            if let Some(error) = &health.error_message {
                println!("   Error: {}", error);
            }
            if !health.available_models.is_empty() {
                println!("   Available Models: {}", health.available_models.join(", "));
            }
        }
        Err(e) => {
            println!("❌ OpenRouter provider test failed: {}", e);
            println!("\n📋 Troubleshooting:");
            println!("1. Check your API key is correct");
            println!("2. Ensure you have credits on OpenRouter");
            println!("3. Verify network connectivity");
        }
    }

    println!("\n🎯 Next Steps:");
    println!("1. If test passed, your API key is working!");
    println!("2. Try AI analysis with a log file:");
    println!("   echo 'ERROR: Something went wrong' > test.log");
    println!("   cargo run --release --features tui --bin loglens -- ai analyze --input test.log");
    println!("3. Or run process monitoring:");
    println!("   cargo run --release --features tui --bin loglens -- run echo 'test' --follow");

    Ok(())
}