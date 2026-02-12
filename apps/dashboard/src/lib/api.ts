export class ApiError extends Error {
  constructor(public status: number, message: string) {
    super(message);
  }
}

async function request<T>(url: string, options?: RequestInit): Promise<T> {
  const res = await fetch(url, {
    headers: { "Content-Type": "application/json" },
    ...options,
  });

  if (res.status === 401) {
    window.location.href = "/login";
    throw new ApiError(401, "Unauthorized");
  }

  const data = await res.json();
  if (!res.ok) throw new ApiError(res.status, data.error || "Request failed");
  return data;
}

export const api = {
  tunnels: {
    list: () =>
      request<{ id: string; subdomain: string; localPort: number; status: string; createdAt: string; lastActive: string }[]>("/api/tunnels"),
  },
  apiKeys: {
    list: () =>
      request<{ id: string; name: string; prefix: string; createdAt: string; lastUsed: string | null }[]>("/api/api-keys"),
    create: (name: string) =>
      request<{ id: string; name: string; prefix: string; rawKey: string; createdAt: string }>("/api/api-keys", {
        method: "POST",
        body: JSON.stringify({ name }),
      }),
    revoke: (id: string) =>
      request<{ ok: true }>(`/api/api-keys/${id}`, { method: "DELETE" }),
  },
};
