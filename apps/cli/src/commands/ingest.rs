use crate::api::ApiClient;

pub async fn ingest(client: &ApiClient, file_path: &str) -> anyhow::Result<()> {
    println!("Ingesting: {}", file_path);
    let result = client.ingest(file_path).await?;

    println!("\n=== Ingest Result ===");
    println!("Title   : {}", result.title);
    println!("Artists : {}", result.artists.join(", "));
    if result.needs_validation {
        println!("Status  : pending manual validation");
    } else {
        println!("Status  : ingested successfully");
    }

    Ok(())
}
