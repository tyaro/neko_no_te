use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::process::Stdio;
use std::sync::{Arc, Mutex};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, ChildStdin, ChildStdout, Command};
use tokio::time::{timeout, Duration};

/// MCP (Model Context Protocol) クライアント
/// 外部MCPサーバーと通信してツールを呼び出す
pub struct McpClient {
    server_process: Option<Child>,
    stdin: Option<ChildStdin>,
    stdout: Option<BufReader<ChildStdout>>,
    request_id: Arc<Mutex<u64>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JsonRpcRequest {
    jsonrpc: String,
    id: u64,
    method: String,
    params: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JsonRpcResponse {
    jsonrpc: String,
    id: u64,
    result: Option<serde_json::Value>,
    error: Option<JsonRpcError>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JsonRpcError {
    code: i32,
    message: String,
    data: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpTool {
    pub name: String,
    pub description: String,
    pub input_schema: serde_json::Value,
}

impl McpClient {
    /// 新しいMCPクライアントを作成し、サーバープロセスを起動
    pub async fn new(
        server_command: &str,
        args: &[String],
        env: Option<HashMap<String, String>>,
    ) -> Result<Self, String> {
        let mut command = Command::new(server_command);
        command
            .args(args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null());

        // 環境変数を設定（指定がなければ親プロセスから継承）
        if let Some(custom_env) = env {
            command.env_clear();
            // システムPATHを維持
            if let Ok(path) = std::env::var("PATH") {
                command.env("PATH", path);
            }
            // カスタム環境変数を追加
            for (key, value) in custom_env {
                command.env(key, value);
            }
        }

        let mut child = command
            .spawn()
            .map_err(|e| format!("Failed to start MCP server: {}", e))?;

        let stdin = child.stdin.take().ok_or("Failed to get stdin")?;
        let stdout = child.stdout.take().ok_or("Failed to get stdout")?;

        Ok(Self {
            server_process: Some(child),
            stdin: Some(stdin),
            stdout: Some(BufReader::new(stdout)),
            request_id: Arc::new(Mutex::new(1)),
        })
    }

    /// 次のリクエストIDを取得
    fn next_request_id(&self) -> u64 {
        let mut id = self.request_id.lock().unwrap();
        let current = *id;
        *id += 1;
        current
    }

    /// MCPサーバーを初期化
    pub async fn initialize(&mut self) -> Result<(), String> {
        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: self.next_request_id(),
            method: "initialize".to_string(),
            params: Some(serde_json::json!({
                "protocolVersion": "2024-11-05",
                "capabilities": {
                    "tools": {}
                },
                "clientInfo": {
                    "name": "neko-assistant",
                    "version": "0.1.0"
                }
            })),
        };

        self.send_request(&request).await?;
        let _response = self.receive_response().await?;

        // initialized通知を送信
        let notification = serde_json::json!({
            "jsonrpc": "2.0",
            "method": "notifications/initialized"
        });

        if let Some(stdin) = &mut self.stdin {
            let json = serde_json::to_string(&notification)
                .map_err(|e| format!("Failed to serialize notification: {}", e))?;
            stdin
                .write_all(format!("{}\n", json).as_bytes())
                .await
                .map_err(|e| format!("Failed to write notification: {}", e))?;
            stdin
                .flush()
                .await
                .map_err(|e| format!("Failed to flush: {}", e))?;
        }

        Ok(())
    }

    /// 利用可能なツール一覧を取得
    pub async fn list_tools(&mut self) -> Result<Vec<McpTool>, String> {
        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: self.next_request_id(),
            method: "tools/list".to_string(),
            params: None,
        };

        self.send_request(&request).await?;
        let response = self.receive_response().await?;

        eprintln!("DEBUG: list_tools response: {:#?}", response);

        if let Some(error) = response.error {
            return Err(format!("Error: {} - {}", error.code, error.message));
        }

        let tools_value = response
            .result
            .and_then(|r| r.get("tools").cloned())
            .ok_or("No tools in response")?;

        eprintln!("DEBUG: tools_value: {:#?}", tools_value);

        let tools: Vec<McpTool> = serde_json::from_value(tools_value)
            .map_err(|e| format!("Failed to parse tools: {}", e))?;

        Ok(tools)
    }

    /// ツールを呼び出す
    pub async fn call_tool(
        &mut self,
        tool_name: &str,
        arguments: serde_json::Value,
    ) -> Result<serde_json::Value, String> {
        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: self.next_request_id(),
            method: "tools/call".to_string(),
            params: Some(serde_json::json!({
                "name": tool_name,
                "arguments": arguments
            })),
        };

        self.send_request(&request).await?;
        let response = self.receive_response().await?;

        if let Some(error) = response.error {
            return Err(format!(
                "Tool call error: {} - {}",
                error.code, error.message
            ));
        }

        response.result.ok_or("No result in response".to_string())
    }

    /// リクエストを送信
    async fn send_request(&mut self, request: &JsonRpcRequest) -> Result<(), String> {
        if let Some(stdin) = &mut self.stdin {
            let json = serde_json::to_string(request)
                .map_err(|e| format!("Failed to serialize request: {}", e))?;
            stdin
                .write_all(format!("{}\n", json).as_bytes())
                .await
                .map_err(|e| format!("Failed to write request: {}", e))?;
            stdin
                .flush()
                .await
                .map_err(|e| format!("Failed to flush: {}", e))?;
            Ok(())
        } else {
            Err("No stdin available".to_string())
        }
    }

    /// レスポンスを受信
    async fn receive_response(&mut self) -> Result<JsonRpcResponse, String> {
        let stdout = self
            .stdout
            .as_mut()
            .ok_or("No stdout available".to_string())?;
        let mut line = String::new();
        let mut frame = String::new();

        loop {
            line.clear();
            let read_result = timeout(Duration::from_secs(10), stdout.read_line(&mut line))
                .await
                .map_err(|_| "Timed out waiting for MCP response".to_string())?;

            match read_result {
                Ok(0) => return Err("MCP server closed the connection".to_string()),
                Ok(_) => {
                    if line.trim().is_empty() {
                        continue;
                    }
                    frame.push_str(&line);
                    let trimmed = frame.trim();
                    match serde_json::from_str::<JsonRpcResponse>(trimmed) {
                        Ok(response) => {
                            eprintln!("DEBUG: MCP raw frame: {}", trimmed);
                            frame.clear();
                            return Ok(response);
                        }
                        Err(err) => {
                            if err.is_eof() {
                                // 不完全なフレーム。追加行を待つ。
                                continue;
                            }
                            eprintln!(
                                "Failed to parse MCP response fragment: {}. Current frame: {}",
                                err, trimmed
                            );
                            frame.clear();
                        }
                    }
                }
                Err(e) => {
                    return Err(format!("Failed to read response: {}", e));
                }
            }
        }
    }
}

impl Drop for McpClient {
    fn drop(&mut self) {
        if let Some(mut child) = self.server_process.take() {
            let _ = child.kill();
        }
    }
}

/// MCP設定
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpServerConfig {
    pub name: String,
    pub command: String,
    pub args: Vec<String>,
    pub env: Option<HashMap<String, String>>,
}

/// MCPサーバー設定を読み込む
pub fn load_mcp_config() -> Result<Vec<McpServerConfig>, String> {
    let config_path = ensure_mcp_config_path()?;

    if !config_path.exists() {
        return Ok(get_default_mcp_config());
    }

    let content = std::fs::read_to_string(&config_path)
        .map_err(|e| format!("Failed to read MCP config: {}", e))?;

    let configs: Vec<McpServerConfig> =
        serde_json::from_str(&content).map_err(|e| format!("Failed to parse MCP config: {}", e))?;

    Ok(configs)
}

/// MCPサーバー設定を書き込む
pub fn save_mcp_config(configs: &[McpServerConfig]) -> Result<(), String> {
    let path = ensure_mcp_config_path()?;
    let json = serde_json::to_string_pretty(configs)
        .map_err(|e| format!("Failed to serialize MCP config: {}", e))?;
    std::fs::write(&path, json).map_err(|e| format!("Failed to write MCP config: {}", e))?;
    Ok(())
}

fn ensure_mcp_config_path() -> Result<PathBuf, String> {
    let exe_dir = std::env::current_exe()
        .map_err(|e| format!("Failed to get current executable path: {}", e))?
        .parent()
        .map(|p| p.to_path_buf())
        .ok_or("Executable has no parent directory")?;
    std::fs::create_dir_all(&exe_dir)
        .map_err(|e| format!("Failed to ensure executable directory exists: {}", e))?;
    Ok(exe_dir.join("mcp_servers.json"))
}

/// デフォルトのMCP設定を返す
fn get_default_mcp_config() -> Vec<McpServerConfig> {
    vec![
        // デフォルトでは設定なし
        // ユーザーは設定ファイルを作成して有効化する
    ]
}

/// MCP設定ファイルのサンプルを生成
#[allow(dead_code)]
pub fn create_sample_config() -> Result<(), String> {
    let config_path = ensure_mcp_config_path()?;
    let config_dir = config_path
        .parent()
        .map(|p| p.to_path_buf())
        .ok_or("Failed to determine MCP config directory")?;

    // ディレクトリを作成
    std::fs::create_dir_all(&config_dir)
        .map_err(|e| format!("Failed to create config directory: {}", e))?;

    let config_path = config_dir.join("mcp_servers.json.sample");

    let sample_configs = vec![
        McpServerConfig {
            name: "filesystem".to_string(),
            command: "npx".to_string(),
            args: vec![
                "-y".to_string(),
                "@modelcontextprotocol/server-filesystem".to_string(),
                "C:\\Users\\YourUsername\\Documents".to_string(),
            ],
            env: None,
        },
        McpServerConfig {
            name: "github".to_string(),
            command: "npx".to_string(),
            args: vec![
                "-y".to_string(),
                "@modelcontextprotocol/server-github".to_string(),
            ],
            env: Some({
                let mut env = std::collections::HashMap::new();
                env.insert(
                    "GITHUB_PERSONAL_ACCESS_TOKEN".to_string(),
                    "your_token_here".to_string(),
                );
                env
            }),
        },
        McpServerConfig {
            name: "weather".to_string(),
            command: "target\\\\debug\\\\mcp-weather-server.exe".to_string(),
            args: vec![],
            env: None,
        },
    ];

    let json = serde_json::to_string_pretty(&sample_configs)
        .map_err(|e| format!("Failed to serialize sample config: {}", e))?;

    std::fs::write(&config_path, json)
        .map_err(|e| format!("Failed to write sample config: {}", e))?;

    println!("Sample MCP config created at: {}", config_path.display());
    Ok(())
}
