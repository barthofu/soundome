//! Streamable-HTTP transport (JSON responses only): `POST /mcp`.

use std::net::SocketAddr;
use std::sync::Arc;

use axum::extract::State;
use axum::http::{header::AUTHORIZATION, HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use tracing::{info, warn};

use crate::server::McpServer;

#[derive(Clone)]
struct HttpState {
    server: Arc<McpServer>,
    token: Option<Arc<String>>,
}

pub async fn run(
    server: Arc<McpServer>,
    bind: SocketAddr,
    token: Option<String>,
) -> anyhow::Result<()> {
    let state = HttpState {
        server,
        token: token.map(Arc::new),
    };
    let app = Router::new()
        .route(
            "/mcp",
            post(mcp_post)
                .get(method_not_allowed)
                .delete(method_not_allowed),
        )
        .route("/health", get(|| async { "ok" }))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(bind).await?;
    info!("MCP HTTP transport listening on http://{bind}/mcp");
    axum::serve(listener, app).await?;
    Ok(())
}

async fn method_not_allowed() -> StatusCode {
    StatusCode::METHOD_NOT_ALLOWED
}

async fn mcp_post(State(state): State<HttpState>, headers: HeaderMap, body: String) -> Response {
    if let Some(expected) = &state.token {
        let provided = headers
            .get(AUTHORIZATION)
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.strip_prefix("Bearer "))
            .unwrap_or("");
        if !constant_time_eq(provided.as_bytes(), expected.as_bytes()) {
            warn!("rejected MCP HTTP request: invalid bearer token");
            return StatusCode::UNAUTHORIZED.into_response();
        }
    }
    match state.server.handle_payload(&body).await {
        Some(resp) => Json(resp).into_response(),
        None => StatusCode::ACCEPTED.into_response(),
    }
}

fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    a.iter().zip(b).fold(0u8, |acc, (x, y)| acc | (x ^ y)) == 0
}
