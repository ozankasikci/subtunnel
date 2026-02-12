"use client";

import { useState } from "react";
import { teamMembers } from "@/lib/mock-data";
import { useToast } from "@/components/toast";
import { Save, UserPlus, Shield, Crown, User, AlertTriangle } from "lucide-react";

export default function SettingsPage() {
  const { toast } = useToast();
  const [profile, setProfile] = useState({ name: "Ozan", email: "ozan@subtunnel.dev" });
  const [inviteEmail, setInviteEmail] = useState("");

  return (
    <div className="space-y-8 max-w-3xl">
      <div>
        <h1 className="text-2xl font-bold">Settings</h1>
        <p className="text-muted mt-1">Manage your account and team</p>
      </div>

      {/* Profile */}
      <div className="bg-card border border-border rounded-xl p-6">
        <h2 className="text-base font-semibold mb-4">Profile</h2>
        <div className="flex items-start gap-5 mb-6">
          <div className="h-16 w-16 rounded-full bg-accent/20 flex items-center justify-center text-accent text-xl font-bold shrink-0">
            {profile.name[0]}
          </div>
          <div className="flex-1 space-y-4">
            <div>
              <label className="block text-sm text-muted mb-1.5">Name</label>
              <input
                type="text"
                value={profile.name}
                onChange={(e) => setProfile((p) => ({ ...p, name: e.target.value }))}
                className="w-full bg-background border border-border rounded-lg px-3 py-2 text-sm focus:outline-none focus:border-accent"
              />
            </div>
            <div>
              <label className="block text-sm text-muted mb-1.5">Email</label>
              <input
                type="email"
                value={profile.email}
                onChange={(e) => setProfile((p) => ({ ...p, email: e.target.value }))}
                className="w-full bg-background border border-border rounded-lg px-3 py-2 text-sm focus:outline-none focus:border-accent"
              />
            </div>
          </div>
        </div>
        <button
          onClick={() => toast("Profile saved")}
          className="inline-flex items-center gap-2 bg-accent hover:bg-accent-hover text-black font-medium rounded-lg px-4 py-2 text-sm transition-colors"
        >
          <Save className="h-4 w-4" /> Save Changes
        </button>
      </div>

      {/* Team */}
      <div className="bg-card border border-border rounded-xl p-6">
        <h2 className="text-base font-semibold mb-4">Team</h2>
        <div className="space-y-3 mb-5">
          {teamMembers.map((m) => {
            const RoleIcon = m.role === "owner" ? Crown : m.role === "admin" ? Shield : User;
            return (
              <div key={m.id} className="flex items-center gap-3 py-2">
                <div className="h-8 w-8 rounded-full bg-accent/10 flex items-center justify-center text-accent text-sm font-bold shrink-0">
                  {m.name[0]}
                </div>
                <div className="flex-1 min-w-0">
                  <p className="text-sm font-medium">{m.name}</p>
                  <p className="text-xs text-muted">{m.email}</p>
                </div>
                <span className="inline-flex items-center gap-1 text-xs text-muted capitalize">
                  <RoleIcon className="h-3.5 w-3.5" /> {m.role}
                </span>
              </div>
            );
          })}
        </div>
        <div className="flex gap-2">
          <input
            type="email"
            value={inviteEmail}
            onChange={(e) => setInviteEmail(e.target.value)}
            placeholder="colleague@company.com"
            className="flex-1 bg-background border border-border rounded-lg px-3 py-2 text-sm focus:outline-none focus:border-accent"
          />
          <button
            onClick={() => {
              if (inviteEmail.trim()) {
                toast(`Invitation sent to ${inviteEmail}`);
                setInviteEmail("");
              }
            }}
            className="inline-flex items-center gap-2 bg-accent hover:bg-accent-hover text-black font-medium rounded-lg px-4 py-2 text-sm transition-colors"
          >
            <UserPlus className="h-4 w-4" /> Invite
          </button>
        </div>
      </div>

      {/* Billing */}
      <div className="bg-card border border-border rounded-xl p-6">
        <h2 className="text-base font-semibold mb-4">Billing</h2>
        <div className="flex items-center justify-between mb-4">
          <div>
            <p className="text-sm font-medium">Current Plan</p>
            <p className="text-2xl font-bold text-accent mt-1">Free</p>
            <p className="text-xs text-muted mt-1">5 tunnels · 10GB bandwidth/month</p>
          </div>
          <button className="bg-accent hover:bg-accent-hover text-black font-medium rounded-lg px-4 py-2 text-sm transition-colors">
            Upgrade to Pro
          </button>
        </div>
      </div>

      {/* Danger Zone */}
      <div className="bg-card border border-danger/30 rounded-xl p-6">
        <h2 className="text-base font-semibold text-danger mb-2 flex items-center gap-2">
          <AlertTriangle className="h-4.5 w-4.5" /> Danger Zone
        </h2>
        <p className="text-sm text-muted mb-4">
          Permanently delete your account and all associated data. This action cannot be undone.
        </p>
        <button
          onClick={() => toast("Account deletion requires confirmation via email", "error")}
          className="bg-danger/10 hover:bg-danger/20 text-danger font-medium rounded-lg px-4 py-2 text-sm transition-colors"
        >
          Delete Account
        </button>
      </div>
    </div>
  );
}
