# Tunnelr — An ngrok Clone in Rust

## Overview
A self-hosted, high-performance tunneling solution written in Rust that exposes local services to the public internet. Like ngrok but fully open source and self-hostable.

## Architecture

### How ngrok-style tunneling works

```
Internet Client → [Public Server :443] → (Tunnel) → [Local Agent] → [Local Service :8080]
```

1. **Agent** (client CLI) runs on the user's machine, connects outbound to the **Server** via a persistent TLS/WebSocket connection
2. **Server** runs on a public VPS, listens for incoming HTTP/TCP traffic on public ports/domains
3. When a request arrives at the server, it's multiplexed through the tunnel connection to the agent
4. Agent forwards the request to the local service and sends the response back

### Key Components

#### 1. Control Connection
- Agent establishes a persistent connection to the server (WebSocket over TLS)
- Used for: authentication, tunnel registration, heartbeats, metadata exchange
- Protocol: Custom binary protocol over WebSocket

#### 2. Data Connections / Multiplexing
- Uses **yamux** (Yet Another Multiplexer) for stream multiplexing over a single TCP connection
- Each incoming client request opens a new yamux stream
- Avoids head-of-line blocking, allows concurrent requests
- Alternative: could use HTTP/2 framing or QUIC

#### 3. Server Components
- **Public listener**: Accepts HTTP/HTTPS/TCP connections from the internet
- **Tunnel manager**: Tracks active tunnels, maps subdomains/ports to agents
- **TLS termination**: Auto SSL via Let's Encrypt (ACME protocol) using `rustls` + `acme-lib`
- **Subdomain routing**: Routes `*.tunnel.example.com` to the correct agent
- **Admin API**: REST API for management, metrics, auth

#### 4. Agent/Client Components
- **CLI interface**: `subtunnel local 8080 --to server.example.com`
- **Connection manager**: Maintains persistent connection with auto-reconnect + exponential backoff
- **Local proxy**: Forwards traffic to local service
- **TUI dashboard**: Real-time connection stats (like ngrok's terminal UI)

### Protocol Design

```
┌─────────────────────────────────────────┐
│           Control Channel               │
│  (Auth, Tunnel CRUD, Heartbeat, Stats)  │
├─────────────────────────────────────────┤
│         Yamux Multiplexer               │
│  ┌─────┐ ┌─────┐ ┌─────┐ ┌─────┐     │
│  │ Str1│ │ Str2│ │ Str3│ │ StrN│     │
│  │(req)│ │(req)│ │(req)│ │(req)│     │
│  └─────┘ └─────┘ └─────┘ └─────┘     │
├─────────────────────────────────────────┤
│         TLS 1.3 (rustls)               │
├─────────────────────────────────────────┤
│              TCP                        │
└─────────────────────────────────────────┘
```

**Control messages** (JSON over the control yamux stream):
```json
{"type": "auth", "token": "secret_key"}
{"type": "tunnel_req", "protocol": "http", "subdomain": "myapp"}
{"type": "tunnel_resp", "url": "https://myapp.tunnel.example.com", "id": "t_abc123"}
{"type": "heartbeat"}
{"type": "heartbeat_ack"}
```

### Features (MVP → Full)

#### MVP (Phase 1)
- [ ] TCP tunneling (agent → server → public port)
- [ ] Single binary for both client and server (`subtunnel server`, `subtunnel local`)
- [ ] Token-based authentication
- [ ] Yamux multiplexing
- [ ] TLS encryption (rustls)
- [ ] Auto-reconnect with exponential backoff
- [ ] Basic CLI with connection info output

#### Phase 2
- [ ] HTTP tunneling with subdomain routing (`*.tunnel.example.com`)
- [ ] Auto HTTPS via Let's Encrypt (ACME)
- [ ] Request/response inspection (TUI dashboard like ngrok)
- [ ] Configurable via TOML file
- [ ] HTTP header rewriting (Host header)

#### Phase 3
- [ ] Web dashboard for server admin
- [ ] Rate limiting & bandwidth throttling
- [ ] Multiple tunnels per agent
- [ ] Custom domains
- [ ] WebSocket passthrough
- [ ] Metrics (Prometheus)
- [ ] Docker images

## Tech Stack

| Component | Library |
|-----------|---------|
| Async runtime | `tokio` |
| Multiplexing | `yamux` |
| TLS | `rustls` + `tokio-rustls` |
| HTTP parsing | `hyper` |
| CLI | `clap` |
| TUI | `ratatui` |
| Serialization | `serde` + `serde_json` |
| Logging | `tracing` + `tracing-subscriber` |
| ACME/Let's Encrypt | `instant-acme` |
| Config | `toml` + `serde` |
| Error handling | `anyhow` + `thiserror` |

## Project Structure

```
subtunnel/
├── Cargo.toml
├── src/
│   ├── main.rs              # CLI entry point (clap)
│   ├── lib.rs               # Shared types & re-exports
│   ├── protocol/
│   │   ├── mod.rs
│   │   ├── messages.rs      # Control message types
│   │   └── codec.rs         # Message framing/serialization
│   ├── server/
│   │   ├── mod.rs
│   │   ├── listener.rs      # Public TCP/HTTP listener
│   │   ├── tunnel_mgr.rs    # Tunnel registry & routing
│   │   ├── auth.rs          # Token auth
│   │   ├── tls.rs           # TLS termination & ACME
│   │   └── http_proxy.rs    # HTTP-aware proxying & subdomain routing
│   ├── client/
│   │   ├── mod.rs
│   │   ├── connector.rs     # Connection to server + reconnect logic
│   │   ├── local_proxy.rs   # Forward to local service
│   │   └── tui.rs           # Terminal UI dashboard
│   ├── transport/
│   │   ├── mod.rs
│   │   ├── mux.rs           # Yamux multiplexer wrapper
│   │   └── tls.rs           # TLS connection setup
│   └── config.rs            # TOML config parsing
├── tests/
│   ├── integration.rs       # End-to-end tunnel tests
│   └── protocol_tests.rs
├── examples/
│   └── basic_tunnel.rs
└── README.md
```

## Reference Implementations
- **bore** (Rust, ~400 LOC, MIT) — minimal TCP tunneling, uses tokio
- **rathole** (Rust) — frp alternative, high performance, noise protocol
- **ngrok v1** (Go, open source) — original ngrok, good protocol reference
- **frp** (Go) — popular self-hosted tunneling

## Performance Goals
- Support 10,000+ concurrent connections per server
- Sub-millisecond overhead per proxied request
- Memory usage < 10MB for agent, < 50MB for server
- Zero-copy proxying where possible (tokio::io::copy)
