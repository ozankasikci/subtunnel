<p align="center">
  <h1 align="center">SubTunnel</h1>
  <p align="center"><b>Expose localhost to the internet, on your own domain, on your own server.</b><br>A self-hosted ngrok alternative written in Rust. One binary, MIT licensed.</p>
</p>

<p align="center">
  <a href="https://github.com/ozankasikci/subtunnel/actions"><img src="https://github.com/ozankasikci/subtunnel/actions/workflows/release.yml/badge.svg" alt="Build"></a>
  <a href="LICENSE"><img src="https://img.shields.io/badge/license-MIT-blue.svg" alt="MIT License"></a>
  <a href="https://github.com/ozankasikci/subtunnel/releases"><img src="https://img.shields.io/github/v/release/ozankasikci/subtunnel" alt="Latest release"></a>
  <a href="https://subtunnel.dev"><img src="https://img.shields.io/badge/docs-subtunnel.dev-green.svg" alt="Docs"></a>
</p>

---

SubTunnel is a **self-hosted tunneling tool**: run the server on any VPS, connect from anywhere, and your local port is live at `https://myapp.your-domain.com`. Perfect for **testing webhooks**, **sharing dev builds**, and **reaching services behind NAT**. None of your traffic routes through a third party.

```console
$ subtunnel local 3000 --to tunnel.example.com:7835 --token TOKEN --subdomain myapp

  subtunnel v0.3.0
  Status:     connected
  Forwarding: https://myapp.tunnel.example.com -> localhost:3000
```

## Why SubTunnel?

- **Your infrastructure, your data.** Traffic flows through *your* VPS, not a SaaS. No request limits, no bandwidth caps, no third party in the middle.
- **Your domain.** Every tunnel gets a wildcard subdomain on a domain you own. Professional URLs for demos and webhook endpoints.
- **One static binary.** Written in Rust on Tokio with yamux multiplexing. No runtime, no dependencies, ~2 MB download.
- **Boringly standard TLS.** nginx + Let's Encrypt terminate HTTPS with tooling you already trust. The control plane is TLS-encrypted with token auth.
- **Auto-reconnect.** Clients survive network interruptions with exponential backoff.
- **MIT licensed.** Read the code, fork it, ship it. No open-core bait, no license traps.

## How it works

```
                    ┌────────────────────────── your VPS ──────────────────────────┐
https://myapp.…  →  │ nginx (TLS :443) → subtunnel server (:8080, routes by Host)  │
                    │                        ↕ yamux streams over TLS (:7835)      │
                    └──────────────────────────────────────────────────────────────┘
                                             ↕
                       subtunnel local (your machine, behind NAT/firewall)
                                             ↕
                                     localhost:3000
```

The client dials **out** to the server's control port and registers a subdomain, so it works behind NATs and firewalls with zero inbound configuration. Incoming HTTPS requests are matched by `Host` header and multiplexed back to your machine over a single TLS connection.

## Quick Start

### 1. Install the CLI

```bash
curl -sSL https://www.subtunnel.dev/install.sh | sh
```

Supports macOS (Apple Silicon & Intel) and Linux (x86_64 & ARM64). Windows binaries are on the [releases page](https://github.com/ozankasikci/subtunnel/releases).

### 2. Connect to a server

```bash
subtunnel local 3000 \
  --to tunnel.example.com:7835 \
  --token YOUR_TOKEN \
  --subdomain myapp
```

Your local port 3000 is now live at `https://myapp.tunnel.example.com`.

> Don't have a server yet? Self-hosting takes about 15 minutes. See below.

## Start on boot

The client can read one or more tunnels from a TOML config file. The default path is your platform config directory plus `subtunnel/config.toml`. On Linux this honors `XDG_CONFIG_HOME`. On macOS, SubTunnel uses `~/.config/subtunnel/config.toml` when that file exists, then falls back to the platform config directory. Pass `--config <path>` to override it.

```toml
server = "tunnel.example.com:7835"
token = "YOUR_TOKEN"
tls_verify = true
# tls_ca = "/path/to/ca.pem"

[tunnels.myapp]
local_port = 3000
subdomain = "myapp"

[tunnels.api]
local_port = 8080
```

Keeping the token in this file avoids exposing it in the process list. Protect the file with permissions appropriate for your user.

Start every configured tunnel in one process, or name a subset:

```bash
subtunnel run --all
subtunnel run myapp api
subtunnel run --all --config /absolute/path/to/config.toml
```

Install the agent as a native systemd service on Linux or a launchd service on macOS:

```bash
subtunnel service install --config /absolute/path/to/config.toml
subtunnel service status
```

Run the install command as your normal user for a user service. On Linux, user services start at login; run `loginctl enable-linger $USER` to make the service start at boot. Run the install command with `sudo` for a system service. The system service uses the config path supplied to the sudo command, so use an absolute path that root can read. The service starts on boot and restarts the SubTunnel agent after a crash.

Use `subtunnel service generate systemd --config /absolute/path/to/config.toml` or replace `systemd` with `launchd` to review and customize the definition before installing it.

SubTunnel supervises only the tunnel agent. It does not start or supervise your application. If a systemd tunnel must wait for your app, customize the generated unit with `After=myapp.service`.

## Self-Host the Server

Everything runs on one small VPS (EC2, Hetzner, DigitalOcean, anything with a public IP). You need a domain and ports 80, 443, and 7835 reachable.

### 1. DNS

Point your tunnel domain and a wildcard to your server's IP:

```
A     tunnel.example.com      → YOUR_SERVER_IP
A     *.tunnel.example.com    → YOUR_SERVER_IP
```

> **Tip:** if your apex domain points elsewhere (e.g. a website host), that's fine. Just have clients connect via a name that resolves to the tunnel box, like `tunnel.example.com:7835`, never the apex.

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

## CLI Reference

Four focused command areas cover the server, one-off clients, configured tunnels, and native service management.

### `subtunnel server`

Run the public-facing server on your VPS.

| Flag | Description |
|---|---|
| `--port` | Control-plane listen port. Default: `7835` |
| `--http-port` | HTTP listener receiving proxied traffic from nginx. Default: `8080` |
| `--host` | Bind address. Default: `0.0.0.0` |
| `--domain` | **Required.** Domain for tunnel subdomains |
| `--extra-domain` | Additional accepted domain (repeatable) |
| `--token` | Auth token clients must provide. Env: `SUBTUNNEL_TOKEN` |
| `--tls-cert` / `--tls-key` | TLS certificate / private key PEM paths for the control plane |

### `subtunnel local`

Expose a local port through a SubTunnel server.

| Flag | Description |
|---|---|
| `<port>` | Positional. Local port to expose |
| `--to` | Server address, `host:port` |
| `--token` | Auth token. Env: `SUBTUNNEL_TOKEN` |
| `--subdomain` | Request a specific subdomain |
| `--tls-verify` | Verify server TLS cert. Default: `true` (set `false` for self-signed) |
| `--tls-ca` | Custom CA certificate PEM path |

### `subtunnel run`

Start configured tunnels in one process. Each tunnel keeps its own connection and reconnect loop.

| Flag | Description |
|---|---|
| `<tunnel>...` | Optional tunnel names. Omit to start all configured tunnels |
| `--all` | Start all configured tunnels |
| `--config` | Override the platform config file path |

### `subtunnel service`

Manage the client agent as a native systemd or launchd service.

| Subcommand | Description |
|---|---|
| `install` | Validate the config, install the native service, and enable it at boot |
| `uninstall` | Stop, disable, and remove the native service |
| `start` / `stop` | Start or stop the installed service |
| `status` | Show service status |
| `generate <systemd\|launchd>` | Print a service definition without installing it |

Use `--config <path>` with `install` or `generate` to override the default config path. Service management is not supported yet on Windows.

## Comparison

The honest differences:

| | SubTunnel | ngrok | frp | Cloudflare Tunnel |
|---|---|---|---|---|
| Self-hosted | ✅ | server is SaaS | ✅ | ❌ (Cloudflare edge) |
| Your own domain | ✅ | paid plans | ✅ | ✅ |
| Open source | ✅ MIT | ❌ | ✅ Apache-2.0 | client only |
| Setup effort | ~15 min (VPS + DNS + nginx) | none (managed) | more config surface | Cloudflare account |
| Request inspector | not yet | ✅ | ❌ | ❌ |
| Traffic through third party | ❌ | ✅ | ❌ | ✅ |

If you want zero setup, use ngrok. If you want your traffic on your own box with a single-purpose, easy-to-read codebase, SubTunnel is for you.

## Use cases

- **Webhook development.** Give Stripe/GitHub/Twilio a stable HTTPS URL that hits your laptop.
- **Demo links.** Share `https://demo.your-domain.com` with a client without deploying.
- **Mobile app backends.** Point a TestFlight build at your local API.
- **Home lab / IoT.** Reach services behind CGNAT without port forwarding.

## Roadmap

Planned, in rough order. Issues and PRs welcome:

- TCP tunnel support (databases, SSH)
- Local request inspector / replay
- Windows support in the install script

## Contributing

Bug reports and PRs are welcome. The codebase is a small Rust workspace (`crates/cli`: client, server, and protocol) plus the docs site (`apps/web`, Next.js). Run the tests with:

```bash
cargo test --workspace
```

## License

[MIT](LICENSE). Do whatever you want with it.
