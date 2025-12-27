fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load .env file from project root (parent of src-tauri)
    // This allows YOUTUBE_API_KEY_PRIMARY etc. to be available at compile time via option_env!
    let env_path = std::path::Path::new("../.env");
    if env_path.exists() {
        dotenvy::from_path(env_path).ok();
        // Re-run build if .env changes
        println!("cargo:rerun-if-changed=../.env");
    }

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
