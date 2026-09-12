# Contributing to ByteLog

Thank you for your interest in improving ByteLog!

---

## Development Standards

### 1. Code Quality
Before submitting code, ensure all formatting, linter, and testing checks pass:
```bash
cargo fmt -- --check
cargo clippy --all-targets -- -D warnings
cargo test --all-targets
```

### 2. Privacy First
Any contribution that introduces telemetry, stores raw text/characters, or inspects unencrypted user file contents will be immediately rejected.

### 3. Cross-Platform Guardrails
Always ensure OS-specific APIs are wrapped in conditional compilation guards:
```rust
#[cfg(windows)]
// Win32 code
#[cfg(target_os = "macos")]
// Cocoa / CoreGraphics code
#[cfg(target_os = "linux")]
// X11 / Wayland code
```
