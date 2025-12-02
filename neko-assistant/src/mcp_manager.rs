use crate::mcp_client::{McpClient, McpServerConfig, McpTool};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

/// 複数のMCPサーバーを管理
pub struct McpManager {
    clients: Arc<Mutex<HashMap<String, McpClient>>>,
    configs: Vec<McpServerConfig>,
}

impl McpManager {
    /// 新しいMCPマネージャーを作成
    pub fn new(configs: Vec<McpServerConfig>) -> Self {
        Self {
            clients: Arc::new(Mutex::new(HashMap::new())),
            configs,
        }
    }

    /// すべてのMCPサーバーを起動して初期化
    pub async fn initialize_all(&self) -> Result<(), String> {
        let mut clients = self.clients.lock().await;

        for config in &self.configs {
            match McpClient::new(&config.command, &config.args, config.env.clone()).await {
                Ok(mut client) => {
                    // 初期化
                    if let Err(e) = client.initialize().await {
                        eprintln!("Failed to initialize MCP server '{}': {}", config.name, e);
                        continue;
                    }
                    clients.insert(config.name.clone(), client);
                    println!("MCP server '{}' initialized successfully", config.name);
                }
                Err(e) => {
                    eprintln!("Failed to start MCP server '{}': {}", config.name, e);
                }
            }
        }

        Ok(())
    }

    async fn ensure_initialized(&self) -> Result<(), String> {
        let needs_init = {
            let clients = self.clients.lock().await;
            clients.is_empty()
        };

        if needs_init {
            self.initialize_all().await?;
        }

        Ok(())
    }

    /// すべてのMCPサーバーからツール一覧を取得
    pub async fn get_all_tools(&self) -> Result<Vec<(String, McpTool)>, String> {
        self.ensure_initialized().await?;
        let mut clients = self.clients.lock().await;
        let mut all_tools = Vec::new();

        for (server_name, client) in clients.iter_mut() {
            match client.list_tools().await {
                Ok(tools) => {
                    for tool in tools {
                        all_tools.push((server_name.clone(), tool));
                    }
                }
                Err(e) => {
                    eprintln!("Failed to list tools from '{}': {}", server_name, e);
                }
            }
        }

        Ok(all_tools)
    }

    /// 指定したサーバーのツールを呼び出す
    pub async fn call_tool(
        &self,
        server_name: &str,
        tool_name: &str,
        arguments: serde_json::Value,
    ) -> Result<serde_json::Value, String> {
        self.ensure_initialized().await?;
        let mut clients = self.clients.lock().await;

        let client = clients
            .get_mut(server_name)
            .ok_or_else(|| format!("MCP server '{}' not found", server_name))?;

        client.call_tool(tool_name, arguments).await
    }

    /// ツール名からサーバー名を検索
    #[allow(dead_code)]
    pub async fn find_server_for_tool(&self, tool_name: &str) -> Result<String, String> {
        self.ensure_initialized().await?;
        let mut clients = self.clients.lock().await;

        for (server_name, client) in clients.iter_mut() {
            match client.list_tools().await {
                Ok(tools) => {
                    if tools.iter().any(|t| t.name == tool_name) {
                        return Ok(server_name.clone());
                    }
                }
                Err(e) => {
                    eprintln!("Failed to list tools from '{}': {}", server_name, e);
                }
            }
        }

        Err(format!("Tool '{}' not found in any MCP server", tool_name))
    }

    /// LangChain用のツール説明を生成
    #[allow(dead_code)]
    pub async fn get_tools_description(&self) -> String {
        let tools = match self.get_all_tools().await {
            Ok(t) => t,
            Err(e) => {
                eprintln!("Failed to get tools: {}", e);
                return String::new();
            }
        };

        if tools.is_empty() {
            return String::new();
        }

        let mut description = String::from("\n\nAvailable Tools:\n");
        for (server, tool) in tools {
            description.push_str(&format!(
                "- {}@{}: {}\n",
                tool.name, server, tool.description
            ));
        }

        description
    }
}
