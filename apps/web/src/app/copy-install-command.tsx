"use client";

import { Check, Copy } from "lucide-react";
import { useState } from "react";

const installCommand =
  "curl -sSL https://www.subtunnel.dev/install.sh | sh";

export function CopyInstallCommand() {
  const [copied, setCopied] = useState(false);

  async function copyCommand() {
    await navigator.clipboard.writeText(installCommand);
    setCopied(true);
    window.setTimeout(() => setCopied(false), 2000);
  }

  return (
    <div className="flex min-w-0 items-center gap-3 rounded-xl border border-border bg-surface px-4 py-3 sm:px-5">
      <code className="min-w-0 flex-1 overflow-x-auto whitespace-nowrap font-mono text-xs text-foreground sm:text-sm">
        {installCommand}
      </code>
      <button
        type="button"
        onClick={copyCommand}
        className="inline-flex h-9 shrink-0 items-center gap-2 rounded-lg border border-border bg-surface-2 px-3 text-xs font-medium text-muted transition-colors hover:border-foreground/20 hover:text-foreground focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-accent"
        aria-label={copied ? "Install command copied" : "Copy install command"}
      >
        {copied ? (
          <Check className="h-3.5 w-3.5 text-accent" aria-hidden="true" />
        ) : (
          <Copy className="h-3.5 w-3.5" aria-hidden="true" />
        )}
        <span className="hidden sm:inline">{copied ? "Copied" : "Copy"}</span>
      </button>
    </div>
  );
}
