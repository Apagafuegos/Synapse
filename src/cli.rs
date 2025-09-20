use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "loglens",
    about = "A fast CLI log analyzer for parsing, filtering, and summarizing logs",
    version = "0.1.0",
    author = "LogLens Team"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,
    
    /// Input file or directory (defaults to stdin)
    #[arg(short, long)]
    pub input: Option<String>,
    
    /// Output format (text, json)
    #[arg(short, long, default_value = "text")]
    pub format: String,
    
    /// Enable verbose output
    #[arg(short, long)]
    pub verbose: bool,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Parse and analyze logs
    Analyze {
        /// Filter by log level (debug, info, warn, error)
        #[arg(short, long)]
        level: Option<String>,
        
        /// Filter by time range (start:end)
        #[arg(short, long)]
        time: Option<String>,
        
        /// Filter by regex pattern
        #[arg(short, long)]
        pattern: Option<String>,
        
        /// Advanced filter query (e.g., 'level=="ERROR" AND message~"database"')
        #[arg(long)]
        query: Option<String>,
        
        /// Enable tailing/follow mode
        #[arg(short, long)]
        follow: bool,
    },
    
    /// Show log summary statistics
    Summary {
        /// Show top N error messages
        #[arg(short, long, default_value = "5")]
        top_errors: usize,
        
        /// Advanced filter query for summary
        #[arg(long)]
        query: Option<String>,
    },
    
    /// Export logs to JSON format
    Export {
        /// Output file path
        #[arg(short, long)]
        output: String,
        
        /// Advanced filter query for export
        #[arg(long)]
        query: Option<String>,
    },
    
    /// Advanced analytics and anomaly detection
    AnalyzeAdvanced {
        /// Enable anomaly detection
        #[arg(long)]
        anomaly: bool,
        
        /// Enable pattern clustering
        #[arg(long)]
        cluster: bool,
        
        /// Generate visualization data
        #[arg(long)]
        visualize: bool,
        
        /// Output format for analytics (text, json, svg)
        #[arg(long, default_value = "text")]
        format: String,
    },
    
    /// Interactive terminal user interface
    Interactive {
        /// Enable real-time log monitoring
        #[arg(long)]
        follow: bool,
        
        /// Pre-apply filter
        #[arg(long)]
        filter: Option<String>,
        
        /// Pre-apply advanced query
        #[arg(long)]
        query: Option<String>,
    },
    
    /// Configuration management for AI providers
    Config {
        #[command(subcommand)]
        action: ConfigCommands,
    },
    
    /// AI-powered log analysis
    Ai {
        #[command(subcommand)]
        action: AiCommands,
    },
    
    /// Execute and monitor processes with AI analysis
    Run {
        /// Command to execute
        #[arg(index = 1)]
        command: String,
        
        /// Command arguments
        args: Vec<String>,
        
        /// Enable real-time log monitoring
        #[arg(long)]
        follow: bool,
        
        /// Disable automatic AI analysis
        #[arg(long)]
        no_analysis: bool,
        
        /// Specific analysis trigger patterns
        #[arg(long)]
        analysis_trigger: Option<Vec<String>>,
    },
}

#[derive(Subcommand, Clone)]
pub enum ConfigCommands {
    /// Initialize default configuration file
    Init,
    
    /// Set the default AI provider
    SetDefaultProvider {
        /// Provider name (e.g., openrouter, openai, anthropic)
        #[arg(index = 1)]
        provider: String,
    },
    
    /// List all configured providers
    ListProviders,
    
    /// Test provider connectivity
    TestProvider {
        /// Provider name to test (or 'all' for all providers)
        #[arg(index = 1)]
        provider: String,
    },
    
    /// Show configuration validation warnings
    Validate,
    
    /// Show current configuration
    Show,
}

#[derive(Subcommand, Clone)]
pub enum AiCommands {
    /// Analyze logs using AI
    Analyze {
        /// Path to log file (if not specified, uses CLI input)
        #[arg(short, long)]
        input: Option<String>,
        
        /// Override default provider
        #[arg(long)]
        provider: Option<String>,
        
        /// Override default model
        #[arg(long)]
        model: Option<String>,
        
        /// Analysis depth (basic, detailed, comprehensive)
        #[arg(long, default_value = "detailed")]
        depth: String,
        
        /// Focus areas (errors, performance, security, etc.)
        #[arg(long)]
        focus: Option<Vec<String>>,
        
        /// Output format (structured, human, json, markdown)
        #[arg(long, default_value = "human")]
        format: String,
        
        /// Maximum context entries to include
        #[arg(long, default_value = "100")]
        max_context: usize,
    },
    
    /// Generate recommendations based on log analysis
    Recommend {
        /// Path to log file or analysis results
        #[arg(short, long)]
        input: String,
        
        /// Override default provider
        #[arg(long)]
        provider: Option<String>,
    },
    
    /// Show provider capabilities and models
    Info {
        /// Provider name (or 'all' for all providers)
        #[arg(index = 1)]
        provider: String,
    },
}