//! Minimal MCP (JSON-RPC 2.0) request handler, transport-agnostic.

use serde_json::{json, Value};
use tracing::debug;

use crate::client::ApiClient;
use crate::tools::{catalog, Tool};

const SUPPORTED_VERSIONS: &[&str] = &["2025-06-18", "2025-03-26", "2024-11-05"];
const MAX_RESULT_CHARS: usize = 60_000;

const INSTRUCTIONS: &str = "Soundome music library manager. Typical validation flow: \
list_pending_validations -> get_validation_matches / get_validation_youtube_candidates -> \
approve_validation (with metadata fixes) or reject_validation. Imports: import_url, then get_task for collections.";

pub struct McpServer {
    client: ApiClient,
    tools: Vec<Tool>,
}

type RpcError = (i64, String);

impl McpServer {
    pub fn new(client: ApiClient) -> Self {
        Self {
            client,
            tools: catalog(),
        }
    }

    /// Handles a raw JSON payload (single message or batch).
    /// Returns `None` when no response must be sent (notifications only).
    pub async fn handle_payload(&self, text: &str) -> Option<Value> {
        let parsed: Value = match serde_json::from_str(text) {
            Ok(v) => v,
            Err(e) => {
                return Some(error_response(
                    Value::Null,
                    -32700,
                    format!("parse error: {e}"),
                ))
            }
        };
        match parsed {
            Value::Array(items) => {
                let mut out = Vec::new();
                for item in items {
                    if let Some(r) = self.handle_message(item).await {
                        out.push(r);
                    }
                }
                (!out.is_empty()).then_some(Value::Array(out))
            }
            other => self.handle_message(other).await,
        }
    }

    async fn handle_message(&self, msg: Value) -> Option<Value> {
        let method = msg.get("method").and_then(Value::as_str)?;
        let params = msg.get("params").cloned().unwrap_or(Value::Null);
        let Some(id) = msg.get("id").cloned() else {
            debug!("notification received: {method}");
            return None;
        };

        let result: Result<Value, RpcError> = match method {
            "initialize" => Ok(self.initialize(&params)),
            "ping" => Ok(json!({})),
            "tools/list" => Ok(json!({
                "tools": self.tools.iter().map(Tool::descriptor).collect::<Vec<_>>()
            })),
            "tools/call" => self.call_tool(&params).await,
            _ => Err((-32601, format!("method not found: {method}"))),
        };

        Some(match result {
            Ok(r) => json!({ "jsonrpc": "2.0", "id": id, "result": r }),
            Err((code, message)) => error_response(id, code, message),
        })
    }

    fn initialize(&self, params: &Value) -> Value {
        let requested = params.get("protocolVersion").and_then(Value::as_str);
        let version = requested
            .filter(|v| SUPPORTED_VERSIONS.contains(v))
            .unwrap_or(SUPPORTED_VERSIONS[0]);
        json!({
            "protocolVersion": version,
            "capabilities": { "tools": { "listChanged": false } },
            "serverInfo": { "name": "soundome-mcp", "version": env!("CARGO_PKG_VERSION") },
            "instructions": INSTRUCTIONS,
        })
    }

    async fn call_tool(&self, params: &Value) -> Result<Value, RpcError> {
        let name = params
            .get("name")
            .and_then(Value::as_str)
            .ok_or((-32602, "missing tool name".to_string()))?;
        let tool = self
            .tools
            .iter()
            .find(|t| t.name == name)
            .ok_or((-32602, format!("unknown tool: {name}")))?;
        let args = params
            .get("arguments")
            .cloned()
            .unwrap_or_else(|| json!({}));

        let (text, is_error) = match tool.execute(&self.client, &args).await {
            Ok(t) => (t, false),
            Err(e) => (e, true),
        };
        Ok(json!({
            "content": [{ "type": "text", "text": truncate(text) }],
            "isError": is_error,
        }))
    }
}

fn truncate(text: String) -> String {
    if text.chars().count() <= MAX_RESULT_CHARS {
        return text;
    }
    let mut out: String = text.chars().take(MAX_RESULT_CHARS).collect();
    out.push_str("\n… [truncated; use pagination or a narrower query]");
    out
}

fn error_response(id: Value, code: i64, message: String) -> Value {
    json!({ "jsonrpc": "2.0", "id": id, "error": { "code": code, "message": message } })
}
