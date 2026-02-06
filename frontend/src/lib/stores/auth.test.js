import { beforeEach, describe, expect, it } from "bun:test";
import { get } from "svelte/store";

import { auth } from "./auth";

function encodeBase64Url(input) {
  return Buffer.from(input, "utf8")
    .toString("base64")
    .replace(/\+/g, "-")
    .replace(/\//g, "_")
    .replace(/=+$/g, "");
}

function buildToken(payload) {
  const header = encodeBase64Url(JSON.stringify({ alg: "HS256", typ: "JWT" }));
  const body = encodeBase64Url(JSON.stringify(payload));
  return `${header}.${body}.signature`;
}

describe("auth", () => {
  beforeEach(() => {
    auth.logout({ reason: "manual" });
  });

  it("should not decode user profile from jwt on login", () => {
    const token = buildToken({
      sub: "11111111-1111-1111-1111-111111111111",
      display_name: "Alice",
      email: "alice@example.com",
    });

    auth.login(token);

    expect(get(auth).isAuthenticated).toBeTrue();
    expect(get(auth).token).toBe(token);
    expect(get(auth).user).toBeNull();
  });
});
