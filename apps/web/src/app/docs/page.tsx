import type { ReactNode } from "react";

const installCommand =
  "curl -sSL https://www.subtunnel.dev/install.sh | sh";

const sideNav = [
  { label: "Quick Start", href: "#quickstart" },
  { label: "Installation", href: "#installation" },
  { label: "Self-Hosting", href: "#self-hosting" },
  { label: "CLI Reference", href: "#cli-reference" },
];

const serverFlags = [
  ["--port", "Control-plane listen port. Default: 7835."],
  [
    "--http-port",
    "HTTP listener receiving proxied traffic from nginx. Default: 8080.",
  ],
  ["--host", "Bind address. Default: 0.0.0.0."],
  ["--domain", "Required. Domain for tunnel subdomains."],
  ["--extra-domain", "Additional accepted domain. Repeatable."],
  [
    "--token",
    "Authentication token agents must provide. Env: SUBTUNNEL_TOKEN.",
  ],
  ["--tls-cert", "TLS certificate PEM path."],
  ["--tls-key", "TLS private key PEM path."],
];

const localFlags = [
  ["<port>", "Positional local port to expose."],
  ["--to", "Server address in host:port format."],
  ["--token", "Authentication token. Env: SUBTUNNEL_TOKEN."],
  ["--subdomain", "Request a specific subdomain."],
  [
    "--tls-verify",
    "Verify the server TLS certificate. Default: true. Set false for self-signed certificates.",
  ],
  ["--tls-ca", "Custom CA certificate PEM path."],
];

function SectionHeader({ kicker, children }: { kicker: string; children: ReactNode }) {
  return (
    <div>
      <p className="mb-4 text-xs font-semibold uppercase tracking-[0.24em] text-accent">
        {kicker}
      </p>
      <h2 className="max-w-2xl text-3xl font-bold leading-tight tracking-[-0.04em] text-foreground sm:text-4xl">
        {children}
      </h2>
    </div>
  );
}

function TerminalWindow({ children }: { children: ReactNode }) {
  return (
    <div className="overflow-hidden rounded-2xl border border-border bg-surface text-left shadow-2xl shadow-black/30">
      <div className="grid h-11 grid-cols-[1fr_auto_1fr] items-center border-b border-border bg-surface-2 px-4">
        <div className="flex gap-1.5" aria-hidden="true">
          <span className="h-2.5 w-2.5 rounded-full bg-foreground/25" />
          <span className="h-2.5 w-2.5 rounded-full bg-foreground/15" />
          <span className="h-2.5 w-2.5 rounded-full bg-accent/70" />
        </div>
        <span className="font-mono text-[11px] text-muted">
          subtunnel · zsh
        </span>
        <span aria-hidden="true" />
      </div>
      <div className="overflow-x-auto p-5 sm:p-7">{children}</div>
    </div>
  );
}

function CodeBlock({
  children,
  label,
}: {
  children: ReactNode;
  label?: string;
}) {
  return (
    <div className="overflow-hidden rounded-xl border border-border bg-surface">
      {label ? (
        <div className="border-b border-border bg-surface-2 px-4 py-2.5 font-mono text-[11px] text-muted">
          {label}
        </div>
      ) : null}
      <pre className="overflow-x-auto p-4 font-mono text-xs leading-6 text-foreground sm:p-5">
        <code>{children}</code>
      </pre>
    </div>
  );
}

function FlagTable({ rows }: { rows: string[][] }) {
  return (
    <div className="overflow-hidden rounded-2xl border border-border">
      <table className="w-full border-collapse text-left">
        <thead className="bg-surface-2">
          <tr>
            <th className="w-40 px-4 py-3 text-xs font-semibold uppercase tracking-[0.16em] text-muted sm:w-52 sm:px-5">
              Flag
            </th>
            <th className="px-4 py-3 text-xs font-semibold uppercase tracking-[0.16em] text-muted sm:px-5">
              Description
            </th>
          </tr>
        </thead>
        <tbody>
          {rows.map(([flag, description]) => (
            <tr key={flag} className="border-t border-border bg-surface">
              <td className="px-4 py-4 align-top sm:px-5">
                <code className="font-mono text-xs text-accent sm:text-sm">
                  {flag}
                </code>
              </td>
              <td className="px-4 py-4 text-sm leading-6 text-muted sm:px-5">
                {description}
              </td>
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}

function GuideStep({
  number,
  title,
  description,
  children,
}: {
  number: string;
  title: string;
  description: ReactNode;
  children: ReactNode;
}) {
  return (
    <li className="bg-background p-6 sm:p-8">
      <div className="grid gap-5 sm:grid-cols-[48px_1fr] sm:gap-7">
        <span className="font-mono text-xs text-accent">{number}</span>
        <div className="min-w-0">
          <h3 className="font-semibold tracking-tight text-foreground">
            {title}
          </h3>
          <div className="mt-2 text-sm leading-6 text-muted">{description}</div>
          <div className="mt-5">{children}</div>
        </div>
      </div>
    </li>
  );
}

export default function DocsPage() {
  return (
    <div className="mx-auto max-w-6xl px-6 py-20 sm:py-28">
      <div className="grid gap-16 lg:grid-cols-[190px_minmax(0,1fr)] lg:gap-20">
        <aside className="hidden lg:block">
          <div className="sticky top-24">
            <p className="mb-5 text-xs font-semibold uppercase tracking-[0.24em] text-muted">
              Documentation
            </p>
            <nav className="flex flex-col border-l border-border" aria-label="Documentation">
              {sideNav.map((item) => (
                <a
                  key={item.href}
                  href={item.href}
                  className="-ml-px border-l border-transparent py-2.5 pl-4 text-sm text-muted transition-colors hover:border-accent hover:text-foreground"
                >
                  {item.label}
                </a>
              ))}
            </nav>
          </div>
        </aside>

        <div className="min-w-0 max-w-4xl">
          <section id="quickstart" className="scroll-mt-24 pb-24 sm:pb-32">
            <SectionHeader kicker="Quick Start">
              Up and running in a minute
            </SectionHeader>
            <p className="mt-5 max-w-2xl text-base leading-7 text-muted sm:text-lg">
              Install the CLI, connect it to your SubTunnel server, and expose a
              local port on your own domain.
            </p>

            <div className="mt-10">
              <TerminalWindow>
                <div className="min-w-[720px] font-mono text-[13px] leading-7 text-foreground sm:text-sm">
                  <p className="whitespace-pre">
                    <span className="text-accent">$</span> {installCommand}
                  </p>
                  <p className="h-7" aria-hidden="true" />
                  <p className="whitespace-pre">
                    <span className="text-accent">$</span>{" "}
                    {"subtunnel local 3000 --to your-server.example.com:7835 \\"}
                  </p>
                  <p className="whitespace-pre">
                    {"    --token YOUR_TOKEN --subdomain myapp"}
                  </p>
                  <p className="mt-2 whitespace-pre text-muted">
                    {"Forwarding "}
                    <span className="text-accent">
                      https://myapp.your-server.example.com
                    </span>
                    {" -> localhost:3000"}
                  </p>
                </div>
              </TerminalWindow>
            </div>
          </section>

          <section
            id="installation"
            className="scroll-mt-24 border-t border-border py-24 sm:py-32"
          >
            <SectionHeader kicker="Installation">Install the binary</SectionHeader>
            <p className="mt-5 max-w-2xl text-base leading-7 text-muted">
              The install script detects your platform and places one static
              binary in <code className="font-mono text-sm text-foreground">/usr/local/bin</code>.
              It supports macOS and Linux.
            </p>

            <div className="mt-8">
              <CodeBlock>
                <span className="text-accent">$</span> {installCommand}
              </CodeBlock>
            </div>

            <p className="mt-5 text-sm leading-6 text-muted">
              <span className="font-semibold text-foreground">Manual download.</span>{" "}
              Download the binary for your platform from{" "}
              <a
                href="https://github.com/ozankasikci/subtunnel/releases"
                className="text-foreground underline decoration-border underline-offset-4 transition-colors hover:decoration-accent"
              >
                GitHub Releases
              </a>
              , extract it, and place it in your PATH.
            </p>
          </section>

          <section
            id="self-hosting"
            className="scroll-mt-24 border-t border-border py-24 sm:py-32"
          >
            <SectionHeader kicker="Self-Hosting">Run your own server</SectionHeader>
            <p className="mt-5 max-w-3xl text-base leading-7 text-muted sm:text-lg">
              Everything below runs on one small VPS. You need a domain and a
              server with ports 80, 443, and 7835 reachable.
            </p>

            <ol className="mt-12 grid gap-px overflow-hidden rounded-2xl border border-border bg-border">
              <GuideStep
                number="01"
                title="Point DNS at your server"
                description="Create A records for the tunnel domain and its wildcard, both pointing to your server's public IP address."
              >
                <CodeBlock label="DNS records">
                  <span className="text-accent">A</span>{"     "}
                  tunnel.example.com{"      "}→ 203.0.113.10{"\n"}
                  <span className="text-accent">A</span>{"     "}
                  *.tunnel.example.com{"    "}→ 203.0.113.10
                </CodeBlock>
              </GuideStep>

              <GuideStep
                number="02"
                title="Install SubTunnel on the server"
                description="SSH into the VPS and install the same static binary used by the client."
              >
                <CodeBlock>
                  <span className="text-accent">$</span> {installCommand}
                </CodeBlock>
              </GuideStep>

              <GuideStep
                number="03"
                title="Generate a token"
                description="Create a shared secret that clients will use to authenticate with the server."
              >
                <CodeBlock>
                  <span className="text-accent">$</span> openssl rand -hex 16
                </CodeBlock>
              </GuideStep>

              <GuideStep
                number="04"
                title="Set up nginx TLS termination"
                description="Use nginx to terminate TLS and forward HTTP traffic to SubTunnel's HTTP listener on port 8080."
              >
                <CodeBlock label="nginx.conf">
                  <span className="text-muted">
                    # Wildcard HTTPS routes *.tunnel.example.com to SubTunnel
                  </span>
                  {"\n"}
                  {'server {'}{"\n"}
                  {"    "}listen 443 ssl;{"\n"}
                  {"    "}server_name *.tunnel.example.com;{"\n\n"}
                  {"    "}ssl_certificate /etc/letsencrypt/live/tunnel.example.com/fullchain.pem;{"\n"}
                  {"    "}ssl_certificate_key /etc/letsencrypt/live/tunnel.example.com/privkey.pem;{"\n\n"}
                  {"    "}location / {'{'}{"\n"}
                  {"        "}proxy_pass http://127.0.0.1:8080;{"\n"}
                  {"        "}proxy_set_header Host $host;{"\n"}
                  {"        "}proxy_set_header X-Real-IP $remote_addr;{"\n"}
                  {"        "}proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;{"\n"}
                  {"        "}proxy_set_header X-Forwarded-Proto $scheme;{"\n"}
                  {"    "}{'}'}{"\n"}
                  {'}'}
                </CodeBlock>
                <p className="mt-4 text-sm leading-6 text-muted">
                  For wildcard certificates, use DNS-based validation with
                  certbot:{" "}
                  <code className="font-mono text-xs text-foreground">
                    certbot certonly --dns-cloudflare -d tunnel.example.com -d
                    *.tunnel.example.com
                  </code>
                  .
                </p>
              </GuideStep>

              <GuideStep
                number="05"
                title="Start the server"
                description="Start the control plane on port 7835 and the internal HTTP listener on port 8080."
              >
                <CodeBlock>
                  <span className="text-accent">$</span> subtunnel server --domain tunnel.example.com --token YOUR_TOKEN --port 7835 --http-port 8080
                </CodeBlock>
              </GuideStep>

              <GuideStep
                number="06"
                title="Run as a systemd service"
                description="For production, run SubTunnel as a systemd service so it starts on boot and restarts automatically."
              >
                <div className="space-y-4">
                  <CodeBlock label="/etc/systemd/system/subtunnel.service">
                    [Unit]{"\n"}
                    Description=SubTunnel Server{"\n"}
                    After=network.target{"\n\n"}
                    [Service]{"\n"}
                    Type=simple{"\n"}
                    User=subtunnel{"\n"}
                    ExecStart=/usr/local/bin/subtunnel server {"\\"}{"\n"}
                    {"    "}--domain tunnel.example.com {"\\"}{"\n"}
                    {"    "}--token YOUR_TOKEN {"\\"}{"\n"}
                    {"    "}--port 7835 {"\\"}{"\n"}
                    {"    "}--http-port 8080{"\n"}
                    Restart=always{"\n"}
                    RestartSec=5{"\n\n"}
                    [Install]{"\n"}
                    WantedBy=multi-user.target
                  </CodeBlock>
                  <CodeBlock label="Enable and start">
                    <span className="text-accent">$</span> sudo systemctl enable subtunnel{"\n"}
                    <span className="text-accent">$</span> sudo systemctl start subtunnel{"\n"}
                    <span className="text-accent">$</span> sudo systemctl status subtunnel
                  </CodeBlock>
                </div>
              </GuideStep>

              <GuideStep
                number="07"
                title="Connect a client"
                description="From your local machine, connect a port to the server and request its public subdomain."
              >
                <CodeBlock>
                  <span className="text-accent">$</span> subtunnel local 3000 --to tunnel.example.com:7835 --token YOUR_TOKEN --subdomain myapp
                </CodeBlock>
              </GuideStep>
            </ol>
          </section>

          <section
            id="cli-reference"
            className="scroll-mt-24 border-t border-border py-24 sm:py-32"
          >
            <SectionHeader kicker="CLI Reference">
              Four focused command areas.
            </SectionHeader>
            <p className="mt-5 max-w-2xl text-base leading-7 text-muted">
              Run the public server, connect one local port directly, start
              configured tunnels, or manage the client as a native service.
            </p>

            <div className="mt-12 space-y-14">
              <div>
                <h3 className="font-mono text-lg font-semibold text-foreground">
                  subtunnel server
                </h3>
                <p className="mb-6 mt-2 text-sm leading-6 text-muted">
                  Run the public-facing server that accepts client connections
                  and routes traffic.
                </p>
                <FlagTable rows={serverFlags} />
              </div>

              <div>
                <h3 className="font-mono text-lg font-semibold text-foreground">
                  subtunnel local
                </h3>
                <p className="mb-6 mt-2 text-sm leading-6 text-muted">
                  Connect to a SubTunnel server and expose one local port.
                </p>
                <FlagTable rows={localFlags} />
              </div>

              <div>
                <h3 className="font-mono text-lg font-semibold text-foreground">
                  subtunnel run
                </h3>
                <p className="mb-6 mt-2 text-sm leading-6 text-muted">
                  Read a TOML config and start all tunnels, or a named subset,
                  in one process. Each tunnel has its own connection and
                  reconnect loop. Keeping the token in the config avoids
                  exposing it in the process list.
                </p>
                <CodeBlock label="config.toml">
                  {'server = "tunnel.example.com:7835"'}{"\n"}
                  {'token = "YOUR_TOKEN"'}{"\n\n"}
                  {"[tunnels.myapp]"}{"\n"}
                  {"local_port = 3000"}{"\n"}
                  {'subdomain = "myapp"'}
                </CodeBlock>
                <div className="mt-4">
                  <CodeBlock>
                    <span className="text-accent">$</span>
                    {" subtunnel run --all\n"}
                    <span className="text-accent">$</span>
                    {" subtunnel run myapp --config /absolute/path/to/config.toml"}
                  </CodeBlock>
                </div>
              </div>

              <div>
                <h3 className="font-mono text-lg font-semibold text-foreground">
                  subtunnel service
                </h3>
                <p className="mb-6 mt-2 text-sm leading-6 text-muted">
                  Install the configured client agent as a systemd service on
                  Linux or a launchd service on macOS. Normal users install a
                  user service. Running the command with sudo installs a system
                  service. SubTunnel does not manage your application process.
                </p>
                <CodeBlock>
                  <span className="text-accent">$</span>
                  {
                    " subtunnel service install --config /absolute/path/to/config.toml\n"
                  }
                  <span className="text-accent">$</span>
                  {" subtunnel service status\n"}
                  <span className="text-accent">$</span>
                  {
                    " subtunnel service generate systemd --config /absolute/path/to/config.toml"
                  }
                </CodeBlock>
              </div>
            </div>
          </section>

          <aside className="border-t border-border pt-16" aria-labelledby="feedback-title">
            <div className="rounded-2xl border border-border bg-surface p-7 sm:flex sm:items-center sm:justify-between sm:gap-8 sm:p-8">
              <div>
                <h2 id="feedback-title" className="font-semibold text-foreground">
                  Questions or feedback?
                </h2>
                <p className="mt-2 text-sm leading-6 text-muted">
                  Open an issue in the SubTunnel repository.
                </p>
              </div>
              <a
                href="https://github.com/ozankasikci/subtunnel/issues"
                className="mt-5 inline-flex items-center gap-2 text-sm font-medium text-foreground underline decoration-border underline-offset-4 transition-colors hover:decoration-accent sm:mt-0"
              >
                GitHub Issues <span className="text-accent" aria-hidden="true">↗</span>
              </a>
            </div>
          </aside>
        </div>
      </div>
    </div>
  );
}
