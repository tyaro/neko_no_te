use clap::{Parser, Subcommand};
use std::path::PathBuf;

mod plugins;
use plugins::{discover_plugins, disable_plugin, enable_plugin};
mod gui;
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
                        println!("- {}  [{}]", title, if p.enabled { "enabled" } else { "disabled" });
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
