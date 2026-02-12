import { getCurrentUser } from "@/lib/auth";

export default async function SettingsPage() {
  const user = await getCurrentUser();

  return (
    <div className="space-y-6">
      <div>
        <h1 className="text-2xl font-bold">Settings</h1>
        <p className="text-muted mt-1">Manage your account settings</p>
      </div>

      <div className="bg-card border border-border rounded-xl p-6 space-y-4">
        <h2 className="text-base font-semibold">Profile</h2>
        <div className="grid grid-cols-1 sm:grid-cols-2 gap-4">
          <div>
            <label className="block text-sm text-muted mb-1.5">Email</label>
            <div className="bg-background border border-border rounded-lg px-3 py-2 text-sm">{user!.email}</div>
          </div>
          <div>
            <label className="block text-sm text-muted mb-1.5">Name</label>
            <div className="bg-background border border-border rounded-lg px-3 py-2 text-sm">{user!.name || "—"}</div>
          </div>
          <div>
            <label className="block text-sm text-muted mb-1.5">Plan</label>
            <div className="bg-background border border-border rounded-lg px-3 py-2 text-sm">{user!.plan}</div>
          </div>
          <div>
            <label className="block text-sm text-muted mb-1.5">Member Since</label>
            <div className="bg-background border border-border rounded-lg px-3 py-2 text-sm">
              {new Date(user!.createdAt).toLocaleDateString("en-US", { month: "long", day: "numeric", year: "numeric" })}
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}
