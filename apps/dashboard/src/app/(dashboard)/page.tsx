import { Network, BarChart3, HardDrive, Key, ArrowRight, Terminal, Zap, Globe, Clock } from "lucide-react";
import { stats, recentActivity, tunnels } from "@/lib/mock-data";
import { formatRelative } from "@/lib/utils";
import Link from "next/link";

const statCards = [
  { label: "Active Tunnels", value: stats.activeTunnels, icon: Network, color: "text-emerald-400" },
  { label: "Requests Today", value: stats.totalRequestsToday.toLocaleString(), icon: BarChart3, color: "text-blue-400" },
  { label: "Bandwidth Used", value: `${stats.bandwidthUsedMB.toFixed(1)} MB`, icon: HardDrive, color: "text-purple-400" },
  { label: "API Keys", value: stats.apiKeyCount, icon: Key, color: "text-amber-400" },
];

const activityIcons: Record<string, typeof Zap> = {
  tunnel_created: Zap,
  tunnel_stopped: Network,
  key_created: Key,
  domain_added: Globe,
  settings_changed: Clock,
};

export default function OverviewPage() {
  const hasTunnels = tunnels.length > 0;

  return (
    <div className="space-y-8">
      <div>
        <h1 className="text-2xl font-bold">Overview</h1>
        <p className="text-muted mt-1">Welcome back, Ozan</p>
      </div>

      {/* Stats */}
      <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-4">
        {statCards.map((s) => (
          <div key={s.label} className="bg-card border border-border rounded-xl p-5">
            <div className="flex items-center justify-between">
              <p className="text-sm text-muted">{s.label}</p>
              <s.icon className={`h-5 w-5 ${s.color}`} />
            </div>
            <p className="text-2xl font-bold mt-2">{s.value}</p>
          </div>
        ))}
      </div>

      <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
        {/* Recent Activity */}
        <div className="bg-card border border-border rounded-xl p-5">
          <h2 className="text-base font-semibold mb-4">Recent Activity</h2>
          <div className="space-y-3">
            {recentActivity.map((a) => {
              const Icon = activityIcons[a.type] || Zap;
              return (
                <div key={a.id} className="flex items-start gap-3">
                  <div className="mt-0.5 h-7 w-7 rounded-full bg-accent/10 flex items-center justify-center shrink-0">
                    <Icon className="h-3.5 w-3.5 text-accent" />
                  </div>
                  <div className="flex-1 min-w-0">
                    <p className="text-sm">{a.message}</p>
                    <p className="text-xs text-muted mt-0.5">{formatRelative(a.timestamp)}</p>
                  </div>
                </div>
              );
            })}
          </div>
        </div>

        {/* Quick Start */}
        {!hasTunnels ? (
          <div className="bg-card border border-border rounded-xl p-6 flex flex-col items-center justify-center text-center">
            <Network className="h-12 w-12 text-accent mb-4" />
            <h2 className="text-lg font-semibold">No tunnels yet</h2>
            <p className="text-muted text-sm mt-2 max-w-xs">
              Create your first tunnel to expose a local service to the internet.
            </p>
            <Link
              href="/tunnels"
              className="mt-4 inline-flex items-center gap-2 bg-accent hover:bg-accent-hover text-black font-medium rounded-lg px-4 py-2 text-sm transition-colors"
            >
              Get Started <ArrowRight className="h-4 w-4" />
            </Link>
          </div>
        ) : (
          <div className="bg-card border border-border rounded-xl p-5">
            <h2 className="text-base font-semibold mb-4">Quick Start</h2>
            <p className="text-sm text-muted mb-4">Start a tunnel from your terminal:</p>
            <div className="bg-background rounded-lg p-4 font-mono text-sm border border-border">
              <div className="flex items-center gap-2 text-muted mb-2">
                <Terminal className="h-4 w-4" />
                <span>Terminal</span>
              </div>
              <p className="text-accent">$ subtunnel start --port 3000</p>
              <p className="text-muted mt-1">→ https://api-dev.subtunnel.dev</p>
            </div>
            <Link
              href="/tunnels"
              className="mt-4 inline-flex items-center gap-1 text-accent hover:text-accent-hover text-sm font-medium transition-colors"
            >
              View all tunnels <ArrowRight className="h-3.5 w-3.5" />
            </Link>
          </div>
        )}
      </div>
    </div>
  );
}
