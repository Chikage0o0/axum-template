import { auth } from "$lib/stores/auth";

const API_BASE = "/api/v1";

export type ApiErrorBody = {
  code?: number;
  message?: string;
  request_id?: string;
  details?: unknown;
};

export class ApiError extends Error {
  status: number;
  body?: ApiErrorBody;

  constructor(message: string, status: number, body?: ApiErrorBody) {
    super(message);
    this.name = "ApiError";
    this.status = status;
    this.body = body;
  }
}

function joinUrl(base: string, path: string): string {
  if (!path) return base;
  if (path.startsWith("/")) return `${base}${path}`;
  return `${base}/${path}`;
}

async function readJsonSafe(res: Response): Promise<unknown | null> {
  const ct = res.headers.get("content-type") ?? "";
  if (!ct.includes("application/json")) return null;
  try {
    return await res.json();
  } catch {
    return null;
  }
}

export async function apiFetchJson<T>(
  path: string,
  init: RequestInit = {},
): Promise<T> {
  const url = path.startsWith("/api/") ? path : joinUrl(API_BASE, path);

  const tokenAtStart = auth.readTokenFromStorage();
  const headers = new Headers(init.headers);
  headers.set("accept", "application/json");

  const hasBody = typeof init.body !== "undefined" && init.body !== null;
  if (hasBody && !headers.has("content-type")) {
    headers.set("content-type", "application/json");
  }
  if (tokenAtStart) {
    headers.set("authorization", `Bearer ${tokenAtStart}`);
  }

  const res = await fetch(url, { ...init, headers });
  if (!res.ok) {
    const body = (await readJsonSafe(res)) as ApiErrorBody | null;
    const message = body?.message || `HTTP ${res.status}`;

    if (res.status === 401) {
      const currentToken = auth.readTokenFromStorage();
      if (tokenAtStart && currentToken === tokenAtStart) {
        auth.logout({ reason: "expired" });
      }
    }

    throw new ApiError(message, res.status, body ?? undefined);
  }

  if (res.status === 204) return undefined as T;
  const json = (await readJsonSafe(res)) as unknown | null;
  if (json === null) {
    throw new ApiError("响应不是 JSON", res.status);
  }
  return json as T;
}
