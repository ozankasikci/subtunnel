# SubTunnel Security Plan

> Last updated: 2026-02-12

## 1. Current Vulnerabilities Assessment

### Critical

| Vulnerability | Severity | Location |
|---|---|---|
| **NoVerifier TLS client** | 🔴 Critical | `client/connector.rs` — accepts ANY server cert, enabling trivial MITM attacks |
| **Hardcoded ServerName "subtunnel"** | 🔴 Critical | `client/connector.rs` — SNI doesn't match real hostname, breaks proper TLS validation |
| **Shared static token auth** | 🟠 High | Single `TUNNELR_TOKEN` env var, no rotation, no per-user isolation |
| **Self-signed certs only** | 🟠 High | `server/mod.rs` — no option to load real certificates |

### Medium

| Vulnerability | Severity | Notes |
|---|---|---|
| No rate limiting on control plane | 🟡 Medium | Unauthenticated clients can open unlimited TLS+yamux sessions |
| No audit logging | 🟡 Medium | No record of auth attempts, tunnel creation, or connection events |
| No security headers on HTTP responses | 🟡 Medium | HTTP listener returns proxied content without protective headers |
| Yamux without resource limits | 🟡 Medium | No max stream count or window size limits configured |

### Low

| Vulnerability | Severity | Notes |
|---|---|---|
| No connection timeout on TLS handshake | 🟢 Low | Slow-loris style attack possible on control port |
| Cert generated fresh on every restart | 🟢 Low | Self-signed cert changes each run; no pinning possible |

---

## 2. TLS Fix Plan

### Server Side
- Add `--tls-cert` and `--tls-key` CLI flags to load PEM cert/key files (Let's Encrypt)
- Keep self-signed cert generation as fallback when no cert files provided
- Log clearly whether real or self-signed certs are in use

### Client Side
- Add `--tls-verify` flag (default: `true`) — when true, use OS/system root CA store via `webpki-roots`
- Add `--tls-ca` flag to specify a custom CA PEM file (e.g., for self-hosted with private CA)
- When `--tls-verify=false`, use `NoVerifier` (explicit opt-in for self-signed dev setups)
- Extract hostname from `--to` address for `ServerName` instead of hardcoding "subtunnel"

### Migration Path
1. ✅ Phase 1 (this PR): Add flags, default to verified TLS
2. Phase 2: ACME/Let's Encrypt auto-provisioning on server
3. Phase 3: mTLS for node-to-node communication

---

## 3. Token Security

### Current State
- Single shared `TUNNELR_TOKEN` compared with constant-time equality
- No expiration, rotation, or per-user scoping
- Token transmitted over (currently unverified) TLS

### Recommendations
- **Short-term**: Fix TLS so the token is at least protected in transit (this PR)
- **Medium-term**: Move to per-user API keys (`stk_live_*` format) with SHA-256 hashed storage (see ARCHITECTURE.md §2)
- **Long-term**: JWT with short-lived access tokens + refresh flow

*Auth improvements are handled by the auth agent — not in scope here.*

---

## 4. Connection Security (Yamux / Control Plane)

### Current State
- Yamux sessions have no configured stream or window limits
- Control stream is authenticated once; no re-auth or session expiry

### Recommendations
- Set `yamux::Config::max_num_streams` to a reasonable limit (e.g., 256)
- Add idle timeout to yamux sessions (drop after 5 min no activity beyond heartbeat)
- Consider per-stream byte counting for bandwidth enforcement
- TLS session tickets should be disabled for forward secrecy (rustls default is fine)

---

## 5. Recommended Security Headers for HTTP Responses

The HTTP listener should inject these headers on proxied responses:

```
X-Content-Type-Options: nosniff
X-Frame-Options: DENY
X-XSS-Protection: 0
Strict-Transport-Security: max-age=31536000; includeSubDomains
Referrer-Policy: strict-origin-when-cross-origin
```

For the management API (future):
```
Content-Security-Policy: default-src 'none'
Cache-Control: no-store
```

---

## 6. Audit Logging Requirements

### Events to Log

| Event | Fields |
|---|---|
| `auth.attempt` | peer_ip, success, token_prefix, timestamp |
| `auth.failure` | peer_ip, reason, timestamp |
| `tunnel.created` | tunnel_id, subdomain, user_id, peer_ip |
| `tunnel.closed` | tunnel_id, reason, duration, bytes_in, bytes_out |
| `connection.proxied` | tunnel_id, client_ip, bytes, duration |
| `server.started` | version, config (redacted), tls_mode |
| `server.stopped` | reason, uptime |

### Format
Use structured JSON logging via `tracing` with `tracing-subscriber` JSON formatter in production mode. Example:

```json
{"timestamp":"2026-02-12T20:41:00Z","level":"INFO","event":"tunnel.created","tunnel_id":"t_abc123","subdomain":"myapp","peer_ip":"1.2.3.4"}
```

### Storage
- Phase 1: Structured log files (current `tracing` setup, enhanced with above events)
- Phase 2: PostgreSQL `audit_log` table (see ARCHITECTURE.md §9)
