# SubTunnel — Product Plan

> Self-hosted ngrok alternative. Your tunnels, your servers, your rules.

---

## 1. Competitive Analysis

### What ngrok does well
- **Instant onboarding** — `ngrok http 80` and you're live in seconds. Zero config.
- **Brand recognition** — "ngrok" is basically a verb among developers. Millions of users.
- **Evolving beyond tunnels** — They've repositioned as a "Universal Gateway" with traffic policies, API gateway features, WAF, rate limiting, and now AI Gateway.
- **Native SDKs** — Embed tunnels directly in Go, Python, Rust, Java code. No sidecar needed.
- **Polished DX** — Web inspector for replaying requests, clean dashboard, good docs.
- **Pay-as-you-go pricing** — $20/mo base + usage. No giant enterprise commitment required.

### What ngrok does poorly
- **Vendor lock-in** — All traffic routes through ngrok's cloud. You can't self-host. Your data transits their servers.
- **Free tier is crippled** — 1 active endpoint, 1 GB bandwidth, 20K HTTP requests/month, interstitial warning page on free URLs. Barely usable.
- **Pricing opacity** — The pricing page is a maze of "charged against credit" line items. Hard to predict monthly cost.
- **No self-hosted option** — Enterprise "on-premises" exists but requires contacting sales. Not open source.
- **Complexity creep** — They've grown into an "all-in-one cloud networking platform" — overkill for teams who just want tunnels.
- **Random URL problem** — Free tier gives you `abc123.ngrok-free.app` which changes every restart. Custom domains require paid plans.
- **Data sovereignty** — No control over which region your traffic routes through. Problematic for GDPR, healthcare, finance.

### Market gaps SubTunnel fills
| Gap | SubTunnel Answer |
|-----|-----------------|
| No self-hosted option | Deploy on your own infra — Docker, bare metal, K8s |
| Vendor lock-in | Open source, MIT licensed, zero phone-home |
| Data sovereignty | Traffic stays on your network. Pick your region. |
| Unpredictable pricing | Self-hosted = your server costs. Flat, predictable. |
| Overkill for simple tunnels | SubTunnel does tunnels excellently. Not trying to be an API gateway. |
| Team/org tunnel sharing | Built-in multi-user with team workspaces from day one |

### Competitors landscape
| Product | Self-hosted? | Open source? | Notes |
|---------|-------------|-------------|-------|
| ngrok | No (enterprise only) | No | Market leader, increasingly complex |
| Cloudflare Tunnel | No | No | Free but locked to Cloudflare ecosystem |
| frp | Yes | Yes (Go) | Popular but no web UI, minimal auth |
| Tailscale Funnel | No | Partial | Requires Tailscale network |
| localtunnel | Yes | Yes (Node) | Unmaintained, basic |
| bore | Yes | Yes (Rust) | Minimal, no auth/dashboard |
| Expose | Yes | Yes (PHP) | Laravel-centric |
| rathole | Yes | Yes (Rust) | Low-level, no UI |
| pgrok | Yes | Yes (Go) | Pomerium-based, niche |

**Key insight:** The self-hosted tunnel space has many half-finished projects. No one has combined self-hosted + great UX + team features + modern dashboard into a polished product. That's SubTunnel.

---

## 2. Target Users

### Persona 1: Solo Developer ("Weekend Hacker")
- **Who:** Freelancer or indie dev building side projects
- **Pain:** ngrok free tier is too limited, doesn't want to pay $20/mo for hobby projects
- **Need:** Quick tunnel setup, custom domains, no usage limits
- **SubTunnel hook:** Free forever on your own $5/mo VPS

### Persona 2: Startup Backend Team ("The DevOps-Lite Team")
- **Who:** 3-10 person dev team at early-stage startup
- **Pain:** Sharing webhook endpoints, demoing to clients, testing integrations. ngrok costs add up across team.
- **Need:** Team tunnel management, persistent URLs, access control
- **SubTunnel hook:** One server, unlimited team members, unlimited tunnels

### Persona 3: Platform Engineer ("Infrastructure Owner")
- **Who:** DevOps/platform eng at mid-size company (50-500 employees)
- **Pain:** Security team won't approve traffic routing through ngrok's cloud. Compliance requirements.
- **Need:** Self-hosted, audit logs, SSO, RBAC
- **SubTunnel hook:** Deploy in your VPC. SOC 2 friendly. Full control.

### Persona 4: IoT / Edge Developer ("Device Whisperer")
- **Who:** Building connected devices, needs remote access to devices behind NAT
- **Pain:** Can't open ports, ngrok agent per device gets expensive
- **Need:** Lightweight agent, TCP tunnels, persistent connections
- **SubTunnel hook:** Tiny agent binary, unlimited devices, your own relay server

### Persona 5: Agency / Consultant ("The Demo Machine")
- **Who:** Ships client work, needs to demo localhost to clients daily
- **Pain:** Ugly ngrok URLs, interstitial pages embarrass them in client demos
- **Need:** Custom branded domains, clean URLs, no interstitial
- **SubTunnel hook:** Your domain, your brand, zero interstitials ever

---

## 3. Value Proposition

### One-liner
> **Your tunnels. Your server. No limits.**

### Elevator pitch
SubTunnel is a self-hosted tunneling server you deploy on your own infrastructure. Get all the power of ngrok — secure tunnels, custom domains, web inspector, team management — without sending your traffic through someone else's cloud. Open source, no usage limits, no vendor lock-in.

### Key differentiators

| | ngrok | SubTunnel |
|---|-------|-----------|
| Hosting | Their cloud | Your server |
| Pricing | $20/mo + usage fees | Free (open source) + your server cost |
| Data routing | Through ngrok's network | Stays on your infra |
| Usage limits | Bandwidth, requests, endpoints all metered | Unlimited (your hardware is the limit) |
| Custom domains | Paid plans only | Free, bring your own domain |
| Team members | 3 included, then extra cost | Unlimited |
| Source code | Proprietary | MIT License |
| Interstitial page | Yes, on free tier | Never |
| Compliance | Contact sales for BAA/HIPAA | You own the infra, you control compliance |

---

## 4. Marketing Website Structure

### 4.1 Homepage (`/`)
**Purpose:** Convert visitors to sign-ups or GitHub stars

**Sections:**
1. **Hero** — Headline + subheadline + 2 CTAs (GitHub / Get Started)
2. **Problem/Solution** — "Why not ngrok?" pain points → SubTunnel answers
3. **How It Works** — 3-step: Deploy server → Install CLI → Create tunnel (with terminal animation)
4. **Features Grid** — 6 cards: Self-hosted, Custom Domains, Team Management, Web Inspector, TCP/HTTP/WS, API & SDK
5. **Comparison Table** — SubTunnel vs ngrok vs Cloudflare Tunnel vs frp
6. **Social Proof** — GitHub stars counter, testimonials, "Used by" logos
7. **CTA Footer** — "Deploy SubTunnel in 5 minutes" + `docker run` command

### 4.2 Features (`/features`)
**Purpose:** Deep-dive on capabilities for evaluating teams

**Sections:**
- Secure Tunnels (HTTP, HTTPS, TCP, WebSocket)
- Custom Domains & Automatic TLS
- Web Traffic Inspector
- Team & Organization Management
- API & CLI
- Dashboard & Analytics
- Access Control & Auth
- Kubernetes Operator (future)

### 4.3 Pricing (`/pricing`)
**Purpose:** Show that self-hosted is free, managed cloud is optional

**Sections:**
- 3 tiers: Community (free) / Pro (self-hosted support) / Cloud (managed)
- Feature comparison table
- FAQ
- CTA: "Start free on GitHub"

### 4.4 Docs (`/docs`)
**Purpose:** Get users from zero to tunnel in < 5 minutes

**Structure:**
- Quick Start (Docker, binary, from source)
- Server Configuration
- CLI Reference
- Custom Domains
- Team Management
- API Reference
- Troubleshooting
- Migration from ngrok

### 4.5 Blog (`/blog`)
**Purpose:** SEO, thought leadership, changelog

**Seed articles:**
- "Why We Built SubTunnel"
- "Migrating from ngrok to SubTunnel in 10 Minutes"
- "Self-Hosted Tunnels: A Security Deep Dive"
- "SubTunnel vs ngrok: Complete Comparison (2026)"
- "How to Expose Your Localhost to the Internet (5 Methods)"

### 4.6 Use Cases (`/use-cases`)
- Webhook Development
- Client Demos
- IoT Remote Access
- CI/CD Preview Environments
- Remote Database Access
- Self-Hosted API Gateway

### 4.7 Compare Pages (`/compare/ngrok`, `/compare/cloudflare-tunnel`, `/compare/frp`)
**Purpose:** SEO landing pages targeting "[competitor] alternative" searches

### 4.8 About / Open Source (`/open-source`)
- MIT license explanation
- Contributing guide
- Roadmap
- Governance

---

## 5. Dashboard Features

### 5.1 MVP Dashboard
- **Active Tunnels** — List with status, URL, upstream target, created time, traffic stats
- **Tunnel Details** — Request log, headers, body inspection, replay button
- **Domains** — Add/remove custom domains, DNS instructions, TLS status
- **API Keys** — Create/revoke tokens for CLI authentication
- **Account Settings** — Email, password, 2FA

### 5.2 v1 Dashboard
Everything in MVP plus:
- **Team Management** — Invite members, roles (admin/member/viewer)
- **Usage Analytics** — Requests/day, bandwidth/day, top tunnels, response times (charts)
- **Audit Log** — Who created/deleted what, when
- **Tunnel Templates** — Save tunnel configs as reusable templates
- **Notifications** — Tunnel down alerts (email, webhook, Slack)

### 5.3 v2 Dashboard
Everything in v1 plus:
- **Organizations** — Multiple teams under one org
- **SSO/SAML** — Enterprise identity provider integration
- **RBAC** — Fine-grained permissions per tunnel/domain
- **Billing** (cloud tier only) — Usage dashboard, invoices, payment methods
- **Traffic Policies** — Basic rate limiting, IP allowlists, header injection
- **Agent Management** — See connected agents, versions, health status

### Dashboard Design Principles
- Dark mode by default (developers)
- Real-time updates via WebSocket (tunnel status, request log)
- Mobile-responsive (check tunnel status from phone)
- Keyboard shortcuts for power users
- One-click copy for tunnel URLs

---

## 6. Pricing Tiers

### Community — Free forever
> Self-hosted, open source, no limits

- ✅ Unlimited tunnels
- ✅ Unlimited bandwidth
- ✅ Unlimited team members
- ✅ Custom domains
- ✅ Web inspector
- ✅ HTTP, HTTPS, TCP, WebSocket tunnels
- ✅ REST API
- ✅ Community support (GitHub Issues, Discord)
- **CTA:** `docker run -d subtunnel/server`

### Pro — $29/month
> Priority support + advanced features for self-hosted

- Everything in Community, plus:
- ✅ Priority email support (24h response SLA)
- ✅ SSO/SAML integration
- ✅ Audit logging
- ✅ Advanced analytics & retention (90 days)
- ✅ Tunnel templates
- ✅ Alerting integrations (Slack, PagerDuty, webhooks)
- ✅ License for commercial use with > 50 users
- **CTA:** "Start 14-day free trial"

### Cloud — $49/month + usage
> We host it for you. Zero ops.

- Everything in Pro, plus:
- ✅ Managed SubTunnel server (we run it)
- ✅ Global edge network (US, EU, Asia)
- ✅ 99.9% uptime SLA
- ✅ Automatic backups
- ✅ 10 custom domains included (additional $2/mo each)
- ✅ 50GB bandwidth included (additional $0.10/GB)
- ✅ Dedicated support channel
- **CTA:** "Get started — no server needed"

### Enterprise — Custom
- On-premises deployment assistance
- Custom SLA
- BAA for HIPAA
- Dedicated support engineer
- Custom integrations
- Volume licensing
- **CTA:** "Contact us"

---

## 7. Landing Page Copy

### Hero Section
```
# Your tunnels. Your server. No limits.

The open-source, self-hosted alternative to ngrok.
Expose your localhost to the internet — without routing
traffic through someone else's cloud.

[⭐ Star on GitHub]  [Get Started →]
```

### Sub-hero (problem statement)
```
## ngrok is great. Until it isn't.

Your traffic routes through their servers. Your free tunnel gets an
interstitial page. You hit bandwidth limits mid-demo. Custom domains
cost extra. And your security team just said no.

SubTunnel gives you everything ngrok does — on infrastructure you control.
```

### How It Works
```
## Live in 3 commands.

# 1. Deploy the server
$ docker run -d -p 443:443 subtunnel/server

# 2. Install the CLI
$ curl -sf https://get.subtunnel.dev | sh

# 3. Create a tunnel
$ subtunnel http 3000
→ https://myapp.yourdomain.com
```

### Features Section
```
## Everything you need. Nothing you don't.

🔒 **Self-Hosted**
Your server, your network, your rules. Traffic never
leaves your infrastructure.

🌐 **Custom Domains**
Bring your own domain. Automatic TLS via Let's Encrypt.
No ugly random URLs.

👥 **Team Management**
Invite your team. Share tunnels. Control access.
No per-seat charges.

🔍 **Traffic Inspector**
See every request in real-time. Inspect headers,
bodies, timing. Replay with one click.

⚡ **All Protocols**
HTTP, HTTPS, TCP, WebSocket. If it speaks TCP,
SubTunnel can tunnel it.

🔑 **API & CLI**
Full REST API. Powerful CLI. Automate everything.
CI/CD friendly.
```

### Comparison Section
```
## SubTunnel vs ngrok — honest comparison

|                    | SubTunnel          | ngrok Free      | ngrok Paid       |
|--------------------|--------------------|-----------------|------------------|
| Price              | Free (self-hosted) | Free            | From $20/mo      |
| Custom domains     | ✅ Free            | ❌              | ✅ Paid          |
| Bandwidth          | Unlimited          | 1 GB/mo         | 5 GB + overage   |
| Team members       | Unlimited          | 1               | 3 + overage      |
| Data sovereignty   | ✅ Your servers    | ❌ ngrok cloud  | ❌ ngrok cloud   |
| Interstitial page  | Never              | Yes             | No               |
| Open source        | ✅ MIT             | ❌              | ❌               |
| Concurrent tunnels | Unlimited          | 1               | 3+               |
```

### Social Proof Section
```
## Developers love SubTunnel

"Replaced ngrok across our entire 30-person team.
Saved us $600/month and our security team finally approved it."
— CTO, Series A Startup

"I set it up on a $5 DigitalOcean droplet and it just works.
Unlimited tunnels, my own domain, zero hassle."
— Freelance Developer

"We needed HIPAA compliance. ngrok couldn't do it without
enterprise pricing. SubTunnel on our own VPC solved it in a day."
— Lead Engineer, HealthTech

⭐ [X,XXX] GitHub stars  |  📦 [XX,XXX] Docker pulls  |  👥 [X,XXX] Discord members
```

### Final CTA
```
## Deploy SubTunnel in 5 minutes.

No sign-up required. No credit card. No limits.
Just a Docker command and you're live.

$ docker run -d -p 443:443 subtunnel/server

[Read the docs →]  [⭐ Star on GitHub]
```

---

## 8. SEO Strategy

### Primary Keywords (target with dedicated pages)
| Keyword | Monthly Volume (est.) | Page |
|---------|----------------------|------|
| ngrok alternative | 8,000 | `/compare/ngrok` |
| self-hosted ngrok | 3,000 | Homepage + `/compare/ngrok` |
| ngrok self-hosted | 2,500 | `/compare/ngrok` |
| expose localhost to internet | 5,000 | Blog post |
| localhost tunnel | 4,000 | Homepage |
| ngrok open source alternative | 1,500 | `/open-source` |
| cloudflare tunnel alternative | 2,000 | `/compare/cloudflare-tunnel` |
| frp alternative | 1,000 | `/compare/frp` |
| webhook testing localhost | 3,000 | `/use-cases/webhooks` |
| share localhost | 2,500 | Blog post |

### Content Strategy
1. **Comparison pages** — `/compare/ngrok`, `/compare/cloudflare-tunnel`, `/compare/frp` — these capture high-intent "alternative" searches
2. **Tutorial blog posts** — "How to expose localhost", "Webhook development guide", "TCP tunnel setup" — capture top-of-funnel
3. **Migration guides** — "Migrate from ngrok to SubTunnel" — capture switching intent
4. **Use case pages** — `/use-cases/webhooks`, `/use-cases/demos`, `/use-cases/iot` — capture specific need searches
5. **Documentation SEO** — Ensure docs are indexed, good structure, answer specific "how to" queries

### Technical SEO
- Static site (Next.js or Astro) with SSG for all marketing pages
- Proper meta titles: "SubTunnel vs ngrok — Self-Hosted Tunnel Alternative (2026)"
- Schema markup for software application
- `og:image` cards for every page (auto-generated)
- Sitemap + robots.txt
- Fast loading (< 1s LCP target)
- GitHub README links back to website

### Link Building
- Submit to awesome-tunneling list (GitHub)
- Hacker News launch post
- Reddit posts in r/selfhosted, r/devops, r/webdev
- Dev.to / Hashnode articles
- Product Hunt launch
- Docker Hub listing

---

## 9. Launch Roadmap

### MVP (Month 1-2) — "It works"
**Goal:** Basic tunnel server + CLI that people can self-host

- [ ] Server binary (Go) with HTTP/HTTPS tunnel support
- [ ] CLI client: `subtunnel http <port>`
- [ ] Automatic TLS via Let's Encrypt
- [ ] Custom domain support (CNAME)
- [ ] Random subdomain generation
- [ ] Basic authentication (API key)
- [ ] Docker image (`subtunnel/server`)
- [ ] Minimal web dashboard (active tunnels list, basic request log)
- [ ] README + quick start docs
- [ ] GitHub repo (MIT license)

**Launch:** GitHub + Hacker News + r/selfhosted

### v1.0 (Month 3-4) — "Teams use it"
**Goal:** Production-ready for small teams

- [ ] TCP tunnel support
- [ ] WebSocket tunnel support
- [ ] Web traffic inspector with replay
- [ ] Team management (invite, roles)
- [ ] Multiple API keys with scopes
- [ ] Usage analytics dashboard
- [ ] Configuration file support (YAML)
- [ ] Helm chart for Kubernetes
- [ ] Marketing website (all pages from Section 4)
- [ ] Documentation site
- [ ] CLI auto-update

**Launch:** Product Hunt + blog post + comparison pages live

### v1.5 (Month 5-6) — "Enterprises evaluate it"
**Goal:** Enterprise features that unlock Pro tier revenue

- [ ] SSO/SAML integration
- [ ] Audit logging
- [ ] RBAC (per-tunnel, per-domain permissions)
- [ ] Tunnel templates / saved configurations
- [ ] Alerting (tunnel down → Slack/email/webhook)
- [ ] Request/response modification (header injection, rewriting)
- [ ] IP allowlisting
- [ ] Agent SDK (Go, Python, Node.js)
- [ ] Pro license + Stripe billing integration

### v2.0 (Month 7-10) — "Platform"
**Goal:** Full platform with managed cloud offering

- [ ] Managed cloud offering (SubTunnel Cloud)
- [ ] Global edge network (multi-region relay)
- [ ] Rate limiting
- [ ] Basic WAF rules
- [ ] Organization management (multi-team)
- [ ] Agent management (remote agent status, versioning)
- [ ] gRPC tunnel support
- [ ] Plugin system for traffic policies
- [ ] Mobile-friendly dashboard
- [ ] `subtunnel.dev` managed service live

### v2.5+ (Month 10+) — "Ecosystem"
- [ ] Kubernetes Operator (CRD-based tunnel management)
- [ ] GitHub Actions integration
- [ ] VS Code extension
- [ ] Terraform provider
- [ ] Mutual TLS support
- [ ] Load balancing across multiple upstreams
- [ ] Marketplace for traffic policy plugins

---

## Appendix: Tech Stack Recommendation

| Component | Technology | Why |
|-----------|-----------|-----|
| Server | Go | Performance, single binary, great for networking |
| CLI | Go (Cobra) | Same language as server, cross-compile easily |
| Dashboard | Next.js (React) | SSR for SEO pages, SPA for dashboard |
| Database | SQLite (MVP) → PostgreSQL (v1+) | Zero-dep start, scale later |
| TLS | Let's Encrypt (ACME) | Free, automatic |
| Tunnel protocol | QUIC or custom over WebSocket | Performant, firewall-friendly |
| Auth | JWT + API keys | Standard, stateless |
| Container | Docker + Helm | Standard deployment |
| Marketing site | Astro or Next.js | Static generation, fast |
| Docs | Docusaurus or Mintlify | Developer-friendly, searchable |

---

*Last updated: February 12, 2026*
*Author: SubTunnel Product Team*
