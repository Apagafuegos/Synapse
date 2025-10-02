use anyhow::Result;
use clap::{Arg, Command};
use loglens_core::{create_provider, Analyzer, Config, parse_log_lines, filter_logs_by_level, generate_report, OutputFormat, input::read_log_file};
use std::io::{self, Read};
use tracing::{error, warn, info, debug};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing with default configuration
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    info!("Starting LogLens CLI");
    
    let matches = Command::new("loglens")
        .version("0.1.0")
        .about("AI-powered log analysis tool")
        .arg(
            Arg::new("file")
                .short('f')
                .long("file")
                .value_name("FILE")
                .help("Log file to analyze")
        )
        .arg(
            Arg::new("exec")
                .short('e')
                .long("exec")
                .value_name("COMMAND")
                .help("Execute command and analyze its output")
        )
        .arg(
            Arg::new("level")
                .short('l')
                .long("level")
                .value_name("LEVEL")
                .help("Minimum log level to analyze")
                .default_value("ERROR")
        )
        .arg(
            Arg::new("provider")
                .short('p')
                .long("provider")
                .value_name("PROVIDER")
                .help("AI provider to use")
                .default_value("openrouter")
        )
        .arg(
            Arg::new("api-key")
                .long("api-key")
                .value_name("KEY")
                .help("API key for the AI provider")
        )
        .arg(
            Arg::new("output")
                .short('o')
                .long("output")
                .value_name("FORMAT")
                .help("Output format")
                .default_value("console")
        )
        .arg(
            Arg::new("mcp-server")
                .long("mcp-server")
                .help("Start MCP server mode")
                .action(clap::ArgAction::SetTrue)
        )
        .arg(
            Arg::new("mcp-mode")
                .long("mcp-mode")
                .help("Process JSON input/output for MCP")
                .action(clap::ArgAction::SetTrue)
        )
        .get_matches();

    // Handle MCP server mode
    if matches.get_flag("mcp-server") {
        info!("Starting MCP server mode");
        warn!("MCP server startup not yet implemented");
        // TODO: Implement MCP server startup
        return Ok(());
    }

    // Handle MCP JSON mode
    if matches.get_flag("mcp-mode") {
        info!("Processing MCP JSON input");
        let mut input = String::new();
        match io::stdin().read_to_string(&mut input) {
            Ok(_) => debug!("Read {} bytes from stdin", input.len()),
            Err(e) => {
                error!("Failed to read from stdin: {}", e);
                return Err(e.into());
            }
        }
        // TODO: Implement MCP JSON processing when MCP server is available
        warn!("MCP mode not yet implemented");        eprintln!("{{\"error\": \"MCP mode not yet implemented\"}}");
        return Ok(());
    }

    // Handle standard CLI operations
    let level = matches.get_one::<String>("level").unwrap();
    let provider = matches.get_one::<String>("provider").unwrap();
    let output_format = matches.get_one::<String>("output").unwrap();
    let api_key = matches.get_one::<String>("api-key");

    info!("Starting analysis with provider: {}, level: {}, format: {}", provider, level, output_format);

    let logs = if let Some(file_path) = matches.get_one::<String>("file") {
        info!("Reading log file: {}", file_path);
        match read_log_file(file_path).await {
            Ok(logs) => {
                info!("Successfully read {} log entries", logs.len());
                logs
            }
            Err(e) => {
                error!("Failed to read log file {}: {}", file_path, e);
                return Err(e);
            }
        }
    } else if let Some(command) = matches.get_one::<String>("exec") {
        info!("Executing command: {}", command);
        let output = match std::process::Command::new("sh")
            .arg("-c")
            .arg(command)
            .output()
        {
            Ok(output) => output,
            Err(e) => {
                error!("Failed to execute command '{}': {}", command, e);
                return Err(e.into());
            }
        };
        
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        
        if !output.status.success() {
            warn!("Command exited with status: {}", output.status);
            if !stderr.is_empty() {
                warn!("Command stderr: {}", stderr);
            }
        }
        
        debug!("Command stdout length: {} bytes", stdout.len());
        if !stderr.is_empty() {
            debug!("Command stderr: {}", stderr);
        }
        
        let mut logs = Vec::new();
        logs.extend(stdout.lines().map(|s| s.to_string()));
        logs.extend(stderr.lines().map(|s| s.to_string()));
        info!("Collected {} log lines from command output", logs.len());
        logs
    } else {
        // Read from stdin
        info!("Reading log data from stdin");
        let mut input = String::new();
        match io::stdin().read_to_string(&mut input) {
            Ok(size) => {
                debug!("Read {} bytes from stdin", size);
                let logs: Vec<String> = input.lines().map(|s| s.to_string()).collect();
                info!("Collected {} log lines from stdin", logs.len());
                logs
            }
            Err(e) => {
                error!("Failed to read from stdin: {}", e);
                return Err(e.into());
            }
        }
    };

    if logs.is_empty() {
        error!("No log data provided. Use --file, --exec, or pipe data to stdin.");
        std::process::exit(1);
    }

    // Parse logs
    info!("Parsing {} log lines", logs.len());
    let parsed_logs = parse_log_lines(&logs);
    debug!("Parsed {} structured log entries", parsed_logs.len());
    
    // Filter by level
    info!("Filtering logs by level: {}", level);
    let filtered_logs = filter_logs_by_level(parsed_logs, level)?;
    info!("Filtered down to {} log entries", filtered_logs.len());
    
    if filtered_logs.is_empty() {
        error!("No log entries match the specified level filter: {}", level);
        std::process::exit(1);
    }
    
    // Load configuration
    debug!("Loading configuration");
    let config = Config::load()?;
    
    // Get API key
    debug!("Resolving API key for provider: {}", provider);
    let api_key_string = api_key.map(|s| s.to_string()).or_else(|| {
        match provider.as_str() {
            "openrouter" => {
                debug!("Checking OPENROUTER_API_KEY environment variable");
                std::env::var("OPENROUTER_API_KEY").ok()
            }
            "openai" => {
                debug!("Checking OPENAI_API_KEY environment variable");
                std::env::var("OPENAI_API_KEY").ok()
            }
            "claude" => {
                debug!("Checking ANTHROPIC_API_KEY environment variable");
                std::env::var("ANTHROPIC_API_KEY").ok()
            }
            "gemini" => {
                debug!("Checking GEMINI_API_KEY environment variable");
                std::env::var("GEMINI_API_KEY").ok()
            }
            _ => {
                warn!("Unknown provider: {}", provider);
                None
            }
        }
    });
    
    let api_key_str = match api_key_string {
        Some(ref key) => {
            debug!("API key found for provider: {}", provider);
            key.as_str()
        }
        None => {
            error!("No API key provided for provider: {}", provider);
            error!("Set environment variable or use --api-key option");
            std::process::exit(1);
        }
    };
    
    // Create AI provider
    info!("Creating AI provider: {}", provider);
    let ai_provider = create_provider(provider, api_key_str)?;
    
    // Perform analysis
    info!("Starting log analysis with {} entries", filtered_logs.len());
    let mut analyzer = Analyzer::new(ai_provider);
    let analysis = match analyzer.analyze_logs(filtered_logs.clone()).await {
        Ok(analysis) => {
            info!("Analysis completed successfully");
            analysis
        }
        Err(e) => {
            error!("Analysis failed: {}", e);
            return Err(e);
        }
    };
    
    // Generate output
    let output_fmt = match output_format.as_str() {
        "json" => OutputFormat::Json,
        "html" => OutputFormat::Html,
        "markdown" => OutputFormat::Markdown,
        _ => OutputFormat::Console,
    };
    debug!("Using output format: {:?}", output_fmt);
    
    let input_source = if matches.contains_id("file") {
        matches.get_one::<String>("file").unwrap()
    } else if matches.contains_id("exec") {
        "command execution"
    } else {
        "stdin"
    };
    
    info!("Generating report from source: {}", input_source);
    let output = generate_report(analysis, filtered_logs, provider, level, input_source, output_fmt)?;
    print!("{}", output);
    info!("Report generation completed");

    Ok(())
}