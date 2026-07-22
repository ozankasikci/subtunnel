import Link from "next/link";
import { ArrowRight, Terminal, Zap, Settings, Server } from "lucide-react";

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
  { icon: Server, label: "Self-Hosting", href: "#self-hosting" },
  { icon: Terminal, label: "CLI Reference", href: "#cli-reference" },
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
              Install the CLI, point it at your server, and your local port is live on the internet.
            </p>
            <CodeBlock title="Terminal">
              <code>
                <span className="text-muted"># 1. Install SubTunnel</span>
                {"\n"}
                <span className="text-accent">$</span> curl -sSL https://www.subtunnel.dev/install.sh | sh{"\n\n"}
                <span className="text-muted"># 2. Expose a local port through your server</span>
                {"\n"}
                <span className="text-accent">$</span> subtunnel local 3000 \{"\n"}
                {"    "}--to your-server.example.com:7835 \{"\n"}
                {"    "}--token YOUR_TOKEN \{"\n"}
                {"    "}--subdomain myapp
              </code>
            </CodeBlock>
            <p className="mt-4 text-sm text-muted">
              Your local server on port 3000 is now accessible at{" "}
              <code className="text-xs bg-surface-2 border border-border rounded px-1.5 py-0.5 font-mono">
                https://myapp.your-domain.com
              </code>
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
              Install the SubTunnel CLI on macOS or Linux. A single binary, no dependencies.
            </p>

            <h3 className="text-lg font-semibold mb-3">macOS / Linux</h3>
            <CodeBlock>
              <code>
                <span className="text-accent">$</span> curl -sSL https://www.subtunnel.dev/install.sh | sh
              </code>
            </CodeBlock>
            <p className="mt-3 text-sm text-muted">
              Supports macOS (Apple Silicon &amp; Intel) and Linux (x86_64 &amp; ARM64).
              The script detects your platform, downloads the latest release, and installs to{" "}
              <code className="text-xs bg-surface-2 border border-border rounded px-1.5 py-0.5 font-mono">
                /usr/local/bin
              </code>.
            </p>

            <h3 className="text-lg font-semibold mt-8 mb-3">
              Manual Download
            </h3>
            <p className="text-muted mb-4 text-sm leading-relaxed">
              Download the binary for your platform from the{" "}
              <a href="https://github.com/ozankasikci/subtunnel/releases" className="text-accent hover:underline">
                GitHub Releases
              </a>{" "}
              page, extract it, and place it in your PATH.
            </p>
          </section>

          {/* Self-Hosting */}
          <section id="self-hosting" className="scroll-mt-24 mb-16">
            <h2 className="text-2xl font-bold tracking-tight mb-4 flex items-center gap-3">
              <div className="flex h-8 w-8 items-center justify-center rounded-lg bg-accent/10 border border-accent/20">
                <Server className="w-4 h-4 text-accent" />
              </div>
              Self-Hosting the Server
            </h2>
            <p className="text-muted mb-6 leading-relaxed">
              SubTunnel is fully self-hosted. You run the server on your own VPS (EC2, DigitalOcean, Hetzner, etc.)
              and connect clients to it. Here&apos;s how to set it up from scratch.
            </p>

            <h3 className="text-lg font-semibold mb-3">Prerequisites</h3>
            <ul className="text-muted text-sm mb-8 space-y-2 list-disc list-inside">
              <li>A VPS with a public IP address (any Linux distro)</li>
              <li>A domain name with DNS access (e.g. Cloudflare, Route 53)</li>
              <li>Ports 7835 (control plane) and 8080 (HTTP traffic) open in your security group / firewall</li>
            </ul>

            <h3 className="text-lg font-semibold mb-3">1. DNS Setup</h3>
            <p className="text-muted mb-4 text-sm leading-relaxed">
              Point your domain and a wildcard subdomain to your server&apos;s IP address.
              This allows SubTunnel to route traffic to tunnels based on subdomain.
            </p>
            <CodeBlock title="DNS Records">
              <code>
                <span className="text-muted"># Replace 203.0.113.10 with your server&apos;s public IP</span>
                {"\n\n"}
                <span className="text-accent">A</span>{"     "}tunnel.example.com{"      "}→ 203.0.113.10{"\n"}
                <span className="text-accent">A</span>{"     "}*.tunnel.example.com{"    "}→ 203.0.113.10
              </code>
            </CodeBlock>

            <h3 className="text-lg font-semibold mt-8 mb-3">2. Install SubTunnel on Your Server</h3>
            <CodeBlock title="SSH into your server">
              <code>
                <span className="text-accent">$</span> curl -sSL https://www.subtunnel.dev/install.sh | sh
              </code>
            </CodeBlock>

            <h3 className="text-lg font-semibold mt-8 mb-3">3. Generate a Token</h3>
            <p className="text-muted mb-4 text-sm leading-relaxed">
              Create a shared secret that clients will use to authenticate with the server.
            </p>
            <CodeBlock>
              <code>
                <span className="text-accent">$</span> openssl rand -hex 16{"\n"}
                <span className="text-muted"># e.g. a1b2c3d4e5f6a7b8c9d0e1f2a3b4c5d6</span>
              </code>
            </CodeBlock>

            <h3 className="text-lg font-semibold mt-8 mb-3">4. Set Up nginx (TLS Termination)</h3>
            <p className="text-muted mb-4 text-sm leading-relaxed">
              Use nginx as a reverse proxy to handle TLS termination and forward HTTP traffic
              to SubTunnel&apos;s HTTP listener. This gives you automatic HTTPS via Let&apos;s Encrypt.
            </p>
            <CodeBlock title="nginx.conf">
              <code>
                <span className="text-muted"># Wildcard HTTPS — routes *.tunnel.example.com to SubTunnel</span>
                {"\n"}
                server {"{"}{"\n"}
                {"    "}listen 443 ssl;{"\n"}
                {"    "}server_name *.tunnel.example.com;{"\n"}
                {"\n"}
                {"    "}ssl_certificate /etc/letsencrypt/live/tunnel.example.com/fullchain.pem;{"\n"}
                {"    "}ssl_certificate_key /etc/letsencrypt/live/tunnel.example.com/privkey.pem;{"\n"}
                {"\n"}
                {"    "}location / {"{"}{"\n"}
                {"        "}proxy_pass http://127.0.0.1:8080;{"\n"}
                {"        "}proxy_set_header Host $host;{"\n"}
                {"        "}proxy_set_header X-Real-IP $remote_addr;{"\n"}
                {"        "}proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;{"\n"}
                {"        "}proxy_set_header X-Forwarded-Proto $scheme;{"\n"}
                {"    "}{"}"}{"\n"}
                {"}"}
              </code>
            </CodeBlock>
            <p className="mt-3 text-sm text-muted">
              For wildcard certificates, use DNS-based validation with certbot:{" "}
              <code className="text-xs bg-surface-2 border border-border rounded px-1.5 py-0.5 font-mono">
                certbot certonly --dns-cloudflare -d tunnel.example.com -d *.tunnel.example.com
              </code>
            </p>

            <h3 className="text-lg font-semibold mt-8 mb-3">5. Start the Server</h3>
            <CodeBlock>
              <code>
                <span className="text-accent">$</span> subtunnel server \{"\n"}
                {"    "}--domain tunnel.example.com \{"\n"}
                {"    "}--token YOUR_TOKEN \{"\n"}
                {"    "}--port 7835 \{"\n"}
                {"    "}--http-port 8080
              </code>
            </CodeBlock>

            <h3 className="text-lg font-semibold mt-8 mb-3">6. Run as a systemd Service</h3>
            <p className="text-muted mb-4 text-sm leading-relaxed">
              For production, run SubTunnel as a systemd service so it starts on boot and auto-restarts.
            </p>
            <CodeBlock title="/etc/systemd/system/subtunnel.service">
              <code>
                [Unit]{"\n"}
                Description=SubTunnel Server{"\n"}
                After=network.target{"\n"}
                {"\n"}
                [Service]{"\n"}
                Type=simple{"\n"}
                User=subtunnel{"\n"}
                ExecStart=/usr/local/bin/subtunnel server \{"\n"}
                {"    "}--domain tunnel.example.com \{"\n"}
                {"    "}--token YOUR_TOKEN \{"\n"}
                {"    "}--port 7835 \{"\n"}
                {"    "}--http-port 8080{"\n"}
                Restart=always{"\n"}
                RestartSec=5{"\n"}
                {"\n"}
                [Install]{"\n"}
                WantedBy=multi-user.target
              </code>
            </CodeBlock>
            <CodeBlock title="Enable and start">
              <code>
                <span className="text-accent">$</span> sudo systemctl enable subtunnel{"\n"}
                <span className="text-accent">$</span> sudo systemctl start subtunnel{"\n"}
                <span className="text-accent">$</span> sudo systemctl status subtunnel
              </code>
            </CodeBlock>

            <h3 className="text-lg font-semibold mt-8 mb-3">7. Connect a Client</h3>
            <p className="text-muted mb-4 text-sm leading-relaxed">
              From your local machine, connect to your server:
            </p>
            <CodeBlock>
              <code>
                <span className="text-accent">$</span> subtunnel local 3000 \{"\n"}
                {"    "}--to your-server.example.com:7835 \{"\n"}
                {"    "}--token YOUR_TOKEN \{"\n"}
                {"    "}--subdomain myapp{"\n\n"}
                <span className="text-muted"># ✓ Connected</span>{"\n"}
                <span className="text-muted"># Forwarding: https://myapp.tunnel.example.com → localhost:3000</span>
              </code>
            </CodeBlock>

            <div className="mt-8 rounded-xl border border-accent/20 bg-accent/5 p-6">
              <h4 className="text-sm font-semibold mb-2">Architecture Overview</h4>
              <pre className="text-xs text-muted font-mono leading-relaxed">
{`Internet → nginx (TLS :443) → HTTP listener (:8080) → route by Host header
                                                         ↕ yamux streams
Client (subtunnel local) ←— TLS + yamux (:7835) ——→ SubTunnel Server
        ↕                                              (control + data)
   localhost:PORT`}
              </pre>
            </div>
          </section>

          {/* CLI Reference */}
          <section id="cli-reference" className="scroll-mt-24 mb-16">
            <h2 className="text-2xl font-bold tracking-tight mb-4 flex items-center gap-3">
              <div className="flex h-8 w-8 items-center justify-center rounded-lg bg-accent/10 border border-accent/20">
                <Terminal className="w-4 h-4 text-accent" />
              </div>
              CLI Reference
            </h2>

            <div className="space-y-8">
              <div>
                <h3 className="text-lg font-semibold mb-3">subtunnel server</h3>
                <p className="text-muted text-sm mb-4">
                  Run the SubTunnel server on your VPS. This is the public-facing component that accepts client connections and routes traffic.
                </p>
                <CodeBlock>
                  <code>
                    <span className="text-accent">$</span> subtunnel server --domain tunnel.example.com --token SECRET{"\n\n"}
                    <span className="text-muted"># All options:</span>{"\n"}
                    <span className="text-accent">$</span> subtunnel server \{"\n"}
                    {"    "}--domain tunnel.example.com \{"  "}<span className="text-muted"># Required: base domain for subdomains</span>{"\n"}
                    {"    "}--token SECRET \{"              "}<span className="text-muted"># Auth token clients must provide</span>{"\n"}
                    {"    "}--port 7835 \{"               "}<span className="text-muted"># Control plane port (default: 7835)</span>{"\n"}
                    {"    "}--http-port 8080 \{"           "}<span className="text-muted"># HTTP listener port (default: 8080)</span>{"\n"}
                    {"    "}--host 0.0.0.0 \{"             "}<span className="text-muted"># Bind address (default: 0.0.0.0)</span>{"\n"}
                    {"    "}--extra-domain other.com{"      "}<span className="text-muted"># Accept additional domains</span>
                  </code>
                </CodeBlock>
              </div>

              <div>
                <h3 className="text-lg font-semibold mb-3">subtunnel local</h3>
                <p className="text-muted text-sm mb-4">
                  Connect to a SubTunnel server and expose a local port to the internet.
                </p>
                <CodeBlock>
                  <code>
                    <span className="text-accent">$</span> subtunnel local 3000 --to server:7835 --token SECRET{"\n\n"}
                    <span className="text-muted"># With a custom subdomain:</span>{"\n"}
                    <span className="text-accent">$</span> subtunnel local 3000 \{"\n"}
                    {"    "}--to server.example.com:7835 \{"\n"}
                    {"    "}--token SECRET \{"\n"}
                    {"    "}--subdomain myapp{"             "}<span className="text-muted"># → myapp.tunnel.example.com</span>{"\n\n"}
                    <span className="text-muted"># Skip TLS verification (self-signed certs):</span>{"\n"}
                    <span className="text-accent">$</span> subtunnel local 3000 --to server:7835 --token SECRET --tls-verify false{"\n\n"}
                    <span className="text-muted"># Use a custom CA certificate:</span>{"\n"}
                    <span className="text-accent">$</span> subtunnel local 3000 --to server:7835 --token SECRET --tls-ca /path/to/ca.pem
                  </code>
                </CodeBlock>
              </div>
            </div>
          </section>

          {/* Next steps */}
          <div className="rounded-xl border border-border bg-surface/50 p-8">
            <h3 className="text-lg font-semibold mb-2">Questions or feedback?</h3>
            <p className="text-sm text-muted mb-4">
              Open an issue in the SubTunnel repository.
            </p>
            <a
              href="https://github.com/ozankasikci/subtunnel/issues"
              className="inline-flex h-9 items-center gap-2 rounded-lg border border-border bg-surface px-4 text-sm hover:bg-surface-2 transition-colors"
            >
              GitHub Issues
              <ArrowRight className="w-3 h-3" />
            </a>
          </div>
        </div>
      </div>
    </div>
  );
}
