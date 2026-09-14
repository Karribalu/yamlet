
fn main() {
    // Compile the provider.proto to generate gRPC client code
    println!("cargo:rerun-if-changed=proto/provider.proto");
    tonic_build::configure()
        .build_server(true)
        .out_dir(std::env::var("OUT_DIR").unwrap())
        .compile_protos(&["proto/provider.proto"], &["proto"]) // proto include path
        .expect("Failed to compile proto files");
}


