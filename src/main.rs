use loglens::cli::{Cli, Commands, ConfigCommands, AiCommands};
use loglens::config::ConfigManager;
use loglens::ai::analyze_logs_with_ai;

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
                match analyze_logs_with_ai(
                    &input_path,
                    provider.clone(),
                    model.clone(),
                    depth.clone(),
                    *max_context
                ).await {
                    Ok(analysis) => {
                        println!("\n{}", analysis);
                    }
                    Err(e) => {
                        return Err(format!("AI analysis failed: {}", e));
                    }
                }
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
            let config = config_manager.get_config()
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