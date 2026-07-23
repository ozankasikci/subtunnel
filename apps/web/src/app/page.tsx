import Link from "next/link";
import { CopyInstallCommand } from "./copy-install-command";

const features = [
  {
    title: "Self-hosted",
    description:
      "Runs on your own VPS. Your traffic never touches a third party.",
  },
  {
    title: "Wildcard + custom domains",
    description:
      "Every tunnel gets a subdomain on your domain, or bring any domain you own.",
  },
  {
    title: "TLS via nginx + Let's Encrypt",
    description:
      "HTTPS with tooling you already trust. No proprietary edge.",
  },
  {
    title: "Token-based auth",
    description: "Only clients holding your token can open a tunnel.",
  },
  {
    title: "Single static binary",
    description: "Written in Rust. No runtime, no dependencies to install.",
  },
  {
    title: "MIT open source",
    description: "Read the code. Fork it. It's yours.",
  },
];

const steps = [
  {
    number: "01",
    title: "Point DNS",
    description: (
      <>
        Point a wildcard A record for{" "}
        <code className="font-mono text-xs text-foreground">
          *.tunnel.example.com
        </code>{" "}
        to your server.
      </>
    ),
  },
  {
    number: "02",
    title: "Run the server",
    description: (
      <>
        Install SubTunnel and run{" "}
        <code className="font-mono text-xs text-foreground">
          subtunnel server
        </code>{" "}
        on the VPS.
      </>
    ),
  },
  {
    number: "03",
    title: "Connect",
    description: (
      <>
        Run{" "}
        <code className="font-mono text-xs text-foreground">
          subtunnel local
        </code>{" "}
        from your machine.
      </>
    ),
  },
];

function TerminalDemo() {
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
      <div className="overflow-x-auto p-5 sm:p-7">
        <div className="min-w-[660px] font-mono text-[13px] leading-7 text-foreground sm:text-sm">
          <p className="whitespace-pre">
            <span className="text-accent">$</span>{" "}
            {"subtunnel local 3000 --to tunnel.example.com:7835 \\"}
          </p>
          <p className="whitespace-pre">
            {"    --token TOKEN --subdomain myapp"}
          </p>
          <p className="whitespace-pre">
            {"Forwarding "}
            <span className="text-accent">
              {"https://myapp.tunnel.example.com"}
            </span>
            {" -> localhost:3000"}
            <span className="terminal-cursor relative top-0.5 ml-1 inline-block h-[1em] w-2 bg-accent" />
          </p>
        </div>
      </div>
    </div>
  );
}

export default function HomePage() {
  return (
    <div>
      <section className="px-6 pb-24 pt-24 sm:pb-32 sm:pt-32 lg:pb-36 lg:pt-40">
        <div className="mx-auto max-w-4xl text-center">
          <h1 className="text-5xl font-bold leading-[0.98] tracking-[-0.055em] text-foreground sm:text-6xl md:text-7xl lg:text-[5.25rem]">
            Expose localhost to
            <br />
            the internet
          </h1>
          <p className="mx-auto mt-7 max-w-2xl text-lg leading-8 text-muted sm:text-xl">
            Self-hosted tunnels on your own domain. One binary, MIT licensed.
          </p>

          <div className="mx-auto mt-14 max-w-3xl sm:mt-16">
            <TerminalDemo />
          </div>
        </div>
      </section>

      <section className="border-y border-border px-6 py-20 sm:py-24">
        <div className="mx-auto max-w-3xl">
          <p className="mb-4 text-xs font-semibold uppercase tracking-[0.24em] text-accent">
            Install
          </p>
          <CopyInstallCommand />
          <p className="mt-4 text-sm text-muted">
            macOS and Linux. Or download the binary from{" "}
            <a
              href="https://github.com/ozankasikci/subtunnel/releases"
              className="text-foreground underline decoration-border underline-offset-4 transition-colors hover:decoration-accent"
            >
              GitHub Releases
            </a>
            .
          </p>
        </div>
      </section>

      <section className="px-6 py-24 sm:py-32">
        <div className="mx-auto grid max-w-6xl overflow-hidden rounded-2xl border border-border bg-border gap-px sm:grid-cols-2 lg:grid-cols-3">
          {features.map((feature) => (
            <article key={feature.title} className="min-h-44 bg-background p-7 sm:p-8">
              <div className="mb-6 h-1.5 w-1.5 rounded-full bg-accent" />
              <h2 className="font-semibold tracking-tight text-foreground">
                {feature.title}
              </h2>
              <p className="mt-2 text-sm leading-6 text-muted">
                {feature.description}
              </p>
            </article>
          ))}
        </div>
      </section>

      <section className="border-y border-border px-6 py-20 sm:py-24">
        <div className="mx-auto max-w-6xl">
          <div className="mb-12 flex flex-col gap-4 sm:flex-row sm:items-end sm:justify-between">
            <div>
              <p className="mb-3 text-xs font-semibold uppercase tracking-[0.24em] text-accent">
                Self-host
              </p>
              <h2 className="text-3xl font-bold tracking-[-0.035em] text-foreground sm:text-4xl">
                Self-host in 3 steps
              </h2>
            </div>
            <Link
              href="/docs"
              className="text-sm font-medium text-muted underline decoration-border underline-offset-4 transition-colors hover:text-foreground hover:decoration-accent"
            >
              Read the full guide
            </Link>
          </div>

          <ol className="grid gap-px overflow-hidden rounded-2xl border border-border bg-border md:grid-cols-3">
            {steps.map((step) => (
              <li key={step.number} className="bg-surface p-7 sm:p-8">
                <span className="font-mono text-xs text-accent">
                  {step.number}
                </span>
                <h3 className="mt-8 font-semibold text-foreground">
                  {step.title}
                </h3>
                <p className="mt-2 max-w-xs text-sm leading-6 text-muted">
                  {step.description}
                </p>
              </li>
            ))}
          </ol>
        </div>
      </section>
    </div>
  );
}
