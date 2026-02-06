import { afterEach, beforeEach, describe, expect, it } from "bun:test";
import { get } from "svelte/store";

import { auth } from "../stores/auth";
import { ApiError, apiClient } from "./mutator";

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
const originalLocalStorage = (globalThis as { localStorage?: MemoryStorage })
  .localStorage;

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
        body: JSON.stringify({ current_password: "bad", new_password: "new-password-123" }),
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
});
