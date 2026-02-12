"use client";

import { usageData, tunnels } from "@/lib/mock-data";
import { BarChart, Bar, XAxis, YAxis, Tooltip, ResponsiveContainer, AreaChart, Area } from "recharts";

const CustomTooltip = ({ active, payload, label }: any) => {
  if (!active || !payload?.length) return null;
  return (
    <div className="bg-[#1a1a1a] border border-[#262626] rounded-lg px-3 py-2 text-xs">
      <p className="text-[#737373] mb-1">{label}</p>
      {payload.map((p: any) => (
        <p key={p.name} style={{ color: p.color }}>
          {p.name}: {p.value.toLocaleString()}{p.name === "Bandwidth" ? " MB" : ""}
        </p>
      ))}
    </div>
  );
};

export default function UsagePage() {
  return (
    <div className="space-y-8">
      <div>
        <h1 className="text-2xl font-bold">Usage</h1>
        <p className="text-muted mt-1">Monitor requests and bandwidth across your tunnels</p>
      </div>

      {/* Requests Chart */}
      <div className="bg-card border border-border rounded-xl p-5">
        <h2 className="text-base font-semibold mb-4">Requests (Last 30 Days)</h2>
        <div className="h-64">
          <ResponsiveContainer width="100%" height="100%">
            <AreaChart data={usageData}>
              <defs>
                <linearGradient id="reqGrad" x1="0" y1="0" x2="0" y2="1">
                  <stop offset="0%" stopColor="#10b981" stopOpacity={0.3} />
                  <stop offset="100%" stopColor="#10b981" stopOpacity={0} />
                </linearGradient>
              </defs>
              <XAxis dataKey="date" tick={{ fontSize: 11, fill: "#737373" }} tickFormatter={(v) => v.slice(5)} axisLine={false} tickLine={false} />
              <YAxis tick={{ fontSize: 11, fill: "#737373" }} axisLine={false} tickLine={false} />
              <Tooltip content={<CustomTooltip />} />
              <Area type="monotone" dataKey="requests" name="Requests" stroke="#10b981" fill="url(#reqGrad)" strokeWidth={2} />
            </AreaChart>
          </ResponsiveContainer>
        </div>
      </div>

      {/* Bandwidth Chart */}
      <div className="bg-card border border-border rounded-xl p-5">
        <h2 className="text-base font-semibold mb-4">Bandwidth (Last 30 Days)</h2>
        <div className="h-64">
          <ResponsiveContainer width="100%" height="100%">
            <BarChart data={usageData}>
              <XAxis dataKey="date" tick={{ fontSize: 11, fill: "#737373" }} tickFormatter={(v) => v.slice(5)} axisLine={false} tickLine={false} />
              <YAxis tick={{ fontSize: 11, fill: "#737373" }} axisLine={false} tickLine={false} />
              <Tooltip content={<CustomTooltip />} />
              <Bar dataKey="bandwidth" name="Bandwidth" fill="#6366f1" radius={[3, 3, 0, 0]} />
            </BarChart>
          </ResponsiveContainer>
        </div>
      </div>

      {/* Per-tunnel breakdown */}
      <div className="bg-card border border-border rounded-xl p-5">
        <h2 className="text-base font-semibold mb-4">Per-Tunnel Breakdown</h2>
        <div className="space-y-4">
          {tunnels.map((t) => {
            const maxReq = Math.max(...tunnels.map((x) => x.requestsToday), 1);
            const pct = (t.requestsToday / maxReq) * 100;
            return (
              <div key={t.id}>
                <div className="flex items-center justify-between text-sm mb-1.5">
                  <span className="font-medium">{t.subdomain}.subtunnel.dev</span>
                  <span className="text-muted">{t.requestsToday.toLocaleString()} req · {t.bandwidthMB} MB</span>
                </div>
                <div className="h-2 bg-background rounded-full overflow-hidden">
                  <div className="h-full bg-accent rounded-full transition-all" style={{ width: `${pct}%` }} />
                </div>
              </div>
            );
          })}
        </div>
      </div>
    </div>
  );
}
