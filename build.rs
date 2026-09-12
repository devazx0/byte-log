use std::process::Command;

fn main() {
    let target = std::env::var("TARGET").unwrap_or_else(|_| "unknown".to_string());
    println!("cargo:rustc-env=BYTELOG_TARGET_TRIPLE={}", target);
    println!("cargo:rustc-env=PCSTATS_TARGET_TRIPLE={}", target);

    let now = match std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH) {
        Ok(d) => format!("Unix timestamp: {}", d.as_secs()),
        Err(_) => "unknown".to_string(),
    };
    println!("cargo:rustc-env=BYTELOG_BUILD_TIMESTAMP={}", now);
    println!("cargo:rustc-env=PCSTATS_BUILD_TIMESTAMP={}", now);

    let git_commit = Command::new("git")
        .args(["rev-parse", "--short", "HEAD"])
        .output()
        .ok()
        .and_then(|output| {
            if output.status.success() {
                String::from_utf8(output.stdout).ok()
            } else {
                None
            }
        })
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| "release".to_string());
    println!("cargo:rustc-env=BYTELOG_GIT_COMMIT={}", git_commit);
    println!("cargo:rustc-env=PCSTATS_GIT_COMMIT={}", git_commit);

    let rustc_ver = Command::new("rustc")
        .arg("--version")
        .output()
        .ok()
        .and_then(|output| {
            if output.status.success() {
                String::from_utf8(output.stdout).ok()
            } else {
                None
            }
        })
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| "rustc 1.98+".to_string());
    println!("cargo:rustc-env=BYTELOG_RUSTC_VERSION={}", rustc_ver);
    println!("cargo:rustc-env=PCSTATS_RUSTC_VERSION={}", rustc_ver);

    println!("cargo:rerun-if-changed=migrations/");
}
