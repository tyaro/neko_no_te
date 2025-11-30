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
        url.set_path(&format!("{}/api/generate", url.path().trim_end_matches('/')));

        let payload = serde_json::json!({
            "model": model,
            "prompt": prompt,
        });

        let res = self.client.post(url).json(&payload).send().await?;
        let status = res.status();
        let text = res.text().await?;

        if !status.is_success() {
            // surface non-2xx for caller to handle
            Err(reqwest::Error::new(
                reqwest::StatusCode::from_u16(status.as_u16()).unwrap_or(reqwest::StatusCode::INTERNAL_SERVER_ERROR),
                format!("server returned {}: {}", status, text),
            ))
        } else {
            Ok(text)
        }
    }
}

// Small helper types for callers who want to deserialize standard responses.
#[derive(Debug, Deserialize)]
pub struct GenerateResponse {
    // Keep this generic; actual shape depends on Ollama version and config.
    pub output: Option<serde_json::Value>,
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
