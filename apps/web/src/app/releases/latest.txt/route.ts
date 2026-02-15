import { NextResponse } from "next/server";

const REPO = "winterwindgames/subtunnel";

export async function GET() {
  const res = await fetch(
    `https://api.github.com/repos/${REPO}/releases/latest`,
    {
      headers: {
        Authorization: `token ${process.env.GITHUB_TOKEN}`,
        Accept: "application/vnd.github+json",
      },
      next: { revalidate: 300 }, // cache 5 min
    }
  );

  if (!res.ok) {
    return NextResponse.json(
      { error: "Failed to fetch latest release" },
      { status: 502 }
    );
  }

  const data = await res.json();
  return new NextResponse(data.tag_name + "\n", {
    headers: { "Content-Type": "text/plain" },
  });
}
