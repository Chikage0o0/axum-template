import { auth } from "$lib/features/auth/state/auth";

export type ApiErrorBody = {
  code?: number;
  message?: string;
  request_id?: string;
  details?: unknown;
};

type SessionRefreshResponse = {
  token: string;
  expires_in: number;
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

const TOKEN_INVALID_ERROR_CODE = 1001;
const SESSION_REFRESH_PATH = "/api/v1/sessions/refresh";

let refreshingTokenPromise: Promise<string | null> | null = null;

function shouldAutoLogoutOnUnauthorized(body: ApiErrorBody | null): boolean {
  if (typeof body?.code !== "number") {
    return true;
  }

  return body.code === TOKEN_INVALID_ERROR_CODE;
}

function isRefreshEndpoint(url: string): boolean {
  return url.startsWith(SESSION_REFRESH_PATH);
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

function buildHeaders(init: RequestInit, token: string | null): Headers {
  const headers = new Headers(init.headers);
  headers.set("accept", "application/json");

  const hasBody = typeof init.body !== "undefined" && init.body !== null;
  if (hasBody && !headers.has("content-type")) {
    headers.set("content-type", "application/json");
  }
  if (token) {
    headers.set("authorization", `Bearer ${token}`);
  }
  return headers;
}

export async function refreshAccessToken(): Promise<string | null> {
  if (refreshingTokenPromise) {
    return refreshingTokenPromise;
  }

  refreshingTokenPromise = (async () => {
    const refreshRes = await fetch(SESSION_REFRESH_PATH, {
      method: "POST",
      headers: { accept: "application/json" },
    });

    if (!refreshRes.ok) {
      return null;
    }

    const refreshBody = (await readJsonSafe(refreshRes)) as SessionRefreshResponse | null;
    if (!refreshBody || typeof refreshBody.token !== "string" || !refreshBody.token.trim()) {
      return null;
    }

    auth.login(refreshBody.token);
    return refreshBody.token;
  })()
    .catch(() => null)
    .finally(() => {
      refreshingTokenPromise = null;
    });

  return refreshingTokenPromise;
}

export async function apiClient<T>(url: string, init: RequestInit = {}): Promise<T> {
  const tokenAtStart = auth.readTokenFromStorage();
  const headers = buildHeaders(init, tokenAtStart);

  const res = await fetch(url, { ...init, headers });
  if (!res.ok) {
    const body = (await readJsonSafe(res)) as ApiErrorBody | null;
    const message = body?.message || `HTTP ${res.status}`;

    if (res.status === 401 && shouldAutoLogoutOnUnauthorized(body) && !isRefreshEndpoint(url)) {
      if (tokenAtStart) {
        const refreshedToken = await refreshAccessToken();
        if (refreshedToken) {
          const retryRes = await fetch(url, {
            ...init,
            headers: buildHeaders(init, refreshedToken),
          });
          if (retryRes.ok) {
            if (retryRes.status === 204) return undefined as T;
            const retryJson = (await readJsonSafe(retryRes)) as unknown | null;
            if (retryJson === null) {
              throw new ApiError("响应不是 JSON", retryRes.status);
            }
            return retryJson as T;
          }
        }

        const currentToken = auth.readTokenFromStorage();
        if (currentToken === tokenAtStart) {
          auth.logout({ reason: "expired" });
        }
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
