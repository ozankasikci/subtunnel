import { PrismaClient } from "@prisma/client";
import bcrypt from "bcryptjs";
import crypto from "crypto";

const prisma = new PrismaClient();

async function main() {
  // Create test user
  const passwordHash = await bcrypt.hash("password123", 12);
  const user = await prisma.user.upsert({
    where: { email: "test@subtunnel.dev" },
    update: {},
    create: {
      email: "test@subtunnel.dev",
      passwordHash,
      name: "Test User",
      plan: "PRO",
    },
  });

  console.log("Created user:", user.email);

  // Create tunnels
  const tunnelsData = [
    { subdomain: "api-dev", localPort: 3000, status: "online" },
    { subdomain: "webhook-test", localPort: 8080, status: "online" },
    { subdomain: "staging-app", localPort: 4200, status: "offline" },
  ];

  for (const t of tunnelsData) {
    await prisma.tunnel.upsert({
      where: { subdomain: t.subdomain },
      update: {},
      create: { ...t, userId: user.id },
    });
  }

  console.log("Created", tunnelsData.length, "tunnels");

  // Create API keys
  const keysData = ["CI/CD Pipeline", "Local Development"];
  for (const name of keysData) {
    const rawKey = `st_live_${crypto.randomBytes(24).toString("base64url")}`;
    const prefix = rawKey.slice(0, 12) + "..." + rawKey.slice(-4);
    const keyHash = await bcrypt.hash(rawKey, 10);

    await prisma.apiKey.create({
      data: { userId: user.id, name, keyHash, prefix },
    });
  }

  console.log("Created", keysData.length, "API keys");
  console.log("\nTest credentials: test@subtunnel.dev / password123");
}

main()
  .catch(console.error)
  .finally(() => prisma.$disconnect());
