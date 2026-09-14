use std::time::Duration;

use tonic::transport::Endpoint;
use tracing::{error, info};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    // Determine the provider address and set env for CLI to discover it
    let addr = std::env::var("LETUS_PROVIDER_AWS_ADDR").unwrap_or_else(|_| "127.0.0.1:50051".to_string());
    let endpoint = format!("http://{}", addr);
    unsafe { std::env::set_var("LETUS_PROVIDER_AWS_ENDPOINT", endpoint.clone()); }

    // Start provider server in background
    tokio::spawn(async move {
        if let Err(e) = aws_provider::serve(&addr).await {
            error!("aws-provider exited with error: {}", e);
        }
    });

    // Wait for provider to become ready
    for _ in 0..50u8 { // up to ~5 seconds
        match Endpoint::from_shared(endpoint.clone())?.connect().await {
            Ok(_ch) => {
                info!("Provider is up at {}", endpoint);
                break;
            }
            Err(_e) => {
                tokio::time::sleep(Duration::from_millis(100)).await;
            }
        }
    }

    // Run the actual CLI, forwarding the original CLI args
    yamlet_core::run_cli().await?;

    Ok(())
}
