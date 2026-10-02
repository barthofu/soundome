//! Soundome MCP server.
//!
//! Exposes the Soundome HTTP API (library, validations, import, tasks) as MCP
//! tools. It never touches the database or domain layer directly: the Rocket
//! server stays the single entry point and keeps workflow invariants.

mod client;
mod http;
mod server;
mod stdio;
mod tools;

use std::net::SocketAddr;
use std::sync::Arc;

use anyhow::{bail, Context};
use clap::{Parser, ValueEnum};
use tracing_subscriber::EnvFilter;

#[derive(Clone, Copy, ValueEnum)]
enum Transport {
    Stdio,
    Http,
}

#[derive(Parser)]
#[command(name = "soundome-mcp", about = "MCP server for Soundome")]
struct Args {
    /// Transport to serve MCP on.
    #[arg(long, value_enum, default_value = "stdio")]
    transport: Transport,

    /// Soundome server base URL.
    #[arg(
        long,
        env = "SOUNDOME_API_URL",
        default_value = "http://localhost:8777"
    )]
    api_url: String,

    /// Bind address for the HTTP transport.
    #[arg(long, env = "SOUNDOME_MCP_BIND", default_value = "127.0.0.1:8778")]
    bind: SocketAddr,

    /// Bearer token required by the HTTP transport (mandatory on non-loopback binds).
    #[arg(long, env = "SOUNDOME_MCP_TOKEN", hide_env_values = true)]
    http_token: Option<String>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    // Logs go to stderr: stdout is reserved for the stdio transport.
    tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .with_ansi(false)
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();

    let client = client::ApiClient::new(&args.api_url).context("invalid API URL")?;
    let server = Arc::new(server::McpServer::new(client));

    match args.transport {
        Transport::Stdio => stdio::run(server).await,
        Transport::Http => {
            let token = args.http_token.filter(|t| !t.is_empty());
            if token.is_none() && !args.bind.ip().is_loopback() {
                bail!("SOUNDOME_MCP_TOKEN is required when binding to a non-loopback address");
            }
            http::run(server, args.bind, token).await
        }
    }
}
