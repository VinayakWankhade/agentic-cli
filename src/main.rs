use anyhow::Result;
use clap::{Parser, Subcommand};
use tracing::{info, warn};
use tracing_subscriber;

mod agent;
mod commands;
mod config;
mod db;
mod ui;
mod warp;

use agent::Agent;
use commands::CommandRegistry;
use config::Config;
use db::Database;
use ui::App;

#[derive(Parser)]
#[command(name = "agentic")]
#[command(about = "A Warp-inspired agentic terminal interface")]
#[command(version = "0.1.0")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
    
    /// Enable debug logging
    #[arg(long, short)]
    debug: bool,
    
    /// Use interactive TUI mode
    #[arg(long, short)]
    interactive: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Task management commands
    Task {
        #[command(subcommand)]
        task_cmd: commands::task::TaskCommand,
    },
    /// Preparation and study commands
    Prep {
        #[command(subcommand)]
        prep_cmd: commands::prep::PrepCommand,
    },
    /// Blog and content commands
    Blog {
        #[command(subcommand)]
        blog_cmd: commands::blog::BlogCommand,
    },
    /// Agent interaction commands
    Agent {
        /// Natural language query for the agent
        query: String,
    },
    /// Warp-mode pipeline: natural language to shell commands
    Warp {
        /// Natural language description of what you want to do
        request: String,
        /// Execute in dry-run mode (no actual execution)
        #[arg(long)]
        dry_run: bool,
    },
    /// Run arbitrary commands
    Run {
        /// Command to execute
        command: String,
    },
    /// Start the interactive TUI
    Tui,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    
    // Initialize tracing
    if cli.debug {
        tracing_subscriber::fmt()
            .with_max_level(tracing::Level::DEBUG)
            .init();
    } else {
        tracing_subscriber::fmt()
            .with_max_level(tracing::Level::INFO)
            .init();
    }
    
    info!("Starting agentic-cli");
    
    // Initialize configuration
    let config = Config::load().await?;
    
    // Initialize database
    let db = Database::new(&config.database_path).await?;
    
    // Initialize agent
    let agent = Agent::new(&config)?;
    
    // Initialize command registry
    let command_registry = CommandRegistry::new();
    
    match cli.command {
        Some(Commands::Task { task_cmd }) => {
            command_registry.execute_task(task_cmd, &db).await?;
        }
        Some(Commands::Prep { prep_cmd }) => {
            command_registry.execute_prep(prep_cmd, &db).await?;
        }
        Some(Commands::Blog { blog_cmd }) => {
            command_registry.execute_blog(blog_cmd, &db).await?;
        }
        Some(Commands::Agent { query }) => {
            let response = agent.process_query(&query).await?;
            println!("{}", response);
        }
        Some(Commands::Warp { request, dry_run }) => {
            let pipeline = warp::WarpPipeline::new(&config)?;
            if dry_run {
                let (plan, command) = pipeline.dry_run(&request).await?;
                println!("\n{} Would execute: {}", "📋", command);
            } else {
                let result = pipeline.execute(&request).await?;
                if !result.is_success() && !result.cancelled {
                    std::process::exit(1);
                }
            }
        }
        Some(Commands::Run { command }) => {
            command_registry.execute_raw_command(&command).await?;
        }
        Some(Commands::Tui) | None => {
            // Start interactive TUI mode
            if cli.interactive || cli.command.is_none() {
                start_tui_mode(config, db, agent, command_registry).await?;
            }
        }
    }
    
    Ok(())
}

async fn start_tui_mode(
    config: Config,
    db: Database,
    agent: Agent,
    command_registry: CommandRegistry,
) -> Result<()> {
    info!("Starting TUI mode");
    
    let mut terminal = ui::setup_terminal()?;
    let mut app = App::new(config, db, agent, command_registry);
    
    let result = app.run(&mut terminal).await;
    
    ui::restore_terminal(&mut terminal)?;
    
    match &result {
        Ok(_) => info!("TUI exited successfully"),
        Err(e) => warn!("TUI exited with error: {}", e),
    }
    
    result
}
