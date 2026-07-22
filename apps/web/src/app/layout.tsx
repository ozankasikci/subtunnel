import type { Metadata } from "next";
import { Geist, Geist_Mono } from "next/font/google";
import Link from "next/link";
import "./globals.css";

const githubUrl = "https://github.com/ozankasikci/subtunnel";

const geistSans = Geist({
  variable: "--font-geist-sans",
  subsets: ["latin"],
});

const geistMono = Geist_Mono({
  variable: "--font-geist-mono",
  subsets: ["latin"],
});

export const metadata: Metadata = {
  title: "SubTunnel | Expose localhost to the internet",
  description:
    "Self-hosted tunnels on your own domain. One binary, MIT licensed.",
  openGraph: {
    title: "SubTunnel | Expose localhost to the internet",
    description:
      "Self-hosted tunnels on your own domain. One binary, MIT licensed.",
    type: "website",
  },
};

function Wordmark() {
  return (
    <span className="flex items-center gap-2.5">
      <span
        className="flex h-5 w-5 items-center justify-center rounded-full border border-accent/80"
        aria-hidden="true"
      >
        <span className="h-1.5 w-1.5 rounded-full bg-accent" />
      </span>
      <span className="font-semibold tracking-[-0.02em] text-foreground">
        SubTunnel
      </span>
    </span>
  );
}

function Header() {
  return (
    <header className="sticky top-0 z-50 border-b border-border bg-background/90 backdrop-blur-xl">
      <div className="mx-auto flex h-14 max-w-6xl items-center justify-between px-6">
        <Link href="/" aria-label="SubTunnel home">
          <Wordmark />
        </Link>
        <nav className="flex items-center gap-6" aria-label="Primary navigation">
          <Link
            href="/docs"
            className="text-sm text-muted transition-colors hover:text-foreground"
          >
            Docs
          </Link>
          <a
            href={githubUrl}
            className="text-sm text-muted transition-colors hover:text-foreground"
          >
            GitHub
          </a>
        </nav>
      </div>
    </header>
  );
}

function Footer() {
  return (
    <footer className="px-6">
      <div className="mx-auto flex max-w-6xl flex-col gap-6 py-10 sm:flex-row sm:items-center sm:justify-between">
        <Link href="/" aria-label="SubTunnel home">
          <Wordmark />
        </Link>
        <nav className="flex flex-wrap items-center gap-x-6 gap-y-3" aria-label="Footer navigation">
          <a
            href={githubUrl}
            className="text-sm text-muted transition-colors hover:text-foreground"
          >
            GitHub
          </a>
          <Link
            href="/docs"
            className="text-sm text-muted transition-colors hover:text-foreground"
          >
            Docs
          </Link>
          <a
            href={`${githubUrl}/blob/main/LICENSE`}
            className="text-sm text-muted transition-colors hover:text-foreground"
          >
            MIT License
          </a>
        </nav>
      </div>
    </footer>
  );
}

export default function RootLayout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  return (
    <html lang="en" className="dark">
      <body
        className={`${geistSans.variable} ${geistMono.variable} min-h-screen antialiased`}
      >
        <Header />
        <main>{children}</main>
        <Footer />
      </body>
    </html>
  );
}
