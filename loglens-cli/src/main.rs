// LogLens CLI - Command-line interface for log analysis and project management

use anyhow::Result;
use clap::{Parser, Subcommand, ValueEnum};
use std::path::PathBuf;
use tracing::{error, info};
use tracing_subscriber;

/// Get the global LogLens database path (~/.loglens/data/loglens.db)
fn get_global_database_path() -> PathBuf {
    let home_dir = dirs::home_dir().unwrap_or_else(|| {
        panic!("Could not find home directory");
    });
    home_dir.join(".loglens").join("data").join("loglens.db")
}

#[derive(Clone, ValueEnum, Debug)]
enum McpTransport {
    Stdio,
    Http,
}

#[derive(Parser)]
#[command(name = "loglens")]
#[command(about = "AI-powered log analysis with project integration", long_about = None)]
#[command(version)]
struct Cli {
    /// Start the web dashboard
    #[arg(long)]
    dashboard: bool,

    /// Dashboard port (default: 3000)
    #[arg(long, default_value = "3000")]
    port: u16,

    /// Start MCP server
    #[arg(long)]
    mcp_server: bool,

    /// MCP server port (default: 3001)
    #[arg(long, default_value = "3001")]
    mcp_port: u16,

    /// MCP server transport mode (stdio or http)
    #[arg(long, value_enum, default_value = "stdio")]
    mcp_transport: McpTransport,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize LogLens in a project directory
    Init {
        /// Project path (defaults to current directory)
        #[arg(long)]
        path: Option<PathBuf>,
    },

    /// Link an existing LogLens project to the global registry
    Link {
        /// Project path (defaults to current directory)
        #[arg(long)]
        path: Option<PathBuf>,
    },

    /// Unlink a project from the global registry
    Unlink {
        /// Project path (defaults to current directory)
        #[arg(long)]
        path: Option<PathBuf>,
    },

    /// List all linked LogLens projects
    ListProjects,

    /// Validate and optionally repair project links
    ValidateLinks {
        /// Automatically repair issues
        #[arg(long)]
        repair: bool,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    // Parse CLI args first to check if we're in MCP stdio mode
    let cli = Cli::parse();

    // Only initialize logging if NOT in MCP stdio mode
    // MCP stdio transport requires pure JSON-RPC on stdout - no logging allowed
    if !cli.mcp_server || !matches!(cli.mcp_transport, McpTransport::Stdio) {
        tracing_subscriber::fmt()
            .with_env_filter(
                tracing_subscriber::EnvFilter::from_default_env()
                    .add_directive("loglens=info".parse().unwrap())
                    .add_directive("loglens_core=info".parse().unwrap()),
            )
            .init();
    }

    // Handle dashboard flag
    if cli.dashboard {
        info!("Starting web dashboard on port {}", cli.port);
        
        // Set environment for dashboard
        std::env::set_var("PORT", &cli.port.to_string());
        
        // Start web server
        if let Err(e) = start_dashboard().await {
            error!("Failed to start dashboard: {}", e);
            eprintln!("❌ Failed to start dashboard: {}", e);
            std::process::exit(1);
        }
        
        return Ok(());
    }

    // Handle MCP server flag
    if cli.mcp_server {
        info!("Starting MCP server with {:?} transport on port {}", cli.mcp_transport, cli.mcp_port);
        
        // Set environment for MCP server
        std::env::set_var("MCP_PORT", &cli.mcp_port.to_string());
        
        // Start MCP server
        if let Err(e) = start_mcp_server(cli.mcp_transport, cli.mcp_port).await {
            error!("Failed to start MCP server: {}", e);
            eprintln!("❌ Failed to start MCP server: {}", e);
            std::process::exit(1);
        }
        
        return Ok(());
    }

    // Handle subcommands
    match cli.command.unwrap_or(Commands::ListProjects) {
        Commands::Init { path } => {
            info!("Initializing LogLens project...");

            match loglens_core::project::initialize_project(path.as_ref()).await {
                Ok(result) => {
                    println!("\n✓ LogLens initialized successfully!");
                    println!("\nProject Details:");
                    println!("  Type:       {}", result.project_type);
                    println!("  ID:         {}", result.project_id);
                    println!("  Location:   {}", result.project_path.display());
                    println!("\nCreated:");
                    println!("  {}/.loglens/", result.project_path.display());
                    println!("    ├── config.toml       (project configuration)");
                    println!("    ├── metadata.json     (project metadata)");
                    println!("    ├── index.db          (analysis database)");
                    println!("    ├── analyses/         (analysis results)");
                    println!("    └── logs/             (log file cache)");
                    println!("\nNext steps:");
                    println!("  - Add log files for analysis");
                    println!("  - Configure AI provider in config.toml");
                    println!("  - Start dashboard: loglens --dashboard");
                    println!("  - Start MCP server: loglens --mcp-server");

                    Ok(())
                }
                Err(e) => {
                    error!("Failed to initialize project: {}", e);
                    eprintln!("\n✗ Initialization failed: {}", e);
                    std::process::exit(1);
                }
            }
        }

        Commands::Link { path } => {
            info!("Linking LogLens project...");

            match loglens_core::project::link_project(path.as_deref()).await {
                Ok(result) => {
                    if result.already_linked {
                        println!("\nℹ Project already linked");
                    } else {
                        println!("\n✓ Project linked successfully!");
                    }
                    println!("\nProject Details:");
                    println!("  Name:       {}", result.project_name);
                    println!("  ID:         {}", result.project_id);
                    println!("  Location:   {}", result.root_path.display());
                    println!("\nThe project is now registered in the global registry and will appear in the dashboard.");
                    Ok(())
                }
                Err(e) => {
                    error!("Failed to link project: {}", e);
                    eprintln!("\n✗ Link failed: {}", e);
                    std::process::exit(1);
                }
            }
        }

        Commands::Unlink { path } => {
            info!("Unlinking LogLens project...");

            match loglens_core::project::unlink_project(path.as_deref()).await {
                Ok(result) => {
                    if result.was_linked {
                        println!("\n✓ Project unlinked successfully!");
                    } else {
                        println!("\nℹ Project was not linked in the registry");
                    }
                    println!("\nProject Details:");
                    println!("  Name:       {}", result.project_name);
                    println!("  ID:         {}", result.project_id);
                    println!("  Location:   {}", result.root_path.display());
                    println!("\nNote: The .loglens/ directory has been preserved.");
                    Ok(())
                }
                Err(e) => {
                    error!("Failed to unlink project: {}", e);
                    eprintln!("\n✗ Unlink failed: {}", e);
                    std::process::exit(1);
                }
            }
        }

        Commands::ListProjects => {
            info!("Listing linked projects...");

            match loglens_core::project::ProjectRegistry::load() {
                Ok(registry) => {
                    let projects = registry.list_projects();

                    if projects.is_empty() {
                        println!("\nNo linked LogLens projects found.");
                        println!("\nRun 'loglens init' in a project directory to get started.");
                        println!("Then use 'loglens link' to register it in the dashboard.");
                    } else {
                        println!("\nLinked LogLens Projects:");
                        println!("┌{:─<40}┬{:─<60}┬{:─<25}┐", "", "", "");
                        println!(
                            "│ {:38} │ {:58} │ {:23} │",
                            "Name", "Path", "Last Accessed"
                        );
                        println!("├{:─<40}┼{:─<60}┼{:─<25}┤", "", "", "");

                        for (_project_id, entry) in &projects {
                            let time_ago = format_time_ago(&entry.last_accessed);
                            let path_str = entry
                                .root_path
                                .to_str()
                                .unwrap_or("<invalid path>")
                                .chars()
                                .take(58)
                                .collect::<String>();

                            println!(
                                "│ {:38} │ {:58} │ {:23} │",
                                entry.name.chars().take(38).collect::<String>(),
                                path_str,
                                time_ago
                            );
                        }

                        println!("└{:─<40}┴{:─<60}┴{:─<25}┘", "", "", "");
                        println!("\nTotal: {} projects", projects.len());
                        println!("Start the dashboard to view: loglens --dashboard");
                    }

                    Ok(())
                }
                Err(e) => {
                    error!("Failed to load project registry: {}", e);
                    eprintln!("\n✗ Failed to load registry: {}", e);
                    std::process::exit(1);
                }
            }
        }

        Commands::ValidateLinks { repair } => {
            info!("Validating project links...");

            if repair {
                match loglens_core::project::validate_and_repair().await {
                    Ok((validation, repair_report)) => {
                        println!("\n✓ Validation and repair complete!");
                        println!("\nResults:");
                        println!(
                            "  Total projects:   {}",
                            validation.total_projects
                        );
                        println!(
                            "  Valid projects:   {}",
                            validation.valid_projects
                        );
                        println!(
                            "  Projects removed: {}",
                            repair_report.removed.len()
                        );
                        println!(
                            "  Issues remaining: {}",
                            repair_report.manual_intervention.len()
                        );

                        if !repair_report.removed.is_empty() {
                            println!("\nRemoved projects:");
                            for project_id in &repair_report.removed {
                                println!("  - {}", project_id);
                            }
                        }

                        if !repair_report.manual_intervention.is_empty() {
                            println!("\nIssues requiring manual intervention:");
                            for issue in &repair_report.manual_intervention {
                                println!(
                                    "  - {} ({}): {}",
                                    issue.project_id,
                                    loglens_core::project::validate::format_issue_type(&issue.issue_type),
                                    issue.path.display()
                                );
                            }
                        }

                        Ok(())
                    }
                    Err(e) => {
                        error!("Failed to validate and repair: {}", e);
                        eprintln!("\n✗ Validation failed: {}", e);
                        std::process::exit(1);
                    }
                }
            } else {
                match loglens_core::project::validate_links().await {
                    Ok(report) => {
                        println!("\n✓ Validation complete!");
                        println!("\nResults:");
                        println!("  Total projects: {}", report.total_projects);
                        println!("  Valid projects: {}", report.valid_projects);
                        println!("  Issues found:   {}", report.issues.len());

                        if !report.issues.is_empty() {
                            println!("\nIssues found:");
                            for issue in &report.issues {
                                println!(
                                    "  - {} ({}): {}",
                                    issue.project_id,
                                    loglens_core::project::validate::format_issue_type(&issue.issue_type),
                                    issue.path.display()
                                );
                            }
                            println!("\nRun with --repair to automatically fix issues.");
                        }

                        Ok(())
                    }
                    Err(e) => {
                        error!("Failed to validate links: {}", e);
                        eprintln!("\n✗ Validation failed: {}", e);
                        std::process::exit(1);
                    }
                }
            }
        }
    }
}

/// Start the web dashboard
async fn start_dashboard() -> Result<()> {
    // Import the web server functionality
    // This assumes the web server is exposed as a library function
    use loglens_web::start_server;
    
    let port = std::env::var("PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(3000);
    
    println!("🚀 Starting LogLens dashboard on http://127.0.0.1:{}", port);
    println!("   Press Ctrl+C to stop");
    
    start_server(port).await
}

/// Start the MCP server
async fn start_mcp_server(transport: McpTransport, port: u16) -> Result<()> {
    use loglens_mcp::{create_server, Database, Config};

    // MCP server should ALWAYS use the global database at ~/.loglens/loglens.db
    let database_path = get_global_database_path();
    let db_url = format!("sqlite://{}", database_path.to_string_lossy());
    
    // Ensure global data directory exists
    let global_data_dir = database_path.parent().unwrap();
    std::fs::create_dir_all(global_data_dir)?;
    
    tracing::info!("MCP server using global database: {}", database_path.display());
    let db = Database::new(&db_url).await?;
    
    // Create server configuration
    let config = Config::default();
    
    // Create MCP server
    let server = create_server(db, config).await?;
    
    println!("🔗 Starting LogLens MCP server");
    println!("   Transport: {:?}", transport);
    if matches!(transport, McpTransport::Http) {
        println!("   Port: {}", port);
    }
    println!("   Tools available: list_projects, get_project, list_analyses, get_analysis, get_analysis_status, analyze_file");
    println!("   Press Ctrl+C to stop");
    
    // Start server with appropriate transport
    match transport {
        McpTransport::Stdio => {
            server.start_stdio().await?;
        }
        McpTransport::Http => {
            server.start_http(port).await?;
        }
    }
    
    Ok(())
}

/// Format a timestamp as a human-readable "time ago" string
fn format_time_ago(timestamp: &chrono::DateTime<chrono::Utc>) -> String {
    use chrono::Utc;

    let now = Utc::now();
    let duration = now.signed_duration_since(*timestamp);

    if duration.num_seconds() < 60 {
        "just now".to_string()
    } else if duration.num_minutes() < 60 {
        format!("{} mins ago", duration.num_minutes())
    } else if duration.num_hours() < 24 {
        format!("{} hours ago", duration.num_hours())
    } else if duration.num_days() < 7 {
        format!("{} days ago", duration.num_days())
    } else if duration.num_weeks() < 4 {
        format!("{} weeks ago", duration.num_weeks())
    } else {
        format!("{} months ago", duration.num_days() / 30)
    }
}