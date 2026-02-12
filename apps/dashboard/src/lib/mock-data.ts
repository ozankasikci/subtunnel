export interface Tunnel {
  id: string;
  subdomain: string;
  localPort: number;
  status: "online" | "offline";
  createdAt: string;
  lastActive: string;
  requestsToday: number;
  bandwidthMB: number;
}

export interface ApiKey {
  id: string;
  name: string;
  key: string;
  maskedKey: string;
  createdAt: string;
  lastUsed: string | null;
}

export interface Domain {
  id: string;
  subdomain: string;
  type: "reserved" | "custom";
  cname?: string;
  verified: boolean;
  createdAt: string;
}

export interface Activity {
  id: string;
  type: "tunnel_created" | "tunnel_stopped" | "key_created" | "domain_added" | "settings_changed";
  message: string;
  timestamp: string;
}

export interface UsageDataPoint {
  date: string;
  requests: number;
  bandwidth: number;
}

export interface TeamMember {
  id: string;
  name: string;
  email: string;
  role: "owner" | "admin" | "member";
  avatar?: string;
  joinedAt: string;
}

export const tunnels: Tunnel[] = [
  {
    id: "tun_1",
    subdomain: "api-dev",
    localPort: 3000,
    status: "online",
    createdAt: "2026-01-15T10:30:00Z",
    lastActive: "2026-02-12T19:30:00Z",
    requestsToday: 1247,
    bandwidthMB: 45.2,
  },
  {
    id: "tun_2",
    subdomain: "webhook-test",
    localPort: 8080,
    status: "online",
    createdAt: "2026-01-20T14:00:00Z",
    lastActive: "2026-02-12T19:28:00Z",
    requestsToday: 532,
    bandwidthMB: 12.8,
  },
  {
    id: "tun_3",
    subdomain: "staging-app",
    localPort: 4200,
    status: "offline",
    createdAt: "2026-02-01T09:15:00Z",
    lastActive: "2026-02-10T16:45:00Z",
    requestsToday: 0,
    bandwidthMB: 0,
  },
];

export const apiKeys: ApiKey[] = [
  {
    id: "key_1",
    name: "CI/CD Pipeline",
    key: "st_live_a1b2c3d4e5f6g7h8i9j0",
    maskedKey: "st_live_a1b2...j0",
    createdAt: "2026-01-10T08:00:00Z",
    lastUsed: "2026-02-12T18:00:00Z",
  },
  {
    id: "key_2",
    name: "Local Development",
    key: "st_live_k1l2m3n4o5p6q7r8s9t0",
    maskedKey: "st_live_k1l2...t0",
    createdAt: "2026-01-25T12:00:00Z",
    lastUsed: "2026-02-12T15:30:00Z",
  },
  {
    id: "key_3",
    name: "Staging Server",
    key: "st_live_u1v2w3x4y5z6a7b8c9d0",
    maskedKey: "st_live_u1v2...d0",
    createdAt: "2026-02-05T10:00:00Z",
    lastUsed: null,
  },
];

export const domains: Domain[] = [
  { id: "dom_1", subdomain: "api-dev", type: "reserved", verified: true, createdAt: "2026-01-15T10:30:00Z" },
  { id: "dom_2", subdomain: "webhook-test", type: "reserved", verified: true, createdAt: "2026-01-20T14:00:00Z" },
  { id: "dom_3", subdomain: "staging-app", type: "reserved", verified: true, createdAt: "2026-02-01T09:15:00Z" },
  { id: "dom_4", subdomain: "tunnel.example.com", type: "custom", cname: "cname.subtunnel.dev", verified: true, createdAt: "2026-02-08T11:00:00Z" },
  { id: "dom_5", subdomain: "dev.myapp.io", type: "custom", cname: "cname.subtunnel.dev", verified: false, createdAt: "2026-02-11T16:00:00Z" },
];

export const recentActivity: Activity[] = [
  { id: "act_1", type: "tunnel_created", message: 'Tunnel "api-dev" started on port 3000', timestamp: "2026-02-12T19:30:00Z" },
  { id: "act_2", type: "key_created", message: 'API key "Staging Server" created', timestamp: "2026-02-12T15:00:00Z" },
  { id: "act_3", type: "tunnel_stopped", message: 'Tunnel "staging-app" went offline', timestamp: "2026-02-10T16:45:00Z" },
  { id: "act_4", type: "domain_added", message: 'Custom domain "dev.myapp.io" added', timestamp: "2026-02-11T16:00:00Z" },
  { id: "act_5", type: "settings_changed", message: "Team member invited: alice@example.com", timestamp: "2026-02-09T10:00:00Z" },
];

export const usageData: UsageDataPoint[] = Array.from({ length: 30 }, (_, i) => {
  const date = new Date("2026-01-14");
  date.setDate(date.getDate() + i);
  return {
    date: date.toISOString().split("T")[0],
    requests: Math.floor(Math.random() * 3000) + 500,
    bandwidth: Math.floor(Math.random() * 200) + 20,
  };
});

export const teamMembers: TeamMember[] = [
  { id: "usr_1", name: "Ozan", email: "ozan@subtunnel.dev", role: "owner", joinedAt: "2026-01-01T00:00:00Z" },
  { id: "usr_2", name: "Alice Chen", email: "alice@example.com", role: "admin", joinedAt: "2026-02-01T00:00:00Z" },
  { id: "usr_3", name: "Bob Smith", email: "bob@example.com", role: "member", joinedAt: "2026-02-09T00:00:00Z" },
];

export const stats = {
  activeTunnels: tunnels.filter((t) => t.status === "online").length,
  totalRequestsToday: tunnels.reduce((sum, t) => sum + t.requestsToday, 0),
  bandwidthUsedMB: tunnels.reduce((sum, t) => sum + t.bandwidthMB, 0),
  apiKeyCount: apiKeys.length,
};
