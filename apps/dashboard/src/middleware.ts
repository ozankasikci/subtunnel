import { auth } from "@/lib/auth";
import { NextResponse } from "next/server";

const WEB_URL = process.env.NEXT_PUBLIC_WEB_URL || "https://www.subtunnel.dev";

export default auth((req) => {
  const { pathname } = req.nextUrl;
  const isLoggedIn = !!req.auth;

  // API routes — return 401 JSON if not authenticated (except auth routes)
  if (pathname.startsWith("/api/")) {
    if (pathname.startsWith("/api/auth/")) {
      return NextResponse.next();
    }
    if (!isLoggedIn) {
      return NextResponse.json({ error: "Unauthorized" }, { status: 401 });
    }
    return NextResponse.next();
  }

  // Dashboard pages — redirect to marketing site login
  if (!isLoggedIn) {
    return NextResponse.redirect(`${WEB_URL}/login`);
  }

  return NextResponse.next();
});

export const config = {
  matcher: ["/((?!_next/static|_next/image|favicon.ico).*)"],
};
