# CLAUDE.md — Tunnelr

## Project
Tunnelr is a self-hosted ngrok alternative written in Rust. It creates secure tunnels to expose local services to the public internet.

## Architecture
Read DESIGN.md for full architecture. Key points:
- Single binary: `subtunnel server` and `subtunnel local`
- Yamux for stream multiplexing over a single persistent connection
- TLS 1.3 via rustls
- Tokio async runtime
- Control channel (JSON messages) + data streams (raw TCP proxying)

## Build & Test
```bash
cargo build
cargo test
cargo run -- server --port 7835
cargo run -- local 8080 --to localhost:7835
```

## Code Style
- Use `anyhow` for application errors, `thiserror` for library errors
- Use `tracing` for logging (not `log` or `println!`)
- Prefer `tokio::io::copy_bidirectional` for zero-copy proxying
- All public types need doc comments
- Run `cargo clippy` and `cargo fmt` before committing
