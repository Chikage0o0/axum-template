export type AuthRole = "admin" | "user";

function decodeJwtPayload(token: string): Record<string, unknown> | null {
  const parts = token.split(".");
  if (parts.length < 2) return null;

  let base64 = parts[1].replace(/-/g, "+").replace(/_/g, "/");
  while (base64.length % 4 !== 0) {
    base64 += "=";
  }

  try {
    if (typeof atob === "function") {
      return JSON.parse(atob(base64)) as Record<string, unknown>;
    }
  } catch {
    return null;
  }

  return null;
}

export function readRoleFromToken(token: string | null): AuthRole {
  if (!token) return "user";

  const payload = decodeJwtPayload(token);
  const role = payload?.role;
  if (role === "admin") {
    return "admin";
  }

  return "user";
}
