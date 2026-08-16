# Changelog

Notable changes to SubTunnel. The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and versions follow [Semantic Versioning](https://semver.org/).

## [0.3.1] - 2026-08-16

### Fixed

- Latency: `TCP_NODELAY` is now set on every tunnel TCP connection (agent to server, server control and HTTP accepts, and agent to local service). Nagle's algorithm was buffering the small writes typical of interactive traffic, adding up to a few hundred milliseconds per round trip.

## [0.3.0] - 2026-07-26

### Added

- TOML config file support: define the server address, token, TLS options, and named tunnels in `~/.config/subtunnel/config.toml` (XDG-aware, override with `--config`). Keeping the token in the config file avoids exposing it in the process list.
- `subtunnel run [--all | <names>...]`: start all configured tunnels, or a named subset, in one process. Each tunnel keeps its own connection and reconnect loop. A tunnel that fails hard (auth or registration rejection) stops alone; the others keep running and the process exits nonzero.
- `subtunnel service install|uninstall|start|stop|status|generate`: install the agent as a native systemd service on Linux or a launchd service on macOS so tunnels start on boot and restart after a crash. Run as a normal user for a user service, with sudo for a system service. `generate` prints the unit or plist without installing it. Not supported on Windows yet.

### Changed

- Authentication and tunnel registration rejections are treated as hard errors under `subtunnel run` instead of being retried forever. The `local` command behavior is unchanged.

## [0.2.1] - 2026-07-23

### Changed

- Renamed the token environment variable from `TUNNELR_TOKEN` to `SUBTUNNEL_TOKEN`.
- The releases proxy sends its GitHub auth header only when a token is configured, so it works against the public repository without one.

### Fixed

- Stale repository references in docs and metadata.

## [0.2.0] - 2026-07-22

### Added

- Client heartbeat with dead connection detection and automatic reconnect with exponential backoff.

### Fixed

- Connection stability: dead agent teardown, cancellation-safe control loop, and HTTP router hardening.

## [0.1.0] - 2026-02-13

### Added

- Initial release: `subtunnel server` (control plane, HTTP routing, token auth, TLS) and `subtunnel local` (expose a local port at a subdomain), release workflow with prebuilt binaries, and an install script.

[0.3.1]: https://github.com/ozankasikci/subtunnel/compare/v0.3.0...v0.3.1
[0.3.0]: https://github.com/ozankasikci/subtunnel/compare/v0.2.1...v0.3.0
[0.2.1]: https://github.com/ozankasikci/subtunnel/compare/v0.2.0...v0.2.1
[0.2.0]: https://github.com/ozankasikci/subtunnel/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/ozankasikci/subtunnel/releases/tag/v0.1.0
