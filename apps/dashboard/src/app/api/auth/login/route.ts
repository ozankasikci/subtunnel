import { NextRequest, NextResponse } from "next/server";
import bcrypt from "bcryptjs";
import { prisma } from "@/lib/db";
import { encode } from "next-auth/jwt";

const WEB_URL = process.env.NEXT_PUBLIC_WEB_URL || "https://www.subtunnel.dev";
const COOKIE_DOMAIN = process.env.COOKIE_DOMAIN ?? ".subtunnel.dev";

function corsHeaders() {
  return {
    "Access-Control-Allow-Origin": WEB_URL,
    "Access-Control-Allow-Methods": "POST, OPTIONS",
    "Access-Control-Allow-Headers": "Content-Type",
    "Access-Control-Allow-Credentials": "true",
  };
}

export async function OPTIONS() {
  return new NextResponse(null, { status: 204, headers: corsHeaders() });
}

export async function POST(req: NextRequest) {
  try {
    const { email, password } = await req.json();

    if (!email || !password) {
      return NextResponse.json(
        { error: "Email and password required" },
        { status: 400, headers: corsHeaders() }
      );
    }

    const user = await prisma.user.findUnique({ where: { email } });
    if (!user || !user.passwordHash) {
      return NextResponse.json(
        { error: "Invalid email or password" },
        { status: 401, headers: corsHeaders() }
      );
    }

    const valid = await bcrypt.compare(password, user.passwordHash);
    if (!valid) {
      return NextResponse.json(
        { error: "Invalid email or password" },
        { status: 401, headers: corsHeaders() }
      );
    }

    // Create a JWT matching next-auth format
    const token = await encode({
      token: {
        id: user.id,
        email: user.email,
        name: user.name,
        picture: user.image,
        sub: user.id,
      },
      secret: process.env.AUTH_SECRET!,
      salt: "authjs.session-token",
    });

    const response = NextResponse.json(
      { ok: true },
      { headers: corsHeaders() }
    );

    const cookieOptions: {
      httpOnly: boolean;
      secure: boolean;
      sameSite: "lax" | "none";
      path: string;
      maxAge: number;
      domain?: string;
    } = {
      httpOnly: true,
      secure: true,
      sameSite: COOKIE_DOMAIN ? "none" : "lax",
      path: "/",
      maxAge: 30 * 24 * 60 * 60, // 30 days
    };

    if (COOKIE_DOMAIN) {
      cookieOptions.domain = COOKIE_DOMAIN;
    }

    response.cookies.set("authjs.session-token", token, cookieOptions);

    return response;
  } catch (e) {
    console.error("Login error:", e);
    return NextResponse.json(
      { error: "Internal server error" },
      { status: 500, headers: corsHeaders() }
    );
  }
}
