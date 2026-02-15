<p align="center">
  <h1 align="center">SubTunnel</h1>
  <p align="center">Expose localhost to the internet. Self-hosted ngrok alternative.</p>
</p>

<p align="center">
  <a href="https://github.com/winterwindgames/subtunnel/actions"><img src="https://github.com/winterwindgames/subtunnel/actions/workflows/release.yml/badge.svg" alt="Build"></a>
  <a href="LICENSE"><img src="https://img.shields.io/badge/license-MIT-blue.svg" alt="MIT License"></a>
  <a href="https://subtunnel.dev"><img src="https://img.shields.io/badge/docs-subtunnel.dev-green.svg" alt="Docs"></a>
</p>

---

SubTunnel is a self-hosted tunneling solution written in Rust. Run the server on your own VPS, connect from anywhere, and expose local services to the internet with custom subdomains and automatic HTTPS.

## Features

- **Self-hosted** — run on your own infrastructure, full control over your data
- **Single binary** — ships both `server` and `local` subcommands
- **Custom subdomains** — `myapp.tunnel.example.com` out of the box
- **TLS everywhere** — encrypted control plane, HTTPS via nginx/Let's Encrypt
- **Auto-reconnect** — exponential backoff, survives network interruptions
- **Lightweight** — built with Tokio, yamux multiplexing, minimal dependencies

## Quick Start

### Install

```bash
curl -sSL https://www.subtunnel.dev/install.sh | sh
```

Supports macOS (Apple Silicon & Intel) and Linux (x86_64 & ARM64).

### Connect to a server

```bash
subtunnel local 3000 \
  --to your-server.example.com:7835 \
  --token YOUR_TOKEN \
  --subdomain myapp
```

Your local port 3000 is now live at `https://myapp.your-domain.com`.

## Self-Host the Server

### 1. DNS

Point your domain and a wildcard to your server's IP:

```
A     tunnel.example.com      → YOUR_SERVER_IP
A     *.tunnel.example.com    → YOUR_SERVER_IP
```

### 2. Install & run

```bash
# Install
curl -sSL https://www.subtunnel.dev/install.sh | sh

# Generate a token
openssl rand -hex 16

# Start the server
subtunnel server \
  --domain tunnel.example.com \
  --token YOUR_TOKEN \
  --port 7835 \
  --http-port 8080
```

### 3. nginx (TLS termination)

```nginx
server {
    listen 443 ssl;
    server_name *.tunnel.example.com;

    ssl_certificate /etc/letsencrypt/live/tunnel.example.com/fullchain.pem;
    ssl_certificate_key /etc/letsencrypt/live/tunnel.example.com/privkey.pem;

    location / {
        proxy_pass http://127.0.0.1:8080;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
    }
}
```

For wildcard certs: `certbot certonly --dns-cloudflare -d tunnel.example.com -d *.tunnel.example.com`

### 4. systemd (production)

```ini
[Unit]
Description=SubTunnel Server
After=network.target

[Service]
Type=simple
ExecStart=/usr/local/bin/subtunnel server \
    --domain tunnel.example.com \
    --token YOUR_TOKEN \
    --port 7835 \
    --http-port 8080
Restart=always
RestartSec=5

[Install]
WantedBy=multi-user.target
```

```bash
sudo systemctl enable --now subtunnel
```

## Architecture

```
Internet → nginx (TLS :443) → SubTunnel HTTP (:8080) → route by Host header
                                                         ↕ yamux streams
Client (subtunnel local) ←— TLS + yamux (:7835) ——→ SubTunnel Server
        ↕                                              (control + data)
   localhost:PORT
```

## CLI Reference

### `subtunnel server`

```
Options:
  --domain <DOMAIN>          Base domain for tunnel subdomains (required)
  --token <TOKEN>            Auth token clients must provide
  --port <PORT>              Control plane port [default: 7835]
  --http-port <HTTP_PORT>    HTTP listener port [default: 8080]
  --host <HOST>              Bind address [default: 0.0.0.0]
  --extra-domain <DOMAIN>    Additional domains to accept (repeatable)
```

### `subtunnel local`

```
Arguments:
  <LOCAL_PORT>               Local port to expose

Options:
  --to <HOST:PORT>           Server address (required)
  --token <TOKEN>            Auth token (required)
  --subdomain <NAME>         Request a specific subdomain
  --tls-verify <true|false>  Verify server TLS cert [default: true]
  --tls-ca <PATH>            Custom CA certificate PEM file
```

## Development

```bash
# Build
cargo build

# Run tests
cargo test

# Run server locally
cargo run -- server --domain localhost --token dev --port 7835 --http-port 8080

# Connect a client
cargo run -- local 3000 --to 127.0.0.1:7835 --token dev --tls-verify false --subdomain test
```

## License

MIT — see [LICENSE](LICENSE).
