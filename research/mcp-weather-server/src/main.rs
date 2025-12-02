use std::collections::HashMap;
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tokio::io::{self, AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::sync::Mutex;
use tracing::{error, info};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .with_target(false)
        .without_time()
        .init();

    let server =
        WeatherServer::new("https://weather.tsukumijima.net", Duration::from_secs(300)).await?;
    server.run().await?;
    Ok(())
}

struct WeatherServer {
    http: reqwest::Client,
    cache: Mutex<HashMap<String, CacheEntry>>,
    city_index: HashMap<String, String>,
    base_url: String,
    cache_ttl: Duration,
}

#[derive(Clone)]
struct ToolResult {
    text: String,
    data: Value,
}

struct CacheEntry {
    fetched_at: Instant,
    payload: ToolResult,
}

impl WeatherServer {
    async fn new(base_url: &str, cache_ttl: Duration) -> Result<Self, reqwest::Error> {
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(
            reqwest::header::USER_AGENT,
            reqwest::header::HeaderValue::from_static("neko-weather-mcp/0.1"),
        );

        let http = reqwest::Client::builder()
            .default_headers(headers)
            .timeout(Duration::from_secs(10))
            .build()?;

        Ok(Self {
            http,
            cache: Mutex::new(HashMap::new()),
            city_index: build_city_index(),
            base_url: base_url.to_string(),
            cache_ttl,
        })
    }

    async fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
        let stdin = BufReader::new(io::stdin());
        let mut lines = stdin.lines();
        let mut stdout = io::stdout();

        while let Some(line) = lines.next_line().await? {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }

            let envelope: Result<JsonRpcRequest, _> = serde_json::from_str(trimmed);
            let request = match envelope {
                Ok(envelope) => envelope,
                Err(err) => {
                    error!("Failed to parse request: {err}");
                    continue;
                }
            };

            if request.id.is_none() {
                info!("Received notification: {}", request.method);
                continue;
            }

            let response = match self.handle_request(&request).await {
                Ok(result) => JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    id: request.id.clone(),
                    result: Some(result),
                    error: None,
                },
                Err(err) => JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    id: request.id.clone(),
                    result: None,
                    error: Some(JsonRpcError {
                        code: err.code,
                        message: err.message,
                        data: err.data,
                    }),
                },
            };

            let serialized = serde_json::to_string(&response)?;
            stdout.write_all(serialized.as_bytes()).await?;
            stdout.write_all(b"\n").await?;
            stdout.flush().await?;
        }

        Ok(())
    }

    async fn handle_request(&self, request: &JsonRpcRequest) -> Result<Value, RpcFailure> {
        match request.method.as_str() {
            "initialize" => Ok(json!({
                "protocolVersion": "2024-11-05",
                "serverInfo": {
                    "name": "weather-mcp-server",
                    "version": "0.1.0"
                },
                "capabilities": {
                    "tools": {}
                }
            })),
            "tools/list" => Ok(json!({
                "tools": [self.tool_spec()]
            })),
            "tools/call" => {
                let params = request
                    .params
                    .as_ref()
                    .ok_or_else(|| RpcFailure::invalid_params("Missing params"))?;
                let tool_name = params
                    .get("name")
                    .and_then(Value::as_str)
                    .ok_or_else(|| RpcFailure::invalid_params("Missing tool name"))?;
                if tool_name != "get_weather_forecast" {
                    return Err(RpcFailure::method_not_found(format!(
                        "Unknown tool: {tool_name}"
                    )));
                }
                let args = params.get("arguments").cloned().unwrap_or(Value::Null);
                let payload = self.execute_weather_tool(&args).await?;
                Ok(json!({
                    "content": [
                        {
                            "type": "text",
                            "text": payload.text
                        }
                    ],
                    "data": payload.data
                }))
            }
            other => Err(RpcFailure::method_not_found(format!(
                "Unsupported method: {other}"
            ))),
        }
    }

    fn tool_spec(&self) -> Value {
        json!({
            "name": "get_weather_forecast",
            "description": "Fetches the latest daily forecast from weather.tsukumijima.net using a supported Japanese city name or city code.",
            "input_schema": {
                "type": "object",
                "properties": {
                    "city": {
                        "type": "string",
                        "description": "City name (e.g. 東京, Osaka, sapporo)."
                    },
                    "city_code": {
                        "type": "string",
                        "description": "Optional city code such as 130010. Overrides city name when provided."
                    }
                },
                "anyOf": [
                    { "required": ["city"] },
                    { "required": ["city_code"] }
                ]
            }
        })
    }

    async fn execute_weather_tool(&self, arguments: &Value) -> Result<ToolResult, RpcFailure> {
        let args: WeatherInput = serde_json::from_value(arguments.clone())
            .map_err(|err| RpcFailure::invalid_params(format!("Invalid arguments: {err}")))?;
        let city_code = self.resolve_city_code(&args).ok_or_else(|| {
            RpcFailure::invalid_params("Unknown city. Provide city_code explicitly.")
        })?;
        let forecast = self.fetch_forecast(&city_code).await?;
        Ok(forecast)
    }

    fn resolve_city_code(&self, args: &WeatherInput) -> Option<String> {
        if let Some(code) = args.city_code.as_deref() {
            if code.chars().all(|c| c.is_ascii_digit()) && code.len() == 6 {
                return Some(code.to_string());
            }
        }

        let name = args.city.as_ref()?.trim().to_lowercase();
        self.city_index.get(&name).cloned()
    }

    async fn fetch_forecast(&self, city_code: &str) -> Result<ToolResult, RpcFailure> {
        if let Some(cached) = self.get_cached(city_code).await {
            return Ok(cached);
        }

        let url = format!("{}/api/forecast/city/{}", self.base_url, city_code);
        let response = self
            .http
            .get(&url)
            .send()
            .await
            .map_err(|err| RpcFailure::internal_error(format!("HTTP error: {err}")))?;

        if !response.status().is_success() {
            return Err(RpcFailure::internal_error(format!(
                "Weather API error: HTTP {}",
                response.status()
            )));
        }

        let body = response.json::<WeatherApiResponse>().await.map_err(|err| {
            RpcFailure::internal_error(format!("Failed to parse response: {err}"))
        })?;

        let today = body
            .forecasts
            .first()
            .ok_or_else(|| RpcFailure::internal_error("Forecast data unavailable".to_string()))?;

        let summary_text = build_summary(&body, today);

        let payload = json!({
            "city": {
                "code": city_code,
                "name": body.location.city,
                "prefecture": body.location.prefecture,
                "district": body.location.district
            },
            "title": body.title,
            "summary": today.telop,
            "detail": today
                .detail
                .weather
                .clone()
                .unwrap_or_else(|| "".to_string()),
            "date": today.date,
            "label": today.date_label,
            "temperature": {
                "max_c": today.temperature.max.as_ref().and_then(|t| t.celsius.clone()),
                "min_c": today.temperature.min.as_ref().and_then(|t| t.celsius.clone())
            },
            "chance_of_rain": today.chance_of_rain.clone(),
            "wind": today.detail.wind.clone(),
            "wave": today.detail.wave.clone(),
            "description": body.description.and_then(|d| d.body_text),
            "source": body.copyright.map(|c| c.title),
            "link": body.link
        });

        let result = ToolResult {
            text: summary_text,
            data: payload.clone(),
        };

        self.store_cache(city_code.to_string(), result.clone())
            .await;
        Ok(result)
    }

    async fn get_cached(&self, city_code: &str) -> Option<ToolResult> {
        let cache = self.cache.lock().await;
        cache.get(city_code).and_then(|entry| {
            if entry.fetched_at.elapsed() <= self.cache_ttl {
                Some(entry.payload.clone())
            } else {
                None
            }
        })
    }

    async fn store_cache(&self, city_code: String, payload: ToolResult) {
        let mut cache = self.cache.lock().await;
        cache.insert(
            city_code,
            CacheEntry {
                fetched_at: Instant::now(),
                payload,
            },
        );
    }
}

#[derive(Deserialize)]
struct WeatherInput {
    #[serde(default)]
    city: Option<String>,
    #[serde(default)]
    city_code: Option<String>,
}

#[derive(Deserialize)]
struct JsonRpcRequest {
    #[serde(rename = "jsonrpc")]
    _jsonrpc: String,
    #[serde(default)]
    id: Option<Value>,
    method: String,
    #[serde(default)]
    params: Option<Value>,
}

#[derive(Serialize)]
struct JsonRpcResponse {
    jsonrpc: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    id: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<JsonRpcError>,
}

#[derive(Serialize)]
struct JsonRpcError {
    code: i32,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    data: Option<Value>,
}

struct RpcFailure {
    code: i32,
    message: String,
    data: Option<Value>,
}

impl RpcFailure {
    fn invalid_params<T: Into<String>>(message: T) -> Self {
        Self {
            code: -32602,
            message: message.into(),
            data: None,
        }
    }

    fn method_not_found<T: Into<String>>(message: T) -> Self {
        Self {
            code: -32601,
            message: message.into(),
            data: None,
        }
    }

    fn internal_error<T: Into<String>>(message: T) -> Self {
        Self {
            code: -32603,
            message: message.into(),
            data: None,
        }
    }
}

#[derive(Deserialize)]
struct WeatherApiResponse {
    title: String,
    forecasts: Vec<Forecast>,
    location: WeatherLocation,
    link: String,
    #[serde(default)]
    description: Option<WeatherDescription>,
    #[serde(default)]
    copyright: Option<WeatherCopyright>,
}

#[derive(Deserialize)]
struct WeatherLocation {
    prefecture: String,
    district: String,
    city: String,
}

#[derive(Deserialize)]
struct WeatherDescription {
    #[serde(rename = "bodyText")]
    body_text: Option<String>,
}

#[derive(Deserialize)]
struct WeatherCopyright {
    title: String,
}

#[derive(Deserialize)]
struct Forecast {
    date: String,
    #[serde(rename = "dateLabel")]
    date_label: String,
    telop: String,
    #[serde(default)]
    detail: ForecastDetail,
    #[serde(default)]
    temperature: ForecastTemperature,
    #[serde(rename = "chanceOfRain")]
    #[serde(default)]
    chance_of_rain: HashMap<String, String>,
}

#[derive(Deserialize, Default)]
struct ForecastDetail {
    weather: Option<String>,
    wind: Option<String>,
    wave: Option<String>,
}

#[derive(Deserialize, Default)]
struct ForecastTemperature {
    min: Option<TemperatureValue>,
    max: Option<TemperatureValue>,
}

#[derive(Deserialize, Clone)]
struct TemperatureValue {
    celsius: Option<String>,
}

fn build_city_index() -> HashMap<String, String> {
    let pairs = [
        ("tokyo", "130010"),
        ("東京", "130010"),
        ("shinjuku", "130010"),
        ("osaka", "270000"),
        ("大阪", "270000"),
        ("kyoto", "260010"),
        ("京都", "260010"),
        ("yokohama", "140010"),
        ("横浜", "140010"),
        ("sapporo", "016010"),
        ("札幌", "016010"),
        ("nagoya", "230010"),
        ("名古屋", "230010"),
        ("fukuoka", "400010"),
        ("福岡", "400010"),
        ("naha", "471010"),
        ("那覇", "471010"),
    ];

    pairs
        .iter()
        .map(|(name, code)| (name.to_string(), code.to_string()))
        .collect()
}

fn build_summary(body: &WeatherApiResponse, today: &Forecast) -> String {
    let location = format!("{} {}", body.location.prefecture, body.location.city);
    let mut lines = vec![format!(
        "{} ({} {}) の予報: {}",
        location, today.date_label, today.date, today.telop
    )];

    if let Some(detail) = today.detail.weather.as_ref() {
        if !detail.trim().is_empty() {
            lines.push(detail.trim().to_string());
        }
    }

    let mut temps = Vec::new();
    if let Some(max) = today
        .temperature
        .max
        .as_ref()
        .and_then(|t| t.celsius.as_ref())
    {
        temps.push(format!("最高{}℃", max));
    }
    if let Some(min) = today
        .temperature
        .min
        .as_ref()
        .and_then(|t| t.celsius.as_ref())
    {
        temps.push(format!("最低{}℃", min));
    }
    if !temps.is_empty() {
        lines.push(temps.join(" / "));
    }

    if !today.chance_of_rain.is_empty() {
        let rain = today
            .chance_of_rain
            .iter()
            .filter(|(_, v)| !v.is_empty())
            .map(|(slot, value)| format!("{}: {}", slot, value))
            .collect::<Vec<_>>()
            .join(", ");
        if !rain.is_empty() {
            lines.push(format!("降水確率: {}", rain));
        }
    }

    if let Some(desc) = body.description.as_ref().and_then(|d| d.body_text.as_ref()) {
        lines.push(desc.trim().to_string());
    }

    lines.join("\n")
}
