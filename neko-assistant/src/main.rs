use clap::{Parser, Subcommand};
use std::path::PathBuf;

mod plugins;
use plugins::{disable_plugin, discover_plugins, enable_plugin};
mod conversation_service;
mod gui;
mod langchain_tools;
mod mcp_client;
mod mcp_manager;
mod message_handler;
// `run_gui` is called with full path `gui::run_gui()` below; importing the name is unused.

/// Simple CLI for plugin management (foundation for GUI integration).
#[derive(Parser)]
#[command(author, version, about)]
struct Cli {
    /// Repository root (defaults to current working directory)
    #[arg(short, long)]
    repo: Option<PathBuf>,

    /// Launch GUI (requires building with `--features gui` or a GPUI implementation)
    #[arg(long)]
    cli: bool,

    #[command(subcommand)]
    cmd: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// List discovered plugins and their enabled state
    List,
    /// Enable a plugin by name
    Enable { name: String },
    /// Disable a plugin by name
    Disable { name: String },
    /// Test MCP connection
    TestMcp,
}

async fn test_mcp() -> anyhow::Result<()> {
    use crate::mcp_client::load_mcp_config;
    use crate::mcp_manager::McpManager;
    use std::sync::Arc;

    println!("Loading MCP configuration...");
    let configs = load_mcp_config().map_err(|e| anyhow::anyhow!(e))?;
    println!("Found {} MCP server(s)", configs.len());

    let manager = Arc::new(McpManager::new(configs));

    println!("Initializing MCP servers...");
    manager
        .initialize_all()
        .await
        .map_err(|e| anyhow::anyhow!(e))?;
    println!("✓ All servers initialized");

    println!("\nFetching available tools...");
    let tools = manager.get_all_tools().await.map_err(|e| {
        eprintln!("Error fetching tools: {}", e);
        anyhow::anyhow!("Failed to fetch tools: {}", e)
    })?;

    println!("✓ Found {} tool(s)", tools.len());

    println!("\nAvailable tools:");
    for (i, (server_name, tool)) in tools.iter().enumerate() {
        println!(
            "{}. [{}] {} - {}",
            i + 1,
            server_name,
            tool.name,
            tool.description
        );
    }

    println!("\n✓ MCP test completed successfully");
    Ok(())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let repo = cli.repo.unwrap_or(std::env::current_dir()?);

    // If user explicitly requested CLI or provided a subcommand, run CLI mode.
    if cli.cli || cli.cmd.is_some() {
        match cli.cmd {
            Some(Commands::List) => {
                let list = discover_plugins(&repo)?;
                if list.is_empty() {
                    println!("No plugins found in {}/plugins", repo.display());
                } else {
                    println!("Found plugins:");
                    for p in list {
                        let meta = p.metadata.as_ref();
                        let title = meta
                            .and_then(|m| m.name.clone())
                            .unwrap_or_else(|| p.dir_name.clone());
                        let desc = meta.and_then(|m| m.description.clone()).unwrap_or_default();
                        println!(
                            "- {}  [{}]",
                            title,
                            if p.enabled { "enabled" } else { "disabled" }
                        );
                        if !desc.is_empty() {
                            println!("    {}", desc);
                        }
                    }
                }
            }
            Some(Commands::Enable { name }) => {
                enable_plugin(&repo, &name)?;
                println!("Enabled plugin: {}", name);
            }
            Some(Commands::Disable { name }) => {
                disable_plugin(&repo, &name)?;
                println!("Disabled plugin: {}", name);
            }
            Some(Commands::TestMcp) => {
                println!("Testing MCP connection...");
                test_mcp().await?;
            }
            None => {
                // No subcommand and --cli provided: show help-ish message
                println!("No command specified. Use --help for usage.");
            }
        }

        return Ok(());
    }

    // Default behaviour: launch GUI
    gui::run_gui(&repo)?;

    Ok(())
}
