use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // No system-wide protoc install required: use the vendored binary.
    let protoc = protoc_bin_vendored::protoc_bin_path()?;
    std::env::set_var("PROTOC", &protoc);

    let proto_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../proto");
    let protos = [
        "identity",
        "auth",
        "pickup",
        "scheduling",
        "routing",
        "fleet",
        "tracking",
        "payment",
        "compliance",
        "notification",
        "geo",
    ]
    .map(|name| proto_dir.join(format!("{name}.proto")));

    for proto in &protos {
        println!("cargo:rerun-if-changed={}", proto.display());
    }

    // FileDescriptorSet is embedded in the binary for gRPC server reflection.
    let descriptor = PathBuf::from(std::env::var("OUT_DIR")?).join("smarttrash_descriptor.bin");

    tonic_prost_build::configure()
        .build_server(true)
        .build_client(true)
        .file_descriptor_set_path(&descriptor)
        .compile_protos(&protos, &[proto_dir])?;

    Ok(())
}
