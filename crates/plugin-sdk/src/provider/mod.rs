// Re-export generated gRPC server modules from OUT_DIR
pub mod provider {
    include!(concat!(env!("OUT_DIR"), "/yamlet.provider.v1.rs"));
}
