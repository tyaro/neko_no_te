//! A tiny, configurable client for talking to a local Ollama HTTP endpoint.
//!
//! This crate intentionally keeps the implementation minimal and configurable:
//! - `base_url` is provided by the caller (defaults are documented in the crate README).
//! - `generate` posts a JSON payload to `api/generate` by default; if your
//!   local Ollama uses a different path, configure the base URL accordingly.

use reqwest::Url;
use serde::Deserialize;

#[derive(Clone)]
pub struct OllamaClient {
    base: Url,
    client: reqwest::Client,
}

impl OllamaClient {
    /// Create a new client with the given `base_url` (e.g. `http://localhost:11434/`).
    ///
    /// Returns an error if the URL cannot be parsed.
    pub fn new(base_url: &str) -> Result<Self, url::ParseError> {
        let base = Url::parse(base_url)?;
        let client = reqwest::Client::new();
        Ok(Self { base, client })
    }

    /// Check that the server responds (simple GET to base URL).
    ///
    /// This is a lightweight health-check; some Ollama installs may not expose
    /// a root page — in that case, use `generate` or adjust as needed.
    pub async fn health(&self) -> Result<bool, reqwest::Error> {
        let res = self.client.get(self.base.clone()).send().await?;
        Ok(res.status().is_success())
    }

    /// Send a prompt to the server using POST to `<base>/api/generate`.
    ///
    /// The default payload is `{"model": model, "prompt": prompt}`. The
    /// response body is returned as a string for maximum flexibility — callers
    /// can deserialize to a concrete shape if desired.
    pub async fn generate(&self, model: &str, prompt: &str) -> Result<String, reqwest::Error> {
        let mut url = self.base.clone();
        // join with a relative path; if base already contains a path this will work
        url.set_path(&format!(
            "{}/api/generate",
            url.path().trim_end_matches('/')
        ));

        let payload = serde_json::json!({
            "model": model,
            "prompt": prompt,
            "stream": false,
        });

        let res = self.client.post(url).json(&payload).send().await?;

        if !res.status().is_success() {
            return Err(res.error_for_status().unwrap_err());
        }

        let json_text = res.text().await?;

        // Ollama レスポンスから "response" フィールドを抽出
        match serde_json::from_str::<serde_json::Value>(&json_text) {
            Ok(response) => {
                let text = response["response"]
                    .as_str()
                    .unwrap_or(&json_text)
                    .to_string();
                Ok(text)
            }
            Err(_) => {
                // JSON パースに失敗した場合はそのまま返す
                Ok(json_text)
            }
        }
    }

    /// ストリーミングで生成（コールバックで部分応答を受け取る）
    pub async fn generate_stream<F>(
        &self,
        model: &str,
        prompt: &str,
        mut callback: F,
    ) -> Result<String, reqwest::Error>
    where
        F: FnMut(&str),
    {
        let mut url = self.base.clone();
        url.set_path(&format!(
            "{}/api/generate",
            url.path().trim_end_matches('/')
        ));

        let payload = serde_json::json!({
            "model": model,
            "prompt": prompt,
            "stream": true,
        });

        let res = self.client.post(url).json(&payload).send().await?;

        if !res.status().is_success() {
            return Err(res.error_for_status().unwrap_err());
        }

        let text = res.text().await?;
        let mut full_response = String::new();

        // 各行が個別のJSONオブジェクト
        for line in text.lines() {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(line) {
                if let Some(chunk) = json["response"].as_str() {
                    full_response.push_str(chunk);
                    callback(chunk);
                }
            }
        }

        Ok(full_response)
    }

    /// Retrieve the list of locally available models via `<base>/api/tags`.
    pub async fn list_models(&self) -> Result<Vec<OllamaListedModel>, reqwest::Error> {
        let mut url = self.base.clone();
        url.set_path(&format!("{}/api/tags", url.path().trim_end_matches('/')));

        let res = self.client.get(url).send().await?;
        let res = res.error_for_status()?;
        let payload: OllamaTagsResponse = res.json().await?;
        Ok(payload.models)
    }
}

// Small helper types for callers who want to deserialize standard responses.
#[derive(Debug, Deserialize)]
pub struct GenerateResponse {
    // Keep this generic; actual shape depends on Ollama version and config.
    pub output: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct OllamaListedModel {
    pub name: String,
    #[serde(default)]
    pub model: Option<String>,
    #[serde(default)]
    pub size: Option<u64>,
    #[serde(default)]
    pub digest: Option<String>,
    #[serde(default)]
    pub modified_at: Option<String>,
    #[serde(default)]
    pub details: Option<OllamaModelDetails>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct OllamaModelDetails {
    #[serde(default)]
    pub family: Option<String>,
    #[serde(default)]
    pub format: Option<String>,
    #[serde(default)]
    pub parameter_size: Option<String>,
    #[serde(default)]
    pub quantization_level: Option<String>,
}

#[derive(Debug, Deserialize)]
struct OllamaTagsResponse {
    pub models: Vec<OllamaListedModel>,
}

#[cfg(test)]
mod tests {
    use super::*;

    // These tests are lightweight and mostly ensure the client constructs
    // without panicking. Integration tests against a running Ollama should
    // be added by the integrator and are not executed here.
    #[tokio::test]
    async fn create_client() {
        let c = OllamaClient::new("http://localhost:11434/").unwrap();
        let _ = c.health().await; // may fail if server not present; we ignore
    }
}
