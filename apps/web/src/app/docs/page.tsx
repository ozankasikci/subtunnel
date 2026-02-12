import Link from "next/link";
import { ArrowRight, Terminal, BookOpen, Zap, Settings } from "lucide-react";

function CodeBlock({
  title,
  children,
}: {
  title?: string;
  children: React.ReactNode;
}) {
  return (
    <div className="bg-surface border border-border rounded-xl overflow-hidden">
      {title && (
        <div className="flex items-center gap-2 px-4 py-3 bg-surface-2 border-b border-border">
          <Terminal className="w-3.5 h-3.5 text-muted" />
          <span className="text-xs text-muted font-mono">{title}</span>
        </div>
      )}
      <pre className="p-6 font-mono text-sm leading-relaxed overflow-x-auto">
        {children}
      </pre>
    </div>
  );
}

const sideNav = [
  { icon: Zap, label: "Quick Start", href: "#quickstart" },
  { icon: Settings, label: "Installation", href: "#installation" },
  { icon: Terminal, label: "CLI Usage", href: "#usage" },
  { icon: BookOpen, label: "Configuration", href: "#configuration" },
];

export default function DocsPage() {
  return (
    <div className="mx-auto max-w-6xl px-6 py-16 md:py-24">
      <div className="grid lg:grid-cols-[220px_1fr] gap-12">
        {/* Sidebar */}
        <aside className="hidden lg:block">
          <div className="sticky top-24">
            <h4 className="text-xs font-medium text-muted uppercase tracking-wider mb-4">
              Documentation
            </h4>
            <nav className="space-y-1">
              {sideNav.map((item) => (
                <a
                  key={item.href}
                  href={item.href}
                  className="flex items-center gap-2.5 rounded-lg px-3 py-2 text-sm text-muted hover:text-foreground hover:bg-surface transition-colors"
                >
                  <item.icon className="w-4 h-4" />
                  {item.label}
                </a>
              ))}
            </nav>
          </div>
        </aside>

        {/* Content */}
        <div className="max-w-3xl">
          <div className="mb-12">
            <h1 className="text-3xl md:text-4xl font-bold tracking-tight">
              Documentation
            </h1>
            <p className="mt-4 text-lg text-muted">
              Get up and running with SubTunnel in under a minute.
            </p>
          </div>

          {/* Quick Start */}
          <section id="quickstart" className="scroll-mt-24 mb-16">
            <h2 className="text-2xl font-bold tracking-tight mb-4 flex items-center gap-3">
              <div className="flex h-8 w-8 items-center justify-center rounded-lg bg-accent/10 border border-accent/20">
                <Zap className="w-4 h-4 text-accent" />
              </div>
              Quick Start
            </h2>
            <p className="text-muted mb-6 leading-relaxed">
              Three commands to expose your local server to the internet:
            </p>
            <CodeBlock title="Terminal">
              <code>
                <span className="text-muted"># 1. Install SubTunnel</span>
                {"\n"}
                <span className="text-accent">$</span> curl -fsSL
                https://get.subtunnel.dev | sh{"\n\n"}
                <span className="text-muted"># 2. Login (optional for managed service)</span>
                {"\n"}
                <span className="text-accent">$</span> subtunnel login{"\n\n"}
                <span className="text-muted"># 3. Start a tunnel</span>
                {"\n"}
                <span className="text-accent">$</span> subtunnel http 3000
              </code>
            </CodeBlock>
            <p className="mt-4 text-sm text-muted">
              That&apos;s it. Your local server on port 3000 is now accessible via a
              public URL.
            </p>
          </section>

          {/* Installation */}
          <section id="installation" className="scroll-mt-24 mb-16">
            <h2 className="text-2xl font-bold tracking-tight mb-4 flex items-center gap-3">
              <div className="flex h-8 w-8 items-center justify-center rounded-lg bg-accent/10 border border-accent/20">
                <Settings className="w-4 h-4 text-accent" />
              </div>
              Installation
            </h2>
            <p className="text-muted mb-6 leading-relaxed">
              Install the SubTunnel CLI on macOS, Linux, or Windows.
            </p>

            <h3 className="text-lg font-semibold mb-3">macOS / Linux</h3>
            <CodeBlock>
              <code>
                <span className="text-accent">$</span> curl -fsSL
                https://get.subtunnel.dev | sh
              </code>
            </CodeBlock>

            <h3 className="text-lg font-semibold mt-8 mb-3">
              Homebrew (macOS)
            </h3>
            <CodeBlock>
              <code>
                <span className="text-accent">$</span> brew install
                subtunnel/tap/subtunnel
              </code>
            </CodeBlock>

            <h3 className="text-lg font-semibold mt-8 mb-3">npm</h3>
            <CodeBlock>
              <code>
                <span className="text-accent">$</span> npm install -g subtunnel
              </code>
            </CodeBlock>

            <h3 className="text-lg font-semibold mt-8 mb-3">Docker</h3>
            <CodeBlock>
              <code>
                <span className="text-accent">$</span> docker run -it
                subtunnel/cli http 3000
              </code>
            </CodeBlock>

            <h3 className="text-lg font-semibold mt-8 mb-3">
              Self-Host the Server
            </h3>
            <p className="text-muted mb-4 text-sm leading-relaxed">
              To run your own SubTunnel server, use Docker Compose:
            </p>
            <CodeBlock title="docker-compose.yml">
              <code>
                <span className="text-accent">$</span> git clone
                https://github.com/subtunnel/subtunnel{"\n"}
                <span className="text-accent">$</span> cd subtunnel{"\n"}
                <span className="text-accent">$</span> cp .env.example .env
                {"\n"}
                <span className="text-accent">$</span> docker compose up -d
              </code>
            </CodeBlock>
          </section>

          {/* CLI Usage */}
          <section id="usage" className="scroll-mt-24 mb-16">
            <h2 className="text-2xl font-bold tracking-tight mb-4 flex items-center gap-3">
              <div className="flex h-8 w-8 items-center justify-center rounded-lg bg-accent/10 border border-accent/20">
                <Terminal className="w-4 h-4 text-accent" />
              </div>
              CLI Usage
            </h2>

            <div className="space-y-8">
              <div>
                <h3 className="text-lg font-semibold mb-3">HTTP Tunnel</h3>
                <p className="text-muted text-sm mb-4">
                  Expose a local HTTP server to the internet.
                </p>
                <CodeBlock>
                  <code>
                    <span className="text-accent">$</span> subtunnel http 3000
                    {"\n\n"}
                    <span className="text-muted"># With a custom domain</span>
                    {"\n"}
                    <span className="text-accent">$</span> subtunnel http 3000
                    --domain api.example.com{"\n\n"}
                    <span className="text-muted"># With basic auth</span>
                    {"\n"}
                    <span className="text-accent">$</span> subtunnel http 3000
                    --auth &quot;user:pass&quot;
                  </code>
                </CodeBlock>
              </div>

              <div>
                <h3 className="text-lg font-semibold mb-3">TCP Tunnel</h3>
                <p className="text-muted text-sm mb-4">
                  Expose any TCP service (databases, SSH, game servers).
                </p>
                <CodeBlock>
                  <code>
                    <span className="text-muted"># Expose PostgreSQL</span>
                    {"\n"}
                    <span className="text-accent">$</span> subtunnel tcp 5432
                    {"\n\n"}
                    <span className="text-muted"># Expose SSH</span>
                    {"\n"}
                    <span className="text-accent">$</span> subtunnel tcp 22
                  </code>
                </CodeBlock>
              </div>

              <div>
                <h3 className="text-lg font-semibold mb-3">
                  Status & Management
                </h3>
                <CodeBlock>
                  <code>
                    <span className="text-muted"># List active tunnels</span>
                    {"\n"}
                    <span className="text-accent">$</span> subtunnel list
                    {"\n\n"}
                    <span className="text-muted"># Check tunnel status</span>
                    {"\n"}
                    <span className="text-accent">$</span> subtunnel status
                    {"\n\n"}
                    <span className="text-muted"># Stop all tunnels</span>
                    {"\n"}
                    <span className="text-accent">$</span> subtunnel stop --all
                  </code>
                </CodeBlock>
              </div>
            </div>
          </section>

          {/* Configuration */}
          <section id="configuration" className="scroll-mt-24 mb-16">
            <h2 className="text-2xl font-bold tracking-tight mb-4 flex items-center gap-3">
              <div className="flex h-8 w-8 items-center justify-center rounded-lg bg-accent/10 border border-accent/20">
                <BookOpen className="w-4 h-4 text-accent" />
              </div>
              Configuration
            </h2>
            <p className="text-muted mb-6 leading-relaxed">
              SubTunnel can be configured via a{" "}
              <code className="text-xs bg-surface-2 border border-border rounded px-1.5 py-0.5 font-mono">
                subtunnel.yml
              </code>{" "}
              file in your project root:
            </p>
            <CodeBlock title="subtunnel.yml">
              <code>
                <span className="text-accent">server</span>:{"\n"}
                {"  "}
                <span className="text-muted">url</span>: https://tunnel.example.com
                {"\n\n"}
                <span className="text-accent">tunnels</span>:{"\n"}
                {"  "}
                <span className="text-muted">web</span>:{"\n"}
                {"    "}proto: http{"\n"}
                {"    "}addr: 3000{"\n"}
                {"    "}domain: app.example.com{"\n\n"}
                {"  "}
                <span className="text-muted">api</span>:{"\n"}
                {"    "}proto: http{"\n"}
                {"    "}addr: 8080{"\n"}
                {"    "}domain: api.example.com
              </code>
            </CodeBlock>
            <p className="mt-4 text-sm text-muted">
              Then start all tunnels with:{" "}
              <code className="text-xs bg-surface-2 border border-border rounded px-1.5 py-0.5 font-mono">
                subtunnel start
              </code>
            </p>
          </section>

          {/* Next steps */}
          <div className="rounded-xl border border-border bg-surface/50 p-8">
            <h3 className="text-lg font-semibold mb-2">Need help?</h3>
            <p className="text-sm text-muted mb-4">
              Join our community or check the full API reference.
            </p>
            <div className="flex flex-wrap gap-3">
              <a
                href="https://github.com/subtunnel"
                className="inline-flex h-9 items-center gap-2 rounded-lg border border-border bg-surface px-4 text-sm hover:bg-surface-2 transition-colors"
              >
                GitHub
                <ArrowRight className="w-3 h-3" />
              </a>
              <a
                href="https://discord.gg/subtunnel"
                className="inline-flex h-9 items-center gap-2 rounded-lg border border-border bg-surface px-4 text-sm hover:bg-surface-2 transition-colors"
              >
                Discord
                <ArrowRight className="w-3 h-3" />
              </a>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}
