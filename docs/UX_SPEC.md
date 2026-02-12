# SubTunnel — UX Specification

> Self-hosted ngrok alternative. Two web apps: marketing site (`apps/web`) and dashboard (`apps/dashboard`).

---

## Table of Contents

1. [Design System](#design-system)
2. [Marketing Website](#marketing-website-appsweb)
3. [Dashboard](#dashboard-appsdashboard)

---

## Design System

### Color Palette

| Token | Light Mode | Dark Mode | Usage |
|-------|-----------|-----------|-------|
| `--primary` | `#6C3FE0` | `#8B6CEF` | Buttons, links, active states |
| `--primary-hover` | `#5A2FCC` | `#A08AF5` | Hover states |
| `--secondary` | `#1A1A2E` | `#E8E8F0` | Headings, body text |
| `--accent` | `#00D4AA` | `#00EDBE` | Success, CTAs, terminal cursor |
| `--surface-0` | `#FFFFFF` | `#0D0D14` | Page background |
| `--surface-1` | `#F6F6FA` | `#16162A` | Cards, sidebars |
| `--surface-2` | `#EDEDF5` | `#1E1E3A` | Inputs, code blocks |
| `--border` | `#E0E0EA` | `#2A2A4A` | Dividers, card borders |
| `--danger` | `#E5484D` | `#FF6369` | Errors, destructive actions |
| `--warning` | `#F5A623` | `#FFB84D` | Warnings |
| `--info` | `#3B82F6` | `#60A5FA` | Informational |
| `--success` | `#00D4AA` | `#00EDBE` | Same as accent |
| `--text-muted` | `#6B7280` | `#9CA3AF` | Secondary text |

**Gradient (hero/CTAs):** `linear-gradient(135deg, #6C3FE0 0%, #00D4AA 100%)`

### Typography

| Token | Family | Weight | Size | Line Height |
|-------|--------|--------|------|-------------|
| `display` | Inter | 700 | 56px / 3.5rem | 1.1 |
| `h1` | Inter | 700 | 40px / 2.5rem | 1.2 |
| `h2` | Inter | 600 | 32px / 2rem | 1.25 |
| `h3` | Inter | 600 | 24px / 1.5rem | 1.3 |
| `body` | Inter | 400 | 16px / 1rem | 1.6 |
| `body-sm` | Inter | 400 | 14px / 0.875rem | 1.5 |
| `caption` | Inter | 500 | 12px / 0.75rem | 1.4 |
| `code` | JetBrains Mono | 400 | 14px / 0.875rem | 1.6 |

**Scale:** 4px base unit. Spacing: 4, 8, 12, 16, 24, 32, 48, 64, 96, 128.

### Component Inventory

#### Buttons

| Variant | Background | Text | Border | Border Radius |
|---------|-----------|------|--------|---------------|
| Primary | `--primary` | `#FFFFFF` | none | 8px |
| Secondary | transparent | `--primary` | 1px `--primary` | 8px |
| Ghost | transparent | `--secondary` | none | 8px |
| Danger | `--danger` | `#FFFFFF` | none | 8px |
| Sizes | sm: 32px h, md: 40px h, lg: 48px h | | | |

All buttons: `font-weight: 500`, `padding: 0 16px` (md), `transition: all 150ms ease`.

#### Cards

```
.card {
  background: var(--surface-1);
  border: 1px solid var(--border);
  border-radius: 12px;
  padding: 24px;
  transition: border-color 200ms ease, box-shadow 200ms ease;
}
.card:hover {
  border-color: var(--primary);
  box-shadow: 0 4px 24px rgba(108, 63, 224, 0.08);
}
```

#### Inputs

```
.input {
  height: 40px;
  padding: 0 12px;
  background: var(--surface-2);
  border: 1px solid var(--border);
  border-radius: 8px;
  font-size: 14px;
  transition: border-color 150ms ease;
}
.input:focus {
  border-color: var(--primary);
  outline: none;
  box-shadow: 0 0 0 3px rgba(108, 63, 224, 0.15);
}
```

#### Tables

- Header row: `background: var(--surface-1)`, `font-weight: 600`, `font-size: 12px`, uppercase, `letter-spacing: 0.05em`
- Rows: `border-bottom: 1px solid var(--border)`, `height: 48px`
- Hover: `background: var(--surface-1)`

#### Modals

- Overlay: `background: rgba(0,0,0,0.5)`, `backdrop-filter: blur(4px)`
- Panel: `max-width: 480px`, `border-radius: 16px`, `padding: 32px`
- Close button: top-right, `24×24` icon button

#### Toasts

- Position: bottom-right, stacked with 8px gap
- Auto-dismiss: 5s (info), persistent (error)
- Variants: success (green left border), error (red), info (blue), warning (yellow)
- `border-radius: 8px`, `padding: 12px 16px`, slide-in from right

#### Status Badge

```
.badge { padding: 2px 8px; border-radius: 9999px; font-size: 12px; font-weight: 500; }
.badge-online { background: #00D4AA20; color: #00D4AA; }
.badge-offline { background: #6B728020; color: #6B7280; }
.badge-error { background: #E5484D20; color: #E5484D; }
```

### Dark / Light Mode

- Default: **dark** (developer tool convention, matches terminal aesthetic)
- Toggle: sun/moon icon in header — persisted to `localStorage` and `prefers-color-scheme` respected on first visit
- Implementation: CSS custom properties on `:root` swapped via `[data-theme="light"]` attribute on `<html>`
- All colors defined as pairs in the palette table above

### Motion Principles

| Pattern | Duration | Easing | Usage |
|---------|----------|--------|-------|
| Micro | 150ms | `ease-out` | Button hover, focus rings |
| Standard | 200ms | `ease-in-out` | Card hover, dropdown open |
| Page | 300ms | `cubic-bezier(0.4, 0, 0.2, 1)` | Page transitions, modals |
| Emphasis | 500ms | `spring(1, 80, 10)` | Hero animations, number counters |

- **Reduce motion:** Respect `prefers-reduced-motion` — collapse all animations to instant
- **Scroll animations:** Use `IntersectionObserver` — fade-up 20px with 200ms stagger between siblings
- **Terminal typing:** Monospaced text typed char-by-char at 40ms/char with blinking cursor

---

## Marketing Website (`apps/web`)

### Page Map

| Path | Page | Purpose |
|------|------|---------|
| `/` | Home | Hero + features + social proof + CTA |
| `/features` | Features | Detailed feature breakdown with diagrams |
| `/pricing` | Pricing | Self-hosted vs managed tiers |
| `/docs` | Documentation | Guides, API reference, configuration |
| `/docs/[...slug]` | Doc page | Individual doc content |
| `/blog` | Blog index | Updates, tutorials, case studies |
| `/blog/[slug]` | Blog post | Individual blog post |
| `/about` | About | Mission, team, open source philosophy |
| `/login` | Login redirect | Redirects to dashboard `/login` |
| `/github` | GitHub redirect | → GitHub repo |

### Navigation

#### Header (sticky, 64px height)

```
┌──────────────────────────────────────────────────────────────────┐
│  [Logo]  Features  Pricing  Docs  Blog  ·····  [GitHub ★]  [Dashboard →] │
└──────────────────────────────────────────────────────────────────┘
```

- `max-width: 1200px`, centered, `padding: 0 24px`
- Logo: "SubTunnel" wordmark + icon (tunnel/portal glyph), links to `/`
- Nav links: `font-size: 14px`, `font-weight: 500`, `color: var(--text-muted)`, hover → `var(--secondary)`
- Right side: GitHub star badge (pulls live count), primary "Dashboard →" button
- **Scroll behavior:** transparent background at top, gains `background: var(--surface-0)/90%` + `backdrop-filter: blur(8px)` after 48px scroll
- **Mobile (< 768px):** Hamburger menu → full-screen overlay with stacked links, 300ms slide-down

#### Footer

```
┌─────────────────────────────────────────────────────┐
│  [Logo]                                              │
│                                                      │
│  Product        Resources       Community            │
│  Features       Docs            GitHub               │
│  Pricing        Blog            Discord              │
│  Self-Hosting   Changelog       Twitter              │
│                                                      │
│  ─────────────────────────────────────────────────── │
│  © 2026 SubTunnel · MIT License · Privacy            │
└─────────────────────────────────────────────────────┘
```

- 3-column grid on desktop, stacked on mobile
- `background: var(--surface-1)`, `padding: 64px 24px 32px`

### Home Page — Hero Section

```
┌──────────────────────────────────────────────────────────────────┐
│                                                                  │
│              Self-hosted tunnels.                                 │
│              Your server. Your rules.                            │
│                                                                  │
│     Expose local services to the internet with a single          │
│     command. Open source, self-hosted, no vendor lock-in.        │
│                                                                  │
│     [Get Started]  [View on GitHub]                              │
│                                                                  │
│     ┌──────────────────────────────────────────────┐            │
│     │ $ subtunnel http 3000                        │            │
│     │                                              │            │
│     │ ✔ Tunnel established                         │            │
│     │ ✔ https://myapp.tunnel.example.com → :3000   │            │
│     │                                              │            │
│     │ Connections: 12  │  Transfer: 4.2 MB         │            │
│     │ Status: online   │  Uptime: 2h 34m           │            │
│     └──────────────────────────────────────────────┘            │
│                                                                  │
└──────────────────────────────────────────────────────────────────┘
```

- **Layout:** Centered text, `max-width: 720px` for copy
- **Headline:** `display` size, gradient text (`background-clip: text`) using primary→accent gradient
- **Subhead:** `body` size, `--text-muted`, `max-width: 540px`
- **Buttons:** Primary "Get Started" + Secondary "View on GitHub" (with GitHub icon)
- **Terminal demo:** Fake terminal window with macOS-style dots (red/yellow/green), dark background (`#0D0D14`), monospace font. Text types in sequentially using the typing animation. After typing completes, the stats line updates live (incrementing connection count every 2s).
- **Background:** Subtle gradient mesh or grid pattern with animated dots/particles (low opacity, `--primary` color)

### Home Page — Features Section

Below the hero, a 3-card row highlights core features:

```
┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐
│  🔒              │  │  🌍              │  │  🛠              │
│  End-to-End      │  │  Custom          │  │  Self-Hosted     │
│  Encryption      │  │  Domains         │  │  Freedom         │
│                  │  │                  │  │                  │
│  All traffic     │  │  Use your own    │  │  Run on your     │
│  encrypted with  │  │  domains with    │  │  own infra.      │
│  TLS. Zero       │  │  automatic       │  │  No data leaves  │
│  trust by        │  │  TLS certs via   │  │  your network    │
│  default.        │  │  Let's Encrypt.  │  │  unless you      │
│                  │  │                  │  │  want it to.     │
└─────────────────┘  └─────────────────┘  └─────────────────┘
```

- `display: grid; grid-template-columns: repeat(3, 1fr); gap: 24px;` (stacks to 1-col on mobile)
- Each card uses the `.card` component with a top icon (32px emoji or custom SVG), `h3` title, `body-sm` description
- **Scroll animation:** Cards fade up with 100ms stagger on intersection

**Expanded features (below):** Alternating left-right sections (text + illustration):

1. **HTTP & TCP tunnels** — diagram showing request flow
2. **Dashboard & analytics** — screenshot/mockup of dashboard
3. **API & CLI** — code snippet showing API usage
4. **Team management** — icons showing multi-user access

Each section: `max-width: 1200px`, 2-column grid `1fr 1fr`, `gap: 64px`, vertically centered, `padding: 96px 24px`. Alternates image left/right.

### Home Page — Social Proof / Stats

```
┌──────────────────────────────────────────────────────┐
│       ⭐ 2.4k         📦 50k+          🌍 12          │
│    GitHub Stars    Downloads      Server Regions      │
└──────────────────────────────────────────────────────┘
```

- 3-column centered row, numbers use `h1` size with counter animation on scroll-in
- Below: optional testimonial carousel or GitHub contributor avatars

### Home Page — Final CTA

```
┌──────────────────────────────────────────────────────┐
│                                                      │
│   Ready to tunnel?                                   │
│   Get started in under 2 minutes.                    │
│                                                      │
│   [Install SubTunnel →]                              │
│                                                      │
└──────────────────────────────────────────────────────┘
```

- `background: var(--surface-1)`, `border-radius: 16px`, `padding: 64px`, centered text
- Single primary button, large variant

### Pricing Page

SubTunnel is open-source/self-hosted, so pricing is different from ngrok. Two models:

```
┌─────────────────────────────────────────────────────────────────┐
│                     Simple, honest pricing.                      │
│          Self-host for free, or let us run it for you.          │
│                                                                  │
│  ┌─────────────────────┐    ┌─────────────────────┐            │
│  │  Self-Hosted         │    │  SubTunnel Cloud     │            │
│  │  ──────────────────  │    │  ──────────────────   │            │
│  │  Free forever        │    │  $9/mo                │            │
│  │                      │    │                       │            │
│  │  ✓ Unlimited tunnels │    │  ✓ Everything in Free │            │
│  │  ✓ Unlimited users   │    │  ✓ Managed server     │            │
│  │  ✓ Custom domains    │    │  ✓ Global edge nodes  │            │
│  │  ✓ Full API access   │    │  ✓ Custom domains     │            │
│  │  ✓ MIT licensed      │    │  ✓ 99.9% SLA          │            │
│  │                      │    │  ✓ Priority support   │            │
│  │  [View on GitHub]    │    │  [Start Free Trial]   │            │
│  └─────────────────────┘    └─────────────────────┘            │
│                                                                  │
│  ┌─────────────────────────────────────────────────┐            │
│  │  Feature Comparison Table                        │            │
│  │  ───────────────────────────────────────────────  │            │
│  │  Feature          │ Self-Hosted │ Cloud │ ngrok  │            │
│  │  HTTP tunnels     │ ∞           │ ∞     │ 1 free │            │
│  │  TCP tunnels      │ ∞           │ ∞     │ paid   │            │
│  │  Custom domains   │ ✓           │ ✓     │ paid   │            │
│  │  ...              │             │       │        │            │
│  └─────────────────────────────────────────────────┘            │
│                                                                  │
│  FAQ (accordion)                                                 │
│  ▸ Can I migrate from ngrok?                                    │
│  ▸ What infrastructure do I need to self-host?                  │
│  ▸ Is there a free trial for Cloud?                             │
│  ▸ Do you offer team/enterprise plans?                          │
└─────────────────────────────────────────────────────────────────┘
```

- **Layout:** 2-column card grid for tiers, centered, `max-width: 960px`
- Cloud card gets a subtle `--primary` border and "Popular" badge
- Comparison table below: full-width, includes ngrok column for competitive comparison
- FAQ: accordion with `+`/`-` toggle, smooth height animation

### Docs Page

```
┌────────────────────────────────────────────────────────────┐
│  [Header]                                                   │
├──────────┬─────────────────────────────────┬───────────────┤
│ Sidebar  │  Content                        │  On this page │
│ 240px    │  max-width: 720px               │  200px        │
│          │                                  │               │
│ Getting  │  # Installation                 │  • Install    │
│  Started │                                  │  • Config     │
│  Install │  ```bash                         │  • First      │
│  Config  │  curl -fsSL ... | sh             │    tunnel     │
│  Quick   │  subtunnel server start          │               │
│  Start   │  ```                             │               │
│          │                                  │               │
│ Guides   │  ## Configuration                │               │
│  HTTP    │                                  │               │
│  TCP     │  Create `subtunnel.yml`:         │               │
│  TLS     │                                  │               │
│          │  ```yaml                         │               │
│ API Ref  │  server:                         │               │
│  REST    │    domain: tunnel.example.com    │               │
│  CLI     │    port: 443                     │               │
│          │  ```                             │               │
│ Self-    │                                  │               │
│  Host    │                                  │               │
│  Docker  │                                  │               │
│  K8s     │                                  │               │
│  Binary  │                                  │               │
├──────────┴─────────────────────────────────┴───────────────┤
│  ← Previous: Overview    Next: Configuration →             │
└────────────────────────────────────────────────────────────┘
```

- **3-column layout:** left sidebar (240px, collapsible on mobile) + content (flex-grow, `max-width: 720px`) + right TOC (200px, sticky, hidden < 1280px)
- Sidebar: collapsible sections with arrow toggles, active item highlighted with `--primary` left border (3px)
- Code blocks: syntax highlighted (Shiki), with copy button top-right, language badge top-left
- **Mobile:** sidebar becomes a dropdown/drawer triggered by hamburger, TOC hidden
- **Search:** top of sidebar, opens modal with `Cmd+K` shortcut, fuzzy search across all docs
- Previous/Next navigation at bottom of each page

---

## Dashboard (`apps/dashboard`)

### Auth Flow

#### Login (`/login`)

```
┌──────────────────────────────────────────────────────┐
│                                                      │
│              [SubTunnel Logo]                         │
│                                                      │
│          Welcome back                                │
│                                                      │
│   [Continue with GitHub]                             │
│   [Continue with Google]                             │
│                                                      │
│   ──── or ────                                       │
│                                                      │
│   Email         [________________]                   │
│   Password      [________________]                   │
│                                                      │
│   [Log in]                          Forgot password? │
│                                                      │
│   Don't have an account? Sign up →                   │
│                                                      │
└──────────────────────────────────────────────────────┘
```

- **Layout:** Centered card, `max-width: 400px`, vertically centered in viewport
- **Background:** `var(--surface-0)` with subtle grid/dot pattern
- **OAuth buttons:** Full-width, outline style with provider icon (GitHub/Google), 48px height
- **Divider:** Horizontal rule with "or" text centered

#### Signup (`/signup`)

Same layout as login but fields: Name, Email, Password. OAuth on top. Link to login at bottom.

#### Onboarding (`/onboarding`) — shown after first signup

```
Step 1/3: Install the CLI
─────────────────────────
  curl -fsSL https://get.subtunnel.dev | sh

Step 2/3: Authenticate
─────────────────────────
  subtunnel auth <token>

Step 3/3: Create your first tunnel
─────────────────────────
  subtunnel http 3000

  [Complete Setup →]
```

- **Stepper component** at top: 3 circles connected by line, active step filled with `--primary`
- Each step: title + code block with copy button
- Auth token pre-filled and auto-copied
- Can be skipped ("Skip for now" link)

### Main Layout

```
┌─────────────────────────────────────────────────────────────┐
│  [≡] SubTunnel          search (⌘K)         [avatar ▼]      │
├──────────┬──────────────────────────────────────────────────┤
│          │                                                   │
│  Tunnels │   Page Title                                      │
│  Domains │   Description text                                │
│  API Keys│                                                   │
│  Usage   │   ┌─────────────────────────────────────────┐    │
│  ───     │   │  Content area                            │    │
│  Settings│   │  max-width: 1080px                       │    │
│  Docs ↗  │   │                                          │    │
│          │   └─────────────────────────────────────────┘    │
│          │                                                   │
│          │                                                   │
│  ───     │                                                   │
│  v0.1.0  │                                                   │
└──────────┴──────────────────────────────────────────────────┘
```

- **Sidebar:** 240px wide, `background: var(--surface-1)`, full height
  - Logo at top (links to `/tunnels`)
  - Nav items: icon (20px) + label, `padding: 8px 16px`, `border-radius: 6px`
  - Active: `background: var(--primary)/10%`, `color: var(--primary)`
  - Hover: `background: var(--surface-2)`
  - Divider line between main nav and utility links
  - Version number at bottom, muted
  - **Mobile (< 1024px):** Sidebar collapses to icons only (64px) or hidden with hamburger toggle

- **Header:** 56px height, `border-bottom: 1px solid var(--border)`
  - Left: hamburger (mobile), breadcrumb
  - Center: search bar (opens `Cmd+K` modal)
  - Right: avatar dropdown (Profile, Settings, Logout)

- **Content area:** `padding: 32px`, `max-width: 1080px`

### Tunnel List (`/tunnels`)

```
┌─────────────────────────────────────────────────────────────┐
│  Active Tunnels                           [+ New Tunnel]     │
│                                                              │
│  ┌────────────────────────────────────────────────────────┐ │
│  │ ● myapp          https://myapp.tunnel.dev → :3000      │ │
│  │   HTTP · 2h 34m · 1.2k requests · 4.2 MB              │ │
│  ├────────────────────────────────────────────────────────┤ │
│  │ ● api-server     https://api.tunnel.dev → :8080        │ │
│  │   HTTP · 45m · 342 requests · 890 KB                   │ │
│  ├────────────────────────────────────────────────────────┤ │
│  │ ○ db-tunnel      tcp://tunnel.dev:54321 → :5432        │ │
│  │   TCP · offline · last seen 3h ago                     │ │
│  └────────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────┘
```

- **Layout:** List/table of tunnel cards
- Each row: status dot (green=online `#00D4AA`, gray=offline, red=error), tunnel name (bold), public URL → local port
- Second line: protocol badge, uptime, request count, transfer
- Click row → tunnel detail page (logs, metrics, config)
- **"+ New Tunnel" button:** Opens modal with CLI instructions (tunnels are created via CLI, not UI)
- **Filters:** dropdown for status (all/online/offline), search by name
- **Real-time:** Status dots pulse gently for online tunnels, WebSocket updates for new connections

#### Tunnel Detail (`/tunnels/:id`)

- **Header:** Tunnel name, status badge, public URL (clickable), delete button (danger)
- **Tabs:** Overview | Requests | Configuration
  - **Overview:** Uptime chart (sparkline), request/s graph, transfer stats
  - **Requests:** Live-updating table of recent HTTP requests (method, path, status, duration, time) — similar to ngrok's request inspector
  - **Configuration:** Read-only YAML display of tunnel config

### API Keys (`/keys`)

```
┌──────────────────────────────────────────────────────────────┐
│  API Keys                                  [+ Create Key]     │
│                                                               │
│  ┌─────────────────────────────────────────────────────────┐ │
│  │  Name          Key                  Created     Actions  │ │
│  │  ─────────────────────────────────────────────────────── │ │
│  │  production    sk_live_••••••3f2a   Jan 3      [Revoke] │ │
│  │  development   sk_test_••••••8b1c   Jan 15     [Revoke] │ │
│  └─────────────────────────────────────────────────────────┘ │
└──────────────────────────────────────────────────────────────┘
```

- Table layout with masked keys (show last 4 chars)
- **Create modal:** Name input + optional expiry dropdown → shows full key ONCE with copy button + warning: "This won't be shown again"
- **Revoke:** Confirmation modal with key name, danger button "Revoke Key"
- Copy button on each row (copies masked key? No — shows toast "Key hidden for security. Use the key you saved at creation.")

### Usage / Analytics (`/usage`)

```
┌──────────────────────────────────────────────────────────────┐
│  Usage & Analytics              [24h ▼] [7d] [30d] [custom]  │
│                                                               │
│  ┌───────────┐ ┌───────────┐ ┌───────────┐ ┌───────────┐   │
│  │ Requests  │ │ Transfer  │ │ Tunnels   │ │ Uptime    │   │
│  │ 12.4k     │ │ 2.1 GB    │ │ 3 active  │ │ 99.8%     │   │
│  │ ↑ 12%     │ │ ↑ 8%      │ │ ── 2      │ │ ↑ 0.1%    │   │
│  └───────────┘ └───────────┘ └───────────┘ └───────────┘   │
│                                                               │
│  ┌─────────────────────────────────────────────────────────┐ │
│  │  [Area chart: requests over time]                        │ │
│  │  ████████                                                │ │
│  │  ██████████████                                          │ │
│  │  ████████████████████                                    │ │
│  └─────────────────────────────────────────────────────────┘ │
│                                                               │
│  ┌─────────────────────────┐ ┌─────────────────────────────┐│
│  │  Top tunnels by traffic │ │  Response codes breakdown   ││
│  │  (bar chart)            │ │  (donut chart)              ││
│  └─────────────────────────┘ └─────────────────────────────┘│
└──────────────────────────────────────────────────────────────┘
```

- **Time range selector:** Pill buttons (24h, 7d, 30d) + custom date picker
- **Stat cards:** 4-column grid, each with metric name, big number, delta with arrow + % (green=up, red=down for errors)
- **Main chart:** Area chart with gradient fill (`--primary` at 20% opacity), `height: 300px`
- **Secondary charts:** 2-column grid below — bar chart (top tunnels) + donut (status codes)
- **Charting library:** Recharts or Chart.js with custom theme matching design system

### Settings (`/settings`)

Tabbed interface: **Profile** | **Server** | **Team** | **Billing** (cloud only)

#### Profile
- Avatar upload (or Gravatar auto-fetch)
- Name, email (read-only if OAuth)
- Change password
- Two-factor authentication toggle

#### Server (self-hosted specific)
- Server URL, port
- TLS configuration status
- Domain configuration
- Version info + update available banner

#### Team
- Member list table: avatar, name, email, role (Admin/Member), joined date
- Invite button → modal with email input + role selector
- Remove member: confirmation modal

#### Billing (cloud only)
- Current plan card
- Usage this month
- Payment method
- Invoice history table

### Empty States

Every list page needs a thoughtful empty state:

#### No Tunnels

```
┌─────────────────────────────────────────┐
│                                          │
│          [tunnel illustration]            │
│                                          │
│     No tunnels yet                       │
│     Create your first tunnel from        │
│     the command line:                    │
│                                          │
│     $ subtunnel http 3000               │
│                                          │
│     [Read the docs →]                    │
│                                          │
└─────────────────────────────────────────┘
```

- Centered, `max-width: 400px`
- Illustration: minimal line art of a tunnel/portal (64px, `--text-muted` color)
- Code block with copy button
- Link to docs

#### No API Keys
- "No API keys yet. Create one to authenticate your CLI."
- [Create API Key] button

#### No Usage Data
- "No data yet. Usage stats will appear once your tunnels start receiving traffic."
- Muted chart placeholder (dotted line)

---

## Responsive Breakpoints

| Token | Width | Behavior |
|-------|-------|----------|
| `sm` | 640px | Single column, stacked nav |
| `md` | 768px | 2-col grids, sidebar drawer |
| `lg` | 1024px | Full sidebar visible |
| `xl` | 1280px | Right TOC visible (docs) |

---

## Tech Stack Recommendations (for consistency with UX)

- **Framework:** Next.js (marketing), React + Vite (dashboard) — or both in Turborepo monorepo
- **Styling:** Tailwind CSS + CSS custom properties for theming
- **Components:** Radix UI primitives (accessible, unstyled) + custom styling
- **Charts:** Recharts
- **Icons:** Lucide
- **Fonts:** Inter (Google Fonts), JetBrains Mono (Google Fonts)
- **Animations:** Framer Motion
- **Docs:** MDX with next-mdx-remote or Fumadocs
