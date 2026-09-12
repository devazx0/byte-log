pub fn handle_version() {
    println!("ByteLog v{}", env!("CARGO_PKG_VERSION"));
    println!("Target:          {}", env!("BYTELOG_TARGET_TRIPLE"));
    println!("Git Commit:      {}", env!("BYTELOG_GIT_COMMIT"));
    println!("Rustc Version:   {}", env!("BYTELOG_RUSTC_VERSION"));
    println!("Build Timestamp: {}", env!("BYTELOG_BUILD_TIMESTAMP"));
}
