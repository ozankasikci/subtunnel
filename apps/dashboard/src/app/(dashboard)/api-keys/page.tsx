"use client";

import { useState } from "react";
import { apiKeys as initialKeys } from "@/lib/mock-data";
import { formatDate, formatRelative } from "@/lib/utils";
import { Key, Plus, Copy, Trash2, X, Eye, EyeOff } from "lucide-react";
import { useToast } from "@/components/toast";

export default function ApiKeysPage() {
  const [keys, setKeys] = useState(initialKeys);
  const [showModal, setShowModal] = useState(false);
  const [newKeyName, setNewKeyName] = useState("");
  const [revealedKeys, setRevealedKeys] = useState<Set<string>>(new Set());
  const { toast } = useToast();

  const createKey = () => {
    if (!newKeyName.trim()) return;
    const id = `key_${Date.now()}`;
    setKeys((prev) => [
      {
        id,
        name: newKeyName,
        key: `st_live_${Math.random().toString(36).slice(2, 22)}`,
        maskedKey: "st_live_xxxx...xxxx",
        createdAt: new Date().toISOString(),
        lastUsed: null,
      },
      ...prev,
    ]);
    setNewKeyName("");
    setShowModal(false);
    toast("API key created");
  };

  const revokeKey = (id: string) => {
    setKeys((prev) => prev.filter((k) => k.id !== id));
    toast("API key revoked");
  };

  const copyKey = (key: string) => {
    navigator.clipboard?.writeText(key);
    toast("Key copied to clipboard");
  };

  const toggleReveal = (id: string) => {
    setRevealedKeys((prev) => {
      const next = new Set(prev);
      next.has(id) ? next.delete(id) : next.add(id);
      return next;
    });
  };

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold">API Keys</h1>
          <p className="text-muted mt-1">Manage authentication tokens for the CLI and API</p>
        </div>
        <button
          onClick={() => setShowModal(true)}
          className="inline-flex items-center gap-2 bg-accent hover:bg-accent-hover text-black font-medium rounded-lg px-4 py-2 text-sm transition-colors"
        >
          <Plus className="h-4 w-4" /> Create Key
        </button>
      </div>

      {keys.length === 0 ? (
        <div className="bg-card border border-border rounded-xl p-12 text-center">
          <Key className="h-12 w-12 text-muted mx-auto mb-4" />
          <h2 className="text-lg font-semibold">No API keys</h2>
          <p className="text-muted text-sm mt-2">Create an API key to authenticate with the CLI or API.</p>
        </div>
      ) : (
        <div className="bg-card border border-border rounded-xl overflow-hidden">
          <div className="overflow-x-auto">
            <table className="w-full text-sm">
              <thead>
                <tr className="border-b border-border text-muted text-left">
                  <th className="px-5 py-3 font-medium">Name</th>
                  <th className="px-5 py-3 font-medium">Key</th>
                  <th className="px-5 py-3 font-medium">Created</th>
                  <th className="px-5 py-3 font-medium">Last Used</th>
                  <th className="px-5 py-3 font-medium">Actions</th>
                </tr>
              </thead>
              <tbody>
                {keys.map((k) => (
                  <tr key={k.id} className="border-b border-border last:border-0 hover:bg-card-hover transition-colors">
                    <td className="px-5 py-4 font-medium">{k.name}</td>
                    <td className="px-5 py-4 font-mono text-muted">
                      <span>{revealedKeys.has(k.id) ? k.key : k.maskedKey}</span>
                    </td>
                    <td className="px-5 py-4 text-muted">{formatDate(k.createdAt)}</td>
                    <td className="px-5 py-4 text-muted">{k.lastUsed ? formatRelative(k.lastUsed) : "Never"}</td>
                    <td className="px-5 py-4">
                      <div className="flex items-center gap-2">
                        <button onClick={() => toggleReveal(k.id)} className="p-1.5 rounded hover:bg-background text-muted hover:text-foreground transition-colors" title="Toggle visibility">
                          {revealedKeys.has(k.id) ? <EyeOff className="h-4 w-4" /> : <Eye className="h-4 w-4" />}
                        </button>
                        <button onClick={() => copyKey(k.key)} className="p-1.5 rounded hover:bg-background text-muted hover:text-foreground transition-colors" title="Copy">
                          <Copy className="h-4 w-4" />
                        </button>
                        <button onClick={() => revokeKey(k.id)} className="p-1.5 rounded hover:bg-background text-muted hover:text-danger transition-colors" title="Revoke">
                          <Trash2 className="h-4 w-4" />
                        </button>
                      </div>
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        </div>
      )}

      {/* Modal */}
      {showModal && (
        <div className="fixed inset-0 z-50 flex items-center justify-center">
          <div className="absolute inset-0 bg-black/60" onClick={() => setShowModal(false)} />
          <div className="relative bg-card border border-border rounded-xl p-6 w-full max-w-md mx-4">
            <div className="flex items-center justify-between mb-4">
              <h2 className="text-lg font-semibold">Create API Key</h2>
              <button onClick={() => setShowModal(false)} className="text-muted hover:text-foreground">
                <X className="h-5 w-5" />
              </button>
            </div>
            <label className="block text-sm text-muted mb-2">Key Name</label>
            <input
              type="text"
              value={newKeyName}
              onChange={(e) => setNewKeyName(e.target.value)}
              onKeyDown={(e) => e.key === "Enter" && createKey()}
              placeholder="e.g., CI/CD Pipeline"
              className="w-full bg-background border border-border rounded-lg px-3 py-2 text-sm focus:outline-none focus:border-accent"
              autoFocus
            />
            <div className="flex justify-end gap-3 mt-6">
              <button onClick={() => setShowModal(false)} className="px-4 py-2 text-sm text-muted hover:text-foreground transition-colors">
                Cancel
              </button>
              <button
                onClick={createKey}
                disabled={!newKeyName.trim()}
                className="px-4 py-2 text-sm bg-accent hover:bg-accent-hover text-black font-medium rounded-lg transition-colors disabled:opacity-50"
              >
                Create
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
