import { getCurrentUser } from "@/lib/auth";
import { prisma } from "@/lib/db";
import { formatDate, formatRelative } from "@/lib/utils";
import { Network, ExternalLink } from "lucide-react";

export default async function TunnelsPage() {
  const user = await getCurrentUser();
  const tunnels = await prisma.tunnel.findMany({
    where: { userId: user!.id },
    orderBy: { createdAt: "desc" },
  });

  const isEmpty = tunnels.length === 0;

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold">Tunnels</h1>
          <p className="text-muted mt-1">Manage your active and recent tunnels</p>
        </div>
      </div>

      {isEmpty ? (
        <div className="bg-card border border-border rounded-xl p-12 text-center">
          <Network className="h-12 w-12 text-muted mx-auto mb-4" />
          <h2 className="text-lg font-semibold">No tunnels yet</h2>
          <p className="text-muted text-sm mt-2 max-w-md mx-auto">
            Install the SubTunnel CLI and start your first tunnel:
          </p>
          <div className="mt-6 bg-background rounded-lg p-4 font-mono text-sm border border-border max-w-md mx-auto text-left">
            <p className="text-muted"># Install CLI</p>
            <p className="text-accent">$ curl -fsSL https://subtunnel.dev/install | sh</p>
            <p className="text-muted mt-3"># Login</p>
            <p className="text-accent">$ subtunnel login</p>
            <p className="text-muted mt-3"># Start a tunnel</p>
            <p className="text-accent">$ subtunnel start --port 3000</p>
          </div>
        </div>
      ) : (
        <div className="bg-card border border-border rounded-xl overflow-hidden">
          <div className="overflow-x-auto">
            <table className="w-full text-sm">
              <thead>
                <tr className="border-b border-border text-muted text-left">
                  <th className="px-5 py-3 font-medium">Subdomain</th>
                  <th className="px-5 py-3 font-medium">Status</th>
                  <th className="px-5 py-3 font-medium">Local Port</th>
                  <th className="px-5 py-3 font-medium">Created</th>
                  <th className="px-5 py-3 font-medium">Last Active</th>
                  <th className="px-5 py-3 font-medium"></th>
                </tr>
              </thead>
              <tbody>
                {tunnels.map((t) => (
                  <tr key={t.id} className="border-b border-border last:border-0 hover:bg-card-hover transition-colors">
                    <td className="px-5 py-4">
                      <div className="flex items-center gap-2">
                        <Network className="h-4 w-4 text-muted" />
                        <span className="font-medium">{t.subdomain}</span>
                        <span className="text-muted">.subtunnel.dev</span>
                      </div>
                    </td>
                    <td className="px-5 py-4">
                      <span className={`inline-flex items-center gap-1.5 text-xs font-medium px-2.5 py-1 rounded-full ${
                        t.status === "online"
                          ? "bg-emerald-500/10 text-emerald-400"
                          : "bg-zinc-500/10 text-zinc-400"
                      }`}>
                        <span className={`h-1.5 w-1.5 rounded-full ${
                          t.status === "online" ? "bg-emerald-400" : "bg-zinc-400"
                        }`} />
                        {t.status}
                      </span>
                    </td>
                    <td className="px-5 py-4 font-mono text-muted">{t.localPort}</td>
                    <td className="px-5 py-4 text-muted">{formatDate(t.createdAt.toISOString())}</td>
                    <td className="px-5 py-4 text-muted">{formatRelative(t.lastActive.toISOString())}</td>
                    <td className="px-5 py-4">
                      {t.status === "online" && (
                        <a
                          href={`https://${t.subdomain}.subtunnel.dev`}
                          target="_blank"
                          className="text-muted hover:text-foreground transition-colors"
                        >
                          <ExternalLink className="h-4 w-4" />
                        </a>
                      )}
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        </div>
      )}
    </div>
  );
}
