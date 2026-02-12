import { domains } from "@/lib/mock-data";
import { formatDate } from "@/lib/utils";
import { Globe, CheckCircle, AlertCircle, Copy } from "lucide-react";

export default function DomainsPage() {
  const reserved = domains.filter((d) => d.type === "reserved");
  const custom = domains.filter((d) => d.type === "custom");

  return (
    <div className="space-y-8">
      <div>
        <h1 className="text-2xl font-bold">Domains</h1>
        <p className="text-muted mt-1">Manage your reserved subdomains and custom domains</p>
      </div>

      {/* Reserved Subdomains */}
      <div>
        <h2 className="text-base font-semibold mb-3">Reserved Subdomains</h2>
        <div className="bg-card border border-border rounded-xl overflow-hidden">
          <div className="overflow-x-auto">
            <table className="w-full text-sm">
              <thead>
                <tr className="border-b border-border text-muted text-left">
                  <th className="px-5 py-3 font-medium">Subdomain</th>
                  <th className="px-5 py-3 font-medium">URL</th>
                  <th className="px-5 py-3 font-medium">Created</th>
                </tr>
              </thead>
              <tbody>
                {reserved.map((d) => (
                  <tr key={d.id} className="border-b border-border last:border-0 hover:bg-card-hover transition-colors">
                    <td className="px-5 py-4 font-medium">{d.subdomain}</td>
                    <td className="px-5 py-4 text-muted font-mono text-xs">{d.subdomain}.subtunnel.dev</td>
                    <td className="px-5 py-4 text-muted">{formatDate(d.createdAt)}</td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        </div>
      </div>

      {/* Custom Domains */}
      <div>
        <h2 className="text-base font-semibold mb-3">Custom Domains</h2>
        <div className="bg-card border border-border rounded-xl overflow-hidden">
          <div className="overflow-x-auto">
            <table className="w-full text-sm">
              <thead>
                <tr className="border-b border-border text-muted text-left">
                  <th className="px-5 py-3 font-medium">Domain</th>
                  <th className="px-5 py-3 font-medium">Status</th>
                  <th className="px-5 py-3 font-medium">CNAME Target</th>
                  <th className="px-5 py-3 font-medium">Created</th>
                </tr>
              </thead>
              <tbody>
                {custom.map((d) => (
                  <tr key={d.id} className="border-b border-border last:border-0 hover:bg-card-hover transition-colors">
                    <td className="px-5 py-4 font-medium">{d.subdomain}</td>
                    <td className="px-5 py-4">
                      {d.verified ? (
                        <span className="inline-flex items-center gap-1.5 text-xs font-medium text-emerald-400">
                          <CheckCircle className="h-3.5 w-3.5" /> Verified
                        </span>
                      ) : (
                        <span className="inline-flex items-center gap-1.5 text-xs font-medium text-amber-400">
                          <AlertCircle className="h-3.5 w-3.5" /> Pending
                        </span>
                      )}
                    </td>
                    <td className="px-5 py-4 font-mono text-xs text-muted">{d.cname}</td>
                    <td className="px-5 py-4 text-muted">{formatDate(d.createdAt)}</td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        </div>

        {/* CNAME Instructions */}
        <div className="mt-4 bg-card border border-border rounded-xl p-5">
          <h3 className="text-sm font-semibold mb-2">Setup Instructions</h3>
          <p className="text-sm text-muted mb-3">
            Add a CNAME record pointing your domain to SubTunnel:
          </p>
          <div className="bg-background rounded-lg p-4 font-mono text-sm border border-border">
            <div className="grid grid-cols-3 gap-4 text-muted mb-2">
              <span>Type</span><span>Name</span><span>Value</span>
            </div>
            <div className="grid grid-cols-3 gap-4">
              <span className="text-accent">CNAME</span>
              <span>your-domain.com</span>
              <span className="text-muted">cname.subtunnel.dev</span>
            </div>
          </div>
          <p className="text-xs text-muted mt-3">
            DNS changes may take up to 48 hours to propagate. Verification is automatic.
          </p>
        </div>
      </div>
    </div>
  );
}
