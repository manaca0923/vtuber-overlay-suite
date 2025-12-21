fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Tauri build
    tauri_build::build();

    // gRPC proto compilation
    // Only compile if proto file exists (to avoid breaking builds without proto)
    let proto_path = "proto/youtube_live_chat.proto";
    if std::path::Path::new(proto_path).exists() {
        tonic_build::configure()
            .build_server(false) // We only need client
            .compile_protos(&[proto_path], &["proto/"])?;
    }

    Ok(())
}
