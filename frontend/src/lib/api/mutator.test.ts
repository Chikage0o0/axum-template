import { afterEach, beforeEach, describe, expect, it } from "bun:test";
import { get } from "svelte/store";

import { auth } from "$lib/features/auth/state/auth";
import { ApiError, apiClient, refreshAccessToken } from "./mutator";

type MemoryStorage = {
  getItem: (key: string) => string | null;
  setItem: (key: string, value: string) => void;
  removeItem: (key: string) => void;
  clear: () => void;
};

function createMemoryStorage(): MemoryStorage {
  const data = new Map<string, string>();
  return {
    getItem(key: string) {
      return data.has(key) ? data.get(key)! : null;
    },
    setItem(key: string, value: string) {
      data.set(key, value);
    },
    removeItem(key: string) {
      data.delete(key);
    },
    clear() {
      data.clear();
    },
  };
}

const originalFetch = globalThis.fetch;
const originalLocalStorage = (globalThis as { localStorage?: MemoryStorage }).localStorage;

describe("apiClient", () => {
  beforeEach(() => {
    Object.defineProperty(globalThis, "localStorage", {
      value: createMemoryStorage(),
      configurable: true,
      writable: true,
    });
    auth.logout({ reason: "manual" });
  });

  afterEach(() => {
    globalThis.fetch = originalFetch;
    Object.defineProperty(globalThis, "localStorage", {
      value: originalLocalStorage,
      configurable: true,
      writable: true,
    });
    auth.logout({ reason: "manual" });
  });

  it("当前密码错误的 401 不应导致自动登出", async () => {
    auth.login("token-1");

    globalThis.fetch = async () =>
      new Response(
        JSON.stringify({
          code: 1002,
          message: "身份验证失败: Token 无效或已过期",
          request_id: "req_1",
        }),
        {
          status: 401,
          headers: { "content-type": "application/json" },
        },
      );

    await expect(
      apiClient<void>("/api/v1/security/password", {
        method: "PATCH",
        body: JSON.stringify({
          current_password: "bad",
          new_password: "new-password-123",
        }),
      }),
    ).rejects.toBeInstanceOf(ApiError);

    expect(get(auth).isAuthenticated).toBeTrue();
    expect(get(auth).token).toBe("token-1");
  });

  it("令牌失效的 401 仍然应自动登出", async () => {
    auth.login("token-2");

    globalThis.fetch = async () =>
      new Response(
        JSON.stringify({
          code: 1001,
          message: "身份验证失败: 凭证无效",
          request_id: "req_2",
        }),
        {
          status: 401,
          headers: { "content-type": "application/json" },
        },
      );

    await expect(
      apiClient<void>("/api/v1/settings", {
        method: "PATCH",
        body: JSON.stringify({}),
      }),
    ).rejects.toBeInstanceOf(ApiError);

    expect(get(auth).isAuthenticated).toBeFalse();
    expect(get(auth).token).toBeNull();
  });

  it("令牌失效后应尝试刷新并自动重试原请求", async () => {
    auth.login("token-old");

    let calls = 0;
    globalThis.fetch = async (input: RequestInfo | URL) => {
      const url = typeof input === "string" ? input : input.toString();
      calls += 1;

      if (url === "/api/v1/settings") {
        if (calls === 1) {
          return new Response(
            JSON.stringify({
              code: 1001,
              message: "身份验证失败: Token 无效或已过期",
              request_id: "req_3",
            }),
            {
              status: 401,
              headers: { "content-type": "application/json" },
            },
          );
        }

        return new Response(JSON.stringify({ ok: true }), {
          status: 200,
          headers: { "content-type": "application/json" },
        });
      }

      if (url === "/api/v1/sessions/refresh") {
        return new Response(
          JSON.stringify({
            token: "token-new",
            expires_in: 900,
          }),
          {
            status: 200,
            headers: { "content-type": "application/json" },
          },
        );
      }

      return new Response(null, { status: 404 });
    };

    const data = await apiClient<{ ok: boolean }>("/api/v1/settings", {
      method: "GET",
    });

    expect(data.ok).toBeTrue();
    expect(get(auth).isAuthenticated).toBeTrue();
    expect(get(auth).token).toBe("token-new");
  });

  it("已登出且无 token 时，后续 401 不应再次触发 refresh", async () => {
    auth.login("token-old");

    let refreshCalls = 0;

    globalThis.fetch = async (input: RequestInfo | URL) => {
      const url = typeof input === "string" ? input : input.toString();

      if (url === "/api/v1/settings") {
        return new Response(
          JSON.stringify({
            code: 1001,
            message: "身份验证失败: Token 无效或已过期",
            request_id: "req_5",
          }),
          {
            status: 401,
            headers: { "content-type": "application/json" },
          },
        );
      }

      if (url === "/api/v1/sessions/refresh") {
        refreshCalls += 1;
        return new Response(
          JSON.stringify({
            code: 1001,
            message: "身份验证失败: Token 无效或已过期",
            request_id: "req_refresh",
          }),
          {
            status: 401,
            headers: { "content-type": "application/json" },
          },
        );
      }

      return new Response(null, { status: 404 });
    };

    await expect(apiClient<void>("/api/v1/settings", { method: "GET" })).rejects.toBeInstanceOf(
      ApiError,
    );
    expect(get(auth).isAuthenticated).toBeFalse();

    await expect(apiClient<void>("/api/v1/settings", { method: "GET" })).rejects.toBeInstanceOf(
      ApiError,
    );

    expect(refreshCalls).toBe(1);
  });

  it("并发刷新时应复用同一个 refresh 请求", async () => {
    auth.login("token-old");

    let settingsCalls = 0;
    let refreshCalls = 0;

    globalThis.fetch = async (input: RequestInfo | URL) => {
      const url = typeof input === "string" ? input : input.toString();

      if (url === "/api/v1/settings") {
        settingsCalls += 1;
        if (settingsCalls === 1) {
          return new Response(
            JSON.stringify({
              code: 1001,
              message: "身份验证失败: Token 无效或已过期",
              request_id: "req_4",
            }),
            {
              status: 401,
              headers: { "content-type": "application/json" },
            },
          );
        }

        return new Response(JSON.stringify({ ok: true }), {
          status: 200,
          headers: { "content-type": "application/json" },
        });
      }

      if (url === "/api/v1/sessions/refresh") {
        refreshCalls += 1;
        return new Response(
          JSON.stringify({
            token: `token-new-${refreshCalls}`,
            expires_in: 900,
          }),
          {
            status: 200,
            headers: { "content-type": "application/json" },
          },
        );
      }

      return new Response(null, { status: 404 });
    };

    const [settings, refreshedToken] = await Promise.all([
      apiClient<{ ok: boolean }>("/api/v1/settings", { method: "GET" }),
      refreshAccessToken(),
    ]);

    expect(settings.ok).toBeTrue();
    expect(refreshedToken?.startsWith("token-new-")).toBeTrue();
    expect(refreshCalls).toBe(1);
  });
});
