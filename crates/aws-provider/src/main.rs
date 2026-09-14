#![feature(allocator_api)]

use std::alloc::Global;
use std::error::Error;

#[tokio::main]
async fn main() ->  Result<(), Box<dyn Error+Send+Sync, Global>> {
    let addr = std::env::var("LETUS_PROVIDER_AWS_ADDR").unwrap_or_else(|_| "127.0.0.1:50051".to_string());
    aws_provider::serve(&addr).await?;
    Ok(())
}
