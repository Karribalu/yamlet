#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    yamlet_core::run_cli().await
}
