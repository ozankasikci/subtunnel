"use client";

import Link from "next/link";
import {
  Globe,
  Server,
  Shield,
  LayoutDashboard,
  Users,
  Code2,
  ArrowRight,
  Check,
  X,
  Terminal,
} from "lucide-react";

function TerminalDemo() {
  return (
    <div className="relative glow-accent rounded-xl overflow-hidden">
      {/* Glow effect behind terminal */}
      <div className="absolute -inset-1 bg-gradient-to-r from-accent/20 via-accent-2/20 to-accent/20 rounded-xl blur-xl opacity-50" />
      <div className="relative bg-surface border border-border rounded-xl overflow-hidden">
        {/* Title bar */}
        <div className="flex items-center gap-2 px-4 py-3 bg-surface-2 border-b border-border">
          <div className="flex gap-1.5">
            <div className="w-3 h-3 rounded-full bg-[#ff5f57]" />
            <div className="w-3 h-3 rounded-full bg-[#febc2e]" />
            <div className="w-3 h-3 rounded-full bg-[#28c840]" />
          </div>
          <span className="text-xs text-muted font-mono ml-2">Terminal</span>
        </div>
        {/* Content */}
        <div className="p-6 font-mono text-sm leading-relaxed">
          <div className="flex items-center gap-2">
            <span className="text-accent">$</span>
            <span className="text-foreground">subtunnel http 3000</span>
          </div>
          <div className="mt-4 text-muted text-xs">
            <p>SubTunnel v1.0.0</p>
            <p className="mt-1">
              Status: <span className="text-[#28c840]">online</span>
            </p>
            <p>
              Forwarding:{" "}
              <span className="text-accent">
                https://my-app.subtunnel.dev
              </span>{" "}
              → localhost:3000
            </p>
            <p className="mt-1">
              Dashboard:{" "}
              <span className="text-muted">
                http://localhost:4040
              </span>
            </p>
          </div>
          <div className="mt-4 flex items-center gap-2">
            <span className="text-accent">$</span>
            <span className="text-muted">
              <span className="inline-block w-2 h-4 bg-accent/70 animate-pulse" />
            </span>
          </div>
        </div>
      </div>
    </div>
  );
}

const features = [
  {
    icon: Server,
    title: "Self-Hosted",
    description:
      "Run on your own infrastructure. Full control over your data and traffic. No third-party dependencies.",
  },
  {
    icon: Globe,
    title: "Custom Domains",
    description:
      "Use your own domain names for tunnels. Professional URLs for demos, webhooks, and development.",
  },
  {
    icon: Shield,
    title: "End-to-End Encryption",
    description:
      "All traffic encrypted in transit. TLS termination at the edge with automatic certificate management.",
  },
  {
    icon: LayoutDashboard,
    title: "Real-time Dashboard",
    description:
      "Monitor active tunnels, inspect requests, and replay traffic. Built-in request inspector.",
  },
  {
    icon: Users,
    title: "Team Support",
    description:
      "Multi-user support with role-based access control. Share tunnels across your team securely.",
  },
  {
    icon: Code2,
    title: "Open Source",
    description:
      "Fully open source under MIT. Contribute, fork, extend. No vendor lock-in, ever.",
  },
];

const comparison = [
  { feature: "Self-hosted option", subtunnel: true, ngrok: false },
  { feature: "Custom domains", subtunnel: true, ngrok: true },
  { feature: "Open source", subtunnel: true, ngrok: false },
  { feature: "Free tier", subtunnel: "Unlimited", ngrok: "Limited" },
  { feature: "Request inspection", subtunnel: true, ngrok: true },
  { feature: "No vendor lock-in", subtunnel: true, ngrok: false },
  { feature: "Team management", subtunnel: true, ngrok: true },
  { feature: "Data stays on your servers", subtunnel: true, ngrok: false },
];

export default function HomePage() {
  return (
    <div>
      {/* Hero */}
      <section className="relative overflow-hidden">
        {/* Background gradient */}
        <div className="absolute inset-0 bg-[radial-gradient(ellipse_at_top,rgba(0,212,255,0.08),transparent_60%)]" />
        <div className="relative mx-auto max-w-6xl px-6 pt-24 pb-20 md:pt-32 md:pb-28">
          <div className="text-center max-w-3xl mx-auto">
            <div className="inline-flex items-center gap-2 rounded-full border border-border bg-surface px-4 py-1.5 text-xs text-muted mb-8 animate-fade-in-up">
              <span className="h-1.5 w-1.5 rounded-full bg-accent animate-pulse" />
              Now in public beta — self-host in minutes
            </div>
            <h1 className="text-4xl sm:text-5xl md:text-6xl font-bold tracking-tight leading-[1.1] animate-fade-in-up">
              Expose localhost to
              <br />
              the internet.{" "}
              <span className="bg-gradient-to-r from-accent to-cyan-300 bg-clip-text text-transparent">
                Instantly.
              </span>
            </h1>
            <p className="mt-6 text-lg text-muted max-w-xl mx-auto leading-relaxed animate-fade-in-up-delay-1">
              The self-hosted ngrok alternative. Secure tunnels, custom domains,
              and a real-time dashboard — all on your own infrastructure.
            </p>
            <div className="flex flex-col sm:flex-row items-center justify-center gap-4 mt-10 animate-fade-in-up-delay-2">
              <Link
                href="/docs"
                className="inline-flex h-11 items-center gap-2 rounded-lg bg-accent px-6 text-sm font-medium text-black hover:bg-accent/90 transition-colors"
              >
                Get Started Free
                <ArrowRight className="w-4 h-4" />
              </Link>
              <a
                href="https://github.com/subtunnel"
                className="inline-flex h-11 items-center gap-2 rounded-lg border border-border bg-surface px-6 text-sm font-medium text-foreground hover:bg-surface-2 transition-colors"
              >
                <svg
                  width="16"
                  height="16"
                  viewBox="0 0 24 24"
                  fill="currentColor"
                >
                  <path d="M12 0c-6.626 0-12 5.373-12 12 0 5.302 3.438 9.8 8.207 11.387.599.111.793-.261.793-.577v-2.234c-3.338.726-4.033-1.416-4.033-1.416-.546-1.387-1.333-1.756-1.333-1.756-1.089-.745.083-.729.083-.729 1.205.084 1.839 1.237 1.839 1.237 1.07 1.834 2.807 1.304 3.492.997.107-.775.418-1.305.762-1.604-2.665-.305-5.467-1.334-5.467-5.931 0-1.311.469-2.381 1.236-3.221-.124-.303-.535-1.524.117-3.176 0 0 1.008-.322 3.301 1.23.957-.266 1.983-.399 3.003-.404 1.02.005 2.047.138 3.006.404 2.291-1.552 3.297-1.23 3.297-1.23.653 1.653.242 2.874.118 3.176.77.84 1.235 1.911 1.235 3.221 0 4.609-2.807 5.624-5.479 5.921.43.372.823 1.102.823 2.222v3.293c0 .319.192.694.801.576 4.765-1.589 8.199-6.086 8.199-11.386 0-6.627-5.373-12-12-12z" />
                </svg>
                View on GitHub
              </a>
            </div>
          </div>

          {/* Terminal */}
          <div className="mt-16 md:mt-20 max-w-2xl mx-auto animate-fade-in-up-delay-3">
            <TerminalDemo />
          </div>
        </div>
      </section>

      {/* Features */}
      <section id="features" className="relative py-24 md:py-32">
        <div className="mx-auto max-w-6xl px-6">
          <div className="text-center mb-16">
            <h2 className="text-3xl md:text-4xl font-bold tracking-tight">
              Everything you need to tunnel
            </h2>
            <p className="mt-4 text-muted max-w-lg mx-auto">
              Production-grade tunneling with the features your team actually needs.
            </p>
          </div>
          <div className="grid md:grid-cols-2 lg:grid-cols-3 gap-6">
            {features.map((f) => (
              <div
                key={f.title}
                className="group rounded-xl border border-border bg-surface/50 p-6 hover:border-accent/30 hover:bg-surface transition-all duration-300"
              >
                <div className="flex h-10 w-10 items-center justify-center rounded-lg bg-accent/10 border border-accent/20 mb-4 group-hover:bg-accent/20 transition-colors">
                  <f.icon className="w-5 h-5 text-accent" />
                </div>
                <h3 className="font-semibold mb-2">{f.title}</h3>
                <p className="text-sm text-muted leading-relaxed">
                  {f.description}
                </p>
              </div>
            ))}
          </div>
        </div>
      </section>

      {/* Code Example */}
      <section className="py-24 md:py-32 border-t border-border/50">
        <div className="mx-auto max-w-6xl px-6">
          <div className="grid lg:grid-cols-2 gap-12 lg:gap-16 items-center">
            <div>
              <h2 className="text-3xl md:text-4xl font-bold tracking-tight">
                One command.
                <br />
                <span className="text-accent">That&apos;s it.</span>
              </h2>
              <p className="mt-4 text-muted leading-relaxed max-w-md">
                No config files, no complex setup. Install the CLI, run one command, 
                and your local server is live on the internet.
              </p>
              <div className="mt-8 space-y-4">
                {[
                  "Automatic HTTPS with custom domains",
                  "Built-in request inspector",
                  "Replay failed webhooks instantly",
                  "Works behind firewalls and NATs",
                ].map((item) => (
                  <div key={item} className="flex items-center gap-3">
                    <div className="flex h-5 w-5 items-center justify-center rounded-full bg-accent/10">
                      <Check className="w-3 h-3 text-accent" />
                    </div>
                    <span className="text-sm text-muted">{item}</span>
                  </div>
                ))}
              </div>
            </div>
            <div className="bg-surface border border-border rounded-xl overflow-hidden">
              <div className="flex items-center gap-2 px-4 py-3 bg-surface-2 border-b border-border">
                <Terminal className="w-3.5 h-3.5 text-muted" />
                <span className="text-xs text-muted font-mono">Quick Start</span>
              </div>
              <div className="p-6 font-mono text-sm space-y-3">
                <div>
                  <span className="text-muted"># Install the CLI</span>
                </div>
                <div>
                  <span className="text-accent">$</span>{" "}
                  <span>curl -fsSL https://get.subtunnel.dev | sh</span>
                </div>
                <div className="pt-2">
                  <span className="text-muted"># Expose your local server</span>
                </div>
                <div>
                  <span className="text-accent">$</span>{" "}
                  <span>subtunnel http 3000</span>
                </div>
                <div className="pt-2">
                  <span className="text-muted"># Use a custom domain</span>
                </div>
                <div>
                  <span className="text-accent">$</span>{" "}
                  <span>
                    subtunnel http 3000 --domain{" "}
                    <span className="text-accent">api.example.com</span>
                  </span>
                </div>
                <div className="pt-2">
                  <span className="text-muted"># Expose a TCP service</span>
                </div>
                <div>
                  <span className="text-accent">$</span>{" "}
                  <span>subtunnel tcp 5432</span>
                </div>
              </div>
            </div>
          </div>
        </div>
      </section>

      {/* Comparison */}
      <section className="py-24 md:py-32 border-t border-border/50">
        <div className="mx-auto max-w-4xl px-6">
          <div className="text-center mb-16">
            <h2 className="text-3xl md:text-4xl font-bold tracking-tight">
              SubTunnel vs ngrok
            </h2>
            <p className="mt-4 text-muted">
              Self-hosted, open source, no vendor lock-in.
            </p>
          </div>
          <div className="border border-border rounded-xl overflow-hidden">
            <div className="grid grid-cols-3 bg-surface-2 border-b border-border">
              <div className="px-6 py-4 text-sm font-medium text-muted">
                Feature
              </div>
              <div className="px-6 py-4 text-sm font-medium text-center text-accent">
                SubTunnel
              </div>
              <div className="px-6 py-4 text-sm font-medium text-center text-muted">
                ngrok
              </div>
            </div>
            {comparison.map((row, i) => (
              <div
                key={row.feature}
                className={`grid grid-cols-3 ${
                  i < comparison.length - 1 ? "border-b border-border" : ""
                }`}
              >
                <div className="px-6 py-4 text-sm">{row.feature}</div>
                <div className="px-6 py-4 flex justify-center">
                  {row.subtunnel === true ? (
                    <Check className="w-5 h-5 text-accent" />
                  ) : row.subtunnel === false ? (
                    <X className="w-5 h-5 text-red-400/60" />
                  ) : (
                    <span className="text-sm text-accent">{String(row.subtunnel)}</span>
                  )}
                </div>
                <div className="px-6 py-4 flex justify-center">
                  {row.ngrok === true ? (
                    <Check className="w-5 h-5 text-muted" />
                  ) : row.ngrok === false ? (
                    <X className="w-5 h-5 text-red-400/60" />
                  ) : (
                    <span className="text-sm text-muted">{String(row.ngrok)}</span>
                  )}
                </div>
              </div>
            ))}
          </div>
        </div>
      </section>

      {/* Install */}
      <section id="install" className="py-24 md:py-32 border-t border-border/50">
        <div className="mx-auto max-w-4xl px-6">
          <div className="text-center mb-12">
            <h2 className="text-3xl md:text-4xl font-bold tracking-tight">
              Install in seconds
            </h2>
            <p className="mt-4 text-muted">
              One command. No dependencies. Works on macOS and Linux.
            </p>
          </div>

          {/* curl command */}
          <div className="max-w-2xl mx-auto mb-12">
            <div className="bg-surface border border-border rounded-xl overflow-hidden">
              <div className="flex items-center justify-between px-4 py-3 bg-surface-2 border-b border-border">
                <div className="flex items-center gap-2">
                  <Terminal className="w-3.5 h-3.5 text-muted" />
                  <span className="text-xs text-muted font-mono">Install</span>
                </div>
              </div>
              <div className="p-6 font-mono text-sm">
                <span className="text-accent">$</span>{" "}
                <span className="select-all">curl -sSL https://raw.githubusercontent.com/winterwindgames/subtunnel/main/apps/web/public/install.sh | sh</span>
              </div>
            </div>
          </div>

          {/* Platform cards */}
          <div className="grid md:grid-cols-3 gap-6 max-w-3xl mx-auto">
            <div className="rounded-xl border border-border bg-surface/50 p-6 text-center">
              <div className="text-2xl mb-3">🍎</div>
              <h3 className="font-semibold mb-1">macOS</h3>
              <p className="text-xs text-muted mb-3">Apple Silicon &amp; Intel</p>
              <code className="text-xs text-accent bg-accent/10 px-2 py-1 rounded">curl -sSL … | sh</code>
            </div>
            <div className="rounded-xl border border-border bg-surface/50 p-6 text-center">
              <div className="text-2xl mb-3">🐧</div>
              <h3 className="font-semibold mb-1">Linux</h3>
              <p className="text-xs text-muted mb-3">x86_64 &amp; ARM64</p>
              <code className="text-xs text-accent bg-accent/10 px-2 py-1 rounded">curl -sSL … | sh</code>
            </div>
            <div className="rounded-xl border border-border bg-surface/50 p-6 text-center">
              <div className="text-2xl mb-3">🪟</div>
              <h3 className="font-semibold mb-1">Windows</h3>
              <p className="text-xs text-muted mb-3">x86_64</p>
              <a
                href="https://github.com/winterwindgames/subtunnel/releases/latest"
                className="text-xs text-accent hover:underline"
              >
                Download from GitHub →
              </a>
            </div>
          </div>

          <div className="text-center mt-8">
            <a
              href="https://github.com/winterwindgames/subtunnel/releases/latest"
              className="text-sm text-muted hover:text-foreground transition-colors"
            >
              All releases on GitHub →
            </a>
          </div>
        </div>
      </section>

      {/* Bottom CTA */}
      <section className="py-24 md:py-32 border-t border-border/50">
        <div className="mx-auto max-w-4xl px-6 text-center">
          <div className="relative rounded-2xl border border-border bg-surface/50 p-12 md:p-16 overflow-hidden">
            <div className="absolute inset-0 bg-[radial-gradient(ellipse_at_center,rgba(0,212,255,0.06),transparent_70%)]" />
            <div className="relative">
              <h2 className="text-3xl md:text-4xl font-bold tracking-tight">
                Ready to tunnel?
              </h2>
              <p className="mt-4 text-muted max-w-md mx-auto">
                Get started in under 60 seconds. No credit card required.
                Self-host or use our managed service.
              </p>
              <div className="flex flex-col sm:flex-row items-center justify-center gap-4 mt-8">
                <Link
                  href="/docs"
                  className="inline-flex h-11 items-center gap-2 rounded-lg bg-accent px-6 text-sm font-medium text-black hover:bg-accent/90 transition-colors"
                >
                  Get Started Free
                  <ArrowRight className="w-4 h-4" />
                </Link>
                <Link
                  href="/docs"
                  className="inline-flex h-11 items-center gap-2 rounded-lg border border-border px-6 text-sm font-medium text-foreground hover:bg-surface-2 transition-colors"
                >
                  Read the Docs
                </Link>
              </div>
            </div>
          </div>
        </div>
      </section>
    </div>
  );
}
