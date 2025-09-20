use loglens::cli::{Cli, Commands, ConfigCommands, AiCommands};
use loglens::config::{ConfigManager, AnalysisDepth};
use loglens::ai::{LlmProvider, ProviderRegistry};
use loglens::ai::interface::{LogAnalysisRequest, AnalysisFocus};
use loglens::model::LogEntry;
use loglens::parser::{ParseResult, ParserRegistry};
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
                println!("\n🤖 AI Analysis Results:");
                println!("═════════════════════════════════════");

                // Perform real AI analysis using the provider system
                match perform_ai_analysis(&input_path, &provider, &model, &depth, &focus, max_context).await {
                    Ok(analysis_response) => {
                        // Display real AI analysis results
                        println!("📊 AI-Powered Log Analysis Report");
                        println!("═════════════════════════════════════");
                        println!("🔍 Provider: {} | Model: {}",
                            analysis_response.provider,
                            analysis_response.model);

                        // Display summary
                        let summary = &analysis_response.summary;
                        println!("\n📈 Summary:");
                        println!("   • Overall Status: {:?}", summary.overall_status);
                        println!("   • Errors: {} | Warnings: {}", summary.error_count, summary.warning_count);
                        if let Some(time_range) = &summary.time_range {
                            println!("   • Time Range: {} to {}", time_range.0, time_range.1);
                        }

                        // Display key findings
                        if !summary.key_findings.is_empty() {
                            println!("\n🔍 Key Findings:");
                            for (i, finding) in summary.key_findings.iter().enumerate() {
                                println!("   {}. {}", i + 1, finding);
                            }
                        }

                        // Display affected systems
                        if !summary.affected_systems.is_empty() {
                            println!("\n🏗️ Affected Systems: {}", summary.affected_systems.join(", "));
                        }

                        // Display detailed analysis if available
                        if let Some(detailed) = &analysis_response.detailed_analysis {
                            if let Some(failure_analysis) = &detailed.failure_analysis {
                                println!("\n🚨 Failure Analysis:");
                                for root_cause in &failure_analysis.root_causes {
                                    println!("   • {} (confidence: {:.0}%)",
                                        root_cause.description, root_cause.confidence * 100.0);
                                    if !root_cause.supporting_evidence.is_empty() {
                                        println!("     Evidence: {}", root_cause.supporting_evidence.join(", "));
                                    }
                                }

                                let impact = &failure_analysis.impact_assessment;
                                println!("\n📊 Impact Assessment:");
                                println!("   • Severity: {:?}", impact.severity);
                                println!("   • User Impact: {}", impact.user_impact);
                                println!("   • Business Impact: {}", impact.business_impact);
                            }
                        }

                        // Display recommendations
                        if let Some(recommendations) = &analysis_response.recommendations {
                            println!("\n💡 AI-Generated Recommendations:");
                            for (i, rec) in recommendations.iter().enumerate() {
                                println!("   {}. {} (Priority: {:?})", i + 1, rec.title, rec.priority);
                                println!("      {}", rec.description);
                                if !rec.implementation_steps.is_empty() {
                                    println!("      Steps: {}", rec.implementation_steps.join(" → "));
                                }
                                if !rec.expected_outcome.is_empty() {
                                    println!("      Expected outcome: {}", rec.expected_outcome);
                                }
                            }
                        }

                        // Display processing information
                        if let Some(token_usage) = &analysis_response.token_usage {
                            println!("\n⚡ Processing Info:");
                            println!("   • Processing time: {}ms", analysis_response.processing_time_ms);
                            println!("   • Tokens used: {} total ({} prompt + {} completion)",
                                token_usage.total_tokens, token_usage.prompt_tokens, token_usage.completion_tokens);
                            if let Some(cost) = token_usage.estimated_cost_usd {
                                println!("   • Estimated cost: ${:.4}", cost);
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("❌ AI Analysis Failed: {}", e);
                        eprintln!("   • Check your configuration and API keys");
                        eprintln!("   • Verify network connectivity");
                        eprintln!("   • Ensure the provider is properly configured");
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

async fn perform_ai_analysis(
    input_path: &str,
    provider_name: &Option<String>,
    model_name: &Option<String>,
    depth: &str,
    focus_areas: &Option<Vec<String>>,
    max_context: usize,
) -> Result<loglens::ai::interface::LogAnalysisResponse, String> {
    // Load configuration
    let mut config_manager = ConfigManager::new()
        .map_err(|e| format!("Failed to create config manager: {}", e))?;

    // Initialize provider registry
    let mut registry = ProviderRegistry::new(config_manager.clone())
        .map_err(|e| format!("Failed to create provider registry: {}", e))?;

    // Get the provider (default to openrouter if not specified)
    let provider_name = provider_name.as_deref().unwrap_or("openrouter");
    let provider = registry.get_provider(provider_name)
        .map_err(|e| format!("Failed to get provider '{}': {}", provider_name, e))?;

    // Read and parse log file
    let log_content = std::fs::read_to_string(input_path)
        .map_err(|e| format!("Failed to read log file: {}", e))?;

    // Parse logs into LogEntry structures
    let parser_registry = ParserRegistry::new();
    let mut log_entries = Vec::new();

    // Detect best parser for the content
    let sample_lines: Vec<&str> = log_content.lines().take(10).collect();
    let parser = parser_registry.detect_parser(&sample_lines)
        .unwrap_or_else(|| parser_registry.get_parser("text").unwrap());

    // Parse each line into log entries
    for (line_num, line) in log_content.lines().enumerate() {
        let context = loglens::parser::ParseContext {
            line_number: line_num + 1,
            file_path: Some(input_path.to_string()),
            previous_entry: log_entries.last().cloned(),
            ai_analysis_enabled: true,
        };

        match parser.parse_line(line, &context) {
            ParseResult::Success(entry) => log_entries.push(entry),
            ParseResult::Skip => continue,
            ParseResult::Error(_) => continue, // Skip malformed lines
            ParseResult::RawWithMetadata { line, line_number: _, timestamp_hint, level_hint } => {
                // Create a basic log entry from raw data
                let entry = LogEntry::new(
                    timestamp_hint.unwrap_or_else(|| chrono::Utc::now()),
                    level_hint.unwrap_or(loglens::model::LogLevel::Info),
                    line.clone(),
                    line, // raw_line is the same as message for basic parsing
                );
                log_entries.push(entry);
            }
        }
    }

    // Convert depth string to AnalysisDepth enum
    let analysis_depth = match depth {
        "basic" => AnalysisDepth::Basic,
        "detailed" => AnalysisDepth::Detailed,
        "comprehensive" => AnalysisDepth::Comprehensive,
        _ => AnalysisDepth::Detailed, // default
    };

    // Convert focus areas to AnalysisFocus enum
    let focus_areas_enum = if let Some(focus) = focus_areas {
        focus.iter().filter_map(|f| match f.as_str() {
            "errors" => Some(AnalysisFocus::Errors),
            "performance" => Some(AnalysisFocus::Performance),
            "security" => Some(AnalysisFocus::Security),
            "configuration" => Some(AnalysisFocus::Configuration),
            "user_activity" => Some(AnalysisFocus::UserActivity),
            "system_events" => Some(AnalysisFocus::SystemEvents),
            custom => Some(AnalysisFocus::Custom(custom.to_string())),
        }).collect()
    } else {
        vec![AnalysisFocus::Errors, AnalysisFocus::Performance] // default focus areas
    };

    // Create analysis request
    let analysis_request = LogAnalysisRequest {
        log_entries,
        analysis_depth,
        focus_areas: focus_areas_enum,
        output_format: loglens::ai::interface::OutputFormat::HumanReadable,
        include_context: true,
        max_context_entries: max_context,
        custom_prompt: None,
        provider_override: model_name.clone(),
    };

    // Perform AI analysis
    provider.analyze_logs(analysis_request).await
        .map_err(|e| format!("AI analysis failed: {}", e))
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