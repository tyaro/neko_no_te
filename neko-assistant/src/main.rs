use clap::{Parser, Subcommand};
use std::path::PathBuf;

mod plugins;
use plugins::{discover_plugins, disable_plugin, enable_plugin};

/// Simple CLI for plugin management (foundation for GUI integration).
#[derive(Parser)]
#[command(author, version, about)]
struct Cli {
    /// Repository root (defaults to current working directory)
    #[arg(short, long)]
    repo: Option<PathBuf>,

    #[command(subcommand)]
    cmd: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// List discovered plugins and their enabled state
    List,
    /// Enable a plugin by name
    Enable { name: String },
    /// Disable a plugin by name
    Disable { name: String },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let repo = cli.repo.unwrap_or(std::env::current_dir()?);

    match cli.cmd {
        Commands::List => {
            let list = discover_plugins(&repo)?;
            if list.is_empty() {
                println!("No plugins found in {}/plugins", repo.display());
            } else {
                println!("Found plugins:");
                for p in list {
                    println!("- {}  [{}]", p.name, if p.enabled { "enabled" } else { "disabled" });
                }
            }
        }
        Commands::Enable { name } => {
            enable_plugin(&repo, &name)?;
            println!("Enabled plugin: {}", name);
        }
        Commands::Disable { name } => {
            disable_plugin(&repo, &name)?;
            println!("Disabled plugin: {}", name);
        }
    }

    Ok(())
}
