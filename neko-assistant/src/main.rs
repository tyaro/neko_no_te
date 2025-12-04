use clap::{Parser, Subcommand};
use std::path::PathBuf;

use chat_core::{disable_plugin, discover_plugins, enable_plugin};
use langchain_bridge::LangChainToolAgent;
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
    /// Test MCP connection
    TestMcp,
    /// Ask phi4-mini about the weather via MCP tools
    // VerifyWeather {
    //     /// City name passed to the weather MCP tool
    //     #[arg(long, default_value = "大阪")]
    //     city: String,
    //     /// Ollama model identifier
    //     #[arg(long, default_value = "phi4-mini:3.8b")]
    //     model: String,
    // },
    /// Execute a chat prompt with MCP tools and plugins
    Chat {
        /// Prompt to send to the LLM
        #[arg(short, long)]
        prompt: String,
        /// Ollama model identifier
        #[arg(short, long, default_value = "phi4-mini:3.8b")]
        model: String,
        /// Disable MCP tool loading
        #[arg(long)]
        no_mcp: bool,
        /// Disable plugin adapter loading
        #[arg(long)]
        no_plugins: bool,
        /// Output format: text or json
        #[arg(long, default_value = "text")]
        format: String,
        /// Show detailed execution logs
        #[arg(long)]
        verbose: bool,
        /// Show raw LLM interaction (prompts and tool calls)
        #[arg(long)]
        debug: bool,
    },
}

async fn test_mcp() -> anyhow::Result<()> {
    use chat_core::{load_mcp_config, McpManager};
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

async fn chat_cli(
    prompt: String,
    model: String,
    enable_mcp: bool,
    enable_plugins: bool,
    format: String,
    verbose: bool,
    debug: bool,
    repo: &PathBuf,
) -> anyhow::Result<()> {
    use chat_core::{langchain_tools::build_mcp_tools, load_mcp_config, McpManager};
    use std::sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    };

    if verbose {
        eprintln!("[INFO] Repository root: {}", repo.display());
        eprintln!("[INFO] Model: {}", model);
        eprintln!("[INFO] MCP enabled: {}", enable_mcp);
        eprintln!("[INFO] Plugins enabled: {}", enable_plugins);
    }

    // 1. Load plugins if requested
    let mut plugin_info = Vec::new();
    if enable_plugins {
        if verbose {
            eprintln!("[INFO] Discovering plugins...");
        }
        let plugins = discover_plugins(repo)?;
        let enabled = plugins.iter().filter(|p| p.enabled).count();
        plugin_info = plugins
            .iter()
            .filter(|p| p.enabled)
            .map(|p| {
                p.metadata
                    .as_ref()
                    .and_then(|m| m.name.clone())
                    .unwrap_or_else(|| p.dir_name.clone())
            })
            .collect();
        if verbose {
            eprintln!(
                "[INFO] Found {} plugins, {} enabled",
                plugins.len(),
                enabled
            );
        }
    }

    // 2. Load MCP tools if requested
    let mut tools = Vec::new();
    let tool_counter = Arc::new(AtomicUsize::new(0));
    if enable_mcp {
        if verbose {
            eprintln!("[INFO] Loading MCP configuration...");
        }
        let configs = load_mcp_config().map_err(|e| anyhow::anyhow!(e))?;
        if !configs.is_empty() {
            let manager = Arc::new(McpManager::new(configs));
            if verbose {
                eprintln!("[INFO] Initializing MCP servers...");
            }
            manager
                .initialize_all()
                .await
                .map_err(|e| anyhow::anyhow!(e))?;

            let hook: Arc<dyn Fn() + Send + Sync> = {
                let counter = tool_counter.clone();
                Arc::new(move || {
                    counter.fetch_add(1, Ordering::SeqCst);
                })
            };

            if verbose {
                eprintln!("[INFO] Building LangChain tool descriptors...");
            }
            tools = build_mcp_tools(manager.clone(), Some(hook))
                .await
                .map_err(|e| anyhow::anyhow!(e))?;
            if verbose {
                eprintln!("[INFO] Loaded {} MCP tools", tools.len());
            }
        } else if verbose {
            eprintln!("[WARN] No MCP servers configured");
        }
    }

    // 3. Build LangChain agent
    if verbose {
        eprintln!(
            "[INFO] Creating LangChain agent with {} tools...",
            tools.len()
        );
    }
    let agent = LangChainToolAgent::new(&model, tools).map_err(|e| anyhow::anyhow!(e))?;

    // 4. Execute prompt
    if verbose {
        eprintln!("[INFO] Sending prompt to LLM...");
    }
    let start = std::time::Instant::now();
    let response = if debug {
        agent
            .invoke_with_debug(&prompt)
            .await
            .map_err(|e| anyhow::anyhow!(e))?
    } else {
        agent
            .invoke(&prompt)
            .await
            .map_err(|e| anyhow::anyhow!(e))?
    };
    let elapsed = start.elapsed();

    let tool_calls = tool_counter.load(Ordering::SeqCst);

    // 5. Output results
    match format.as_str() {
        "json" => {
            let output = serde_json::json!({
                "response": response.trim(),
                "model": model,
                "elapsed_ms": elapsed.as_millis(),
                "tool_calls": tool_calls,
                "plugins_enabled": plugin_info,
                "mcp_enabled": enable_mcp,
            });
            println!("{}", serde_json::to_string_pretty(&output)?);
        }
        _ => {
            println!("{}", response.trim());
            if verbose {
                eprintln!("\n[INFO] Execution time: {:?}", elapsed);
                eprintln!("[INFO] Tool calls: {}", tool_calls);
                if !plugin_info.is_empty() {
                    eprintln!("[INFO] Plugins: {}", plugin_info.join(", "));
                }
            }
        }
    }

    Ok(())
}

// async fn verify_weather(city: String, model: String) -> anyhow::Result<()> {
//     use chat_core::{langchain_tools::build_mcp_tools, load_mcp_config, McpManager};
//     use std::sync::{
//         atomic::{AtomicUsize, Ordering},
//         Arc,
//     };

//     println!("Loading MCP configuration...");
//     let configs = load_mcp_config().map_err(|e| anyhow::anyhow!(e))?;
//     if configs.is_empty() {
//         anyhow::bail!(
//             "No MCP servers configured. Place mcp_servers.json next to the executable and ensure weather server is defined."
//         );
//     }

//     let manager = Arc::new(McpManager::new(configs));
//     println!("Initializing MCP servers...");
//     manager
//         .initialize_all()
//         .await
//         .map_err(|e| anyhow::anyhow!(e))?;

//     let tool_counter = Arc::new(AtomicUsize::new(0));
//     let hook: Arc<dyn Fn() + Send + Sync> = {
//         let counter = tool_counter.clone();
//         Arc::new(move || {
//             counter.fetch_add(1, Ordering::SeqCst);
//         })
//     };

//     println!("Building LangChain tool descriptors...");
//     let tools = build_mcp_tools(manager.clone(), Some(hook))
//         .await
//         .map_err(|e| anyhow::anyhow!(e))?;
//     if tools.is_empty() {
//         anyhow::bail!(
//             "No MCP tools available. Ensure weather MCP server exposes get_weather_forecast."
//         );
//     }

//     let weather_tool_name = tools
//         .iter()
//         .map(|tool| tool.name())
//         .find(|name| name.contains("get_weather_forecast") || name.eq("get_weather_forecast"))
//         .ok_or_else(|| {
//             anyhow::anyhow!(
//                 "Weather MCP tool not found. Ensure get_weather_forecast is exposed by the weather server."
//             )
//         })?;
//     println!("Detected weather tool: {}", weather_tool_name);

//     println!("Starting LangChain agent with model {model}...");
//     let agent = LangChainToolAgent::new(&model, tools).map_err(|e| anyhow::anyhow!(e))?;

//     let prompt = format!(
//         r#"あなたはMCPの {tool_name} ツールを必ず呼び出してから回答する天気アシスタントです。
// 指定された都市の今日と明日の天気、降水確率、最高/最低気温を日本語でまとめてください。

// 都市: {city}
// "#,
//         tool_name = weather_tool_name,
//         city = city
//     );

//     println!("Sending prompt to phi4-mini:\n{}\n", prompt.trim());
//     let response = agent
//         .invoke(prompt.trim())
//         .await
//         .map_err(|e| anyhow::anyhow!(e))?;

//     println!("LLM response:\n{}\n", response.trim());

//     let calls = tool_counter.load(Ordering::SeqCst);
//     if calls == 0 {
//         anyhow::bail!("LangChain agent responded without invoking the MCP weather tool.");
//     }

//     println!("✓ MCP weather tool invocations observed: {}", calls);
//     Ok(())
// }

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
            // Some(Commands::VerifyWeather { city, model }) => {
            //     println!("Running phi4-mini weather verification...");
            //     verify_weather(city, model).await?;
            // }
            Some(Commands::Chat {
                prompt,
                model,
                no_mcp,
                no_plugins,
                format,
                verbose,
                debug,
            }) => {
                chat_cli(
                    prompt,
                    model,
                    !no_mcp,
                    !no_plugins,
                    format,
                    verbose,
                    debug,
                    &repo,
                )
                .await?;
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
