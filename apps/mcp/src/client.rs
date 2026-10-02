//! Thin HTTP client over the Soundome REST API (`<base>/api/...`).

use std::time::Duration;

use reqwest::Method;
use serde_json::Value;

#[derive(Clone)]
pub struct ApiClient {
    http: reqwest::Client,
    /// Base URL including the `/api` prefix, without trailing slash.
    base: String,
}

impl ApiClient {
    pub fn new(api_url: &str) -> anyhow::Result<Self> {
        let trimmed = api_url.trim_end_matches('/');
        let base = if trimmed.ends_with("/api") {
            trimmed.to_string()
        } else {
            format!("{trimmed}/api")
        };
        // Single-track downloads block until the server queue reaches them,
        // so the timeout must be generous.
        let http = reqwest::Client::builder()
            .timeout(Duration::from_secs(600))
            .build()?;
        Ok(Self { http, base })
    }

    /// Performs a request and returns the parsed JSON body (or `Null` when empty).
    /// Non-2xx responses are turned into a human-readable error string.
    pub async fn request(
        &self,
        method: Method,
        path: &str,
        query: &[(String, String)],
        body: Option<&Value>,
    ) -> Result<Value, String> {
        let url = format!("{}{}", self.base, path);
        let mut req = self.http.request(method, &url);
        if !query.is_empty() {
            req = req.query(query);
        }
        if let Some(body) = body {
            req = req.json(body);
        }
        let resp = req
            .send()
            .await
            .map_err(|e| format!("request to {url} failed: {e}"))?;
        let status = resp.status();
        let text = resp
            .text()
            .await
            .map_err(|e| format!("failed to read response from {url}: {e}"))?;
        if !status.is_success() {
            return Err(format!("HTTP {status}: {text}"));
        }
        if text.trim().is_empty() {
            return Ok(Value::Null);
        }
        Ok(serde_json::from_str(&text).unwrap_or(Value::String(text)))
    }

    pub async fn get(&self, path: &str) -> Result<Value, String> {
        self.request(Method::GET, path, &[], None).await
    }
}
