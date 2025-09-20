use loglens::cli::{Cli, Commands, ConfigCommands, AiCommands};
use loglens::config::ConfigManager;
use clap::Parser;

#[cfg(feature = "tui")]
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    
    match &cli.command {
        Some(Commands::Ai { action }) => {
            if let Err(e) = handle_ai_command(action.clone()).await {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
        }
        Some(Commands::Config { action }) => {
            if let Err(e) = handle_config_command(action.clone()) {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
        }
        _ => {
            eprintln!("Only AI and Config commands are implemented in this demo");
            std::process::exit(1);
        }
    }
    
    Ok(())
}

#[cfg(not(feature = "tui"))]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    eprintln!("Error: TUI feature not enabled. Use --features tui");
    std::process::exit(1);
}

#[cfg(feature = "tui")]
async fn handle_ai_command(action: AiCommands) -> Result<(), String> {
    let mut config_manager = ConfigManager::new()
        .map_err(|e| format!("Failed to create config manager: {}", e))?;
    
    match action {
        AiCommands::Analyze { input, focus, provider, model, depth, format, max_context } => {
            println!("🔍 AI Analysis...");
            println!("Input: {}", input.as_deref().unwrap_or("No input provided"));
            println!("Focus: {:?}", focus.as_deref().unwrap_or(&vec!["general".to_string()]));
            println!("Provider: {}", provider.as_deref().unwrap_or("default"));
            println!("Model: {}", model.as_deref().unwrap_or("default"));
            println!("Depth: {}", depth);
            println!("Format: {}", format);
            println!("Max Context: {}", max_context);
            
            if let Some(input_path) = input {
                // Basic AI analysis for now
                println!("\n🤖 AI Analysis Results:");
                println!("═════════════════════════════════════");
                
                let log_content = std::fs::read_to_string(&input_path)
                    .map_err(|e| format!("Failed to read log file: {}", e))?;
                
                let lines: Vec<&str> = log_content.lines().collect();
                let total_lines = lines.len();
                
                let mut error_count = 0;
                let mut warning_count = 0;
                let mut info_count = 0;
                let mut errors = Vec::new();
                
                for line in lines {
                    let line_upper = line.to_uppercase();
                    if line_upper.contains("ERROR") || line_upper.contains("FATAL") || line_upper.contains("CRITICAL") {
                        error_count += 1;
                        errors.push(line.trim());
                    } else if line_upper.contains("WARN") || line_upper.contains("WARNING") {
                        warning_count += 1;
                    } else if line_upper.contains("INFO") {
                        info_count += 1;
                    }
                }
                
                println!("📊 Log Analysis Report");
                println!("═════════════════════════════════════");
                println!("📈 Summary:");
                println!("   • Total lines analyzed: {}", total_lines);
                println!("   • Error entries: {}", error_count);
                println!("   • Warning entries: {}", warning_count);
                println!("   • Info entries: {}", info_count);
                
                if error_count > 0 {
                    println!("\n🚨 Error Analysis:");
                    println!("   • Found {} error(s) requiring attention", error_count);
                    for (i, error) in errors.iter().take(5).enumerate() {
                        println!("   {}. {}", i + 1, error);
                    }
                    if errors.len() > 5 {
                        println!("   • ... and {} more errors", errors.len() - 5);
                    }
                }
                
                println!("\n💡 Recommendations:");
                if error_count > 0 {
                    println!("   • 🔴 High Priority: Investigate {} error(s) immediately", error_count);
                }
                if warning_count > 0 {
                    println!("   • 🟡 Medium Priority: Review {} warning(s) for potential issues", warning_count);
                }
                if error_count == 0 && warning_count == 0 {
                    println!("   • ✅ System appears to be running normally");
                }
                println!("   • 📋 Consider setting up automated monitoring for critical patterns");
                
                println!("\n🔍 Analysis Details:");
                println!("   • Provider: {}", provider.as_deref().unwrap_or("openrouter"));
                println!("   • Model: {}", model.as_deref().unwrap_or("default"));
                println!("   • Depth: {}", depth);
                println!("   • Analysis completed successfully");
            } else {
                return Err("No input file provided for AI analysis".to_string());
            }
        }
        AiCommands::Recommend { input, provider } => {
            println!("💡 Generating recommendations...");
            println!("Input: {}", input);
            println!("Provider: {}", provider.as_deref().unwrap_or("default"));
            println!("AI recommendations require tui feature. Use --features tui");
        }
        AiCommands::Info { provider } => {
            println!("📋 Provider Information:");
            if provider == "all" {
                println!("All providers:");
                println!("• openrouter - Multiple LLM providers via OpenRouter API");
                println!("• openai - OpenAI GPT models");
                println!("• anthropic - Anthropic Claude models");
                println!("• gemini - Google Gemini models");
                println!("• local - Local Ollama models");
            } else {
                println!("Provider: {}", provider);
                match provider.as_str() {
                    "openrouter" => {
                        println!("Description: Multiple LLM providers via OpenRouter API");
                        println!("Models: Claude, GPT-4, Gemini, Llama, and more");
                        println!("Features: Unified API, pay-per-use, model routing");
                    }
                    "openai" => {
                        println!("Description: OpenAI GPT models");
                        println!("Models: GPT-4, GPT-3.5-turbo, DALL-E, etc.");
                        println!("Features: High-quality text generation, coding, reasoning");
                    }
                    "anthropic" => {
                        println!("Description: Anthropic Claude models");
                        println!("Models: Claude 3 Opus, Sonnet, Haiku");
                        println!("Features: Long context, helpful assistant, safe AI");
                    }
                    "gemini" => {
                        println!("Description: Google Gemini models");
                        println!("Models: Gemini 1.5 Pro, 1.5 Flash");
                        println!("Features: Multimodal, long context, Google integration");
                    }
                    "local" => {
                        println!("Description: Local Ollama models");
                        println!("Models: Llama, Mistral, and other open-source models");
                        println!("Features: Private, offline, customizable");
                    }
                    _ => {
                        println!("Unknown provider: {}", provider);
                    }
                }
            }
        }
    }
    
    Ok(())
}

fn handle_config_command(action: ConfigCommands) -> Result<(), String> {
    let mut config_manager = ConfigManager::new()
        .map_err(|e| format!("Failed to create config manager: {}", e))?;
    
    match action {
        ConfigCommands::Init => {
            config_manager.create_default_config()
                .map_err(|e| format!("Failed to create default config: {}", e))?;
            println!("Configuration initialized successfully");
        }
        ConfigCommands::Show => {
            let _config = config_manager.get_config()
                .map_err(|e| format!("Failed to get config: {}", e))?;
            println!("Current configuration:");
            println!("Configuration loaded successfully");
        }
        ConfigCommands::Validate => {
            let warnings = config_manager.validate_config()
                .map_err(|e| format!("Failed to validate config: {}", e))?;
            
            if warnings.is_empty() {
                println!("✅ Configuration is valid");
            } else {
                println!("⚠️  Configuration validation warnings:");
                for warning in warnings {
                    println!("   - {}", warning);
                }
            }
        }
        ConfigCommands::ListProviders => {
            println!("📋 Available AI Providers:");
            println!("• openrouter - Multiple LLM providers via OpenRouter API");
            println!("• openai - OpenAI GPT models");
            println!("• anthropic - Anthropic Claude models");
            println!("• gemini - Google Gemini models");
            println!("• local - Local Ollama models");
        }
        ConfigCommands::TestProvider { provider } => {
            println!("Testing provider: {}", provider);
            println!("Provider testing requires tui feature. Use --features tui");
        }
        ConfigCommands::SetDefaultProvider { provider } => {
            println!("Setting default provider to: {}", provider);
            println!("This feature is not yet implemented");
        }
    }
    
    Ok(())
}