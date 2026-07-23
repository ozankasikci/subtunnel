import { NextRequest, NextResponse } from "next/server";

const REPO = "ozankasikci/subtunnel";

export async function GET(
  _req: NextRequest,
  { params }: { params: Promise<{ path: string[] }> }
) {
  const { path } = await params;
  // Expected: /releases/v0.1.0/subtunnel-v0.1.0-aarch64-apple-darwin.tar.gz
  // path = ["v0.1.0", "subtunnel-v0.1.0-aarch64-apple-darwin.tar.gz"]
  if (path.length !== 2) {
    return NextResponse.json({ error: "Invalid path" }, { status: 400 });
  }

  const [tag, filename] = path;
  const downloadUrl = `https://github.com/${REPO}/releases/download/${tag}/${filename}`;
  const token = process.env.GITHUB_TOKEN;

  // Fetch from GitHub with auth to access private repo assets
  const res = await fetch(downloadUrl, {
    headers: {
      ...(token ? { Authorization: `token ${token}` } : {}),
      Accept: "application/octet-stream",
    },
    redirect: "follow",
  });

  if (!res.ok) {
    return NextResponse.json(
      { error: `Asset not found: ${filename}` },
      { status: res.status }
    );
  }

  const data = res.body;
  return new NextResponse(data, {
    headers: {
      "Content-Type": "application/octet-stream",
      "Content-Disposition": `attachment; filename="${filename}"`,
      "Cache-Control": "public, max-age=86400",
    },
  });
}
