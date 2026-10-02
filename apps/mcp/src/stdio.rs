//! stdio transport: one JSON-RPC message per line on stdin/stdout.

use std::sync::Arc;

use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::sync::Mutex;
use tokio::task::JoinSet;
use tracing::error;

use crate::server::McpServer;

pub async fn run(server: Arc<McpServer>) -> anyhow::Result<()> {
    let mut lines = BufReader::new(tokio::io::stdin()).lines();
    let out = Arc::new(Mutex::new(tokio::io::stdout()));
    let mut tasks = JoinSet::new();

    while let Some(line) = lines.next_line().await? {
        if line.trim().is_empty() {
            continue;
        }
        let server = Arc::clone(&server);
        let out = Arc::clone(&out);
        // Concurrent handling: a long import must not block other requests.
        tasks.spawn(async move {
            if let Some(resp) = server.handle_payload(&line).await {
                let mut payload = resp.to_string();
                payload.push('\n');
                let mut out = out.lock().await;
                if let Err(e) = out.write_all(payload.as_bytes()).await {
                    error!("failed to write to stdout: {e}");
                }
                let _ = out.flush().await;
            }
        });
    }
    while tasks.join_next().await.is_some() {}
    Ok(())
}
