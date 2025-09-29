use anyhow::Result;
use clap::{Arg, Command};
use loglens_core::{create_provider, Analyzer, Config, parse_log_lines, filter_logs_by_level, generate_report, OutputFormat, input::read_log_file};
use std::io::{self, Read};

#[tokio::main]
async fn main() -> Result<()> {
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
        println!("Starting MCP server...");
        // TODO: Implement MCP server startup
        return Ok(());
    }

    // Handle MCP JSON mode
    if matches.get_flag("mcp-mode") {
        let mut input = String::new();
        io::stdin().read_to_string(&mut input)?;
        // TODO: Implement MCP JSON processing when MCP server is available
        println!("{{\"error\": \"MCP mode not yet implemented\"}}");
        return Ok(());
    }

    // Handle standard CLI operations
    let level = matches.get_one::<String>("level").unwrap();
    let provider = matches.get_one::<String>("provider").unwrap();
    let output_format = matches.get_one::<String>("output").unwrap();
    let api_key = matches.get_one::<String>("api-key");

    let logs = if let Some(file_path) = matches.get_one::<String>("file") {
        read_log_file(file_path).await?
    } else if let Some(command) = matches.get_one::<String>("exec") {
        let output = std::process::Command::new("sh")
            .arg("-c")
            .arg(command)
            .output()?;
        
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        
        let mut logs = Vec::new();
        logs.extend(stdout.lines().map(|s| s.to_string()));
        logs.extend(stderr.lines().map(|s| s.to_string()));
        logs
    } else {
        // Read from stdin
        let mut input = String::new();
        io::stdin().read_to_string(&mut input)?;
        input.lines().map(|s| s.to_string()).collect()
    };

    if logs.is_empty() {
        eprintln!("No log data provided. Use --file, --exec, or pipe data to stdin.");
        std::process::exit(1);
    }

    // Parse logs
    let parsed_logs = parse_log_lines(&logs);
    
    // Filter by level
    let filtered_logs = filter_logs_by_level(parsed_logs, level)?;
    
    if filtered_logs.is_empty() {
        eprintln!("No log entries match the specified level filter: {}", level);
        std::process::exit(1);
    }
    
    // Load configuration
    let config = Config::load()?;
    
    // Get API key
    let api_key_string = api_key.map(|s| s.to_string()).or_else(|| {
        match provider.as_str() {
            "openrouter" => std::env::var("OPENROUTER_API_KEY").ok(),
            "openai" => std::env::var("OPENAI_API_KEY").ok(),
            "claude" => std::env::var("ANTHROPIC_API_KEY").ok(),
            "gemini" => std::env::var("GEMINI_API_KEY").ok(),
            _ => None,
        }
    });
    
    let api_key_str = match api_key_string {
        Some(ref key) => key.as_str(),
        None => {
            eprintln!("No API key provided for provider: {}", provider);
            eprintln!("Set environment variable or use --api-key option");
            std::process::exit(1);
        }
    };
    
    // Create AI provider
    let ai_provider = create_provider(provider, api_key_str)?;
    
    // Perform analysis
    let mut analyzer = Analyzer::new(ai_provider);
    let analysis = analyzer.analyze_logs(filtered_logs.clone()).await?;
    
    // Generate output
    let output_fmt = match output_format.as_str() {
        "json" => OutputFormat::Json,
        "html" => OutputFormat::Html,
        "markdown" => OutputFormat::Markdown,
        _ => OutputFormat::Console,
    };
    
    let input_source = if matches.contains_id("file") {
        matches.get_one::<String>("file").unwrap()
    } else if matches.contains_id("exec") {
        "command execution"
    } else {
        "stdin"
    };
    
    let output = generate_report(analysis, filtered_logs, provider, level, input_source, output_fmt)?;
    println!("{}", output);

    Ok(())
}