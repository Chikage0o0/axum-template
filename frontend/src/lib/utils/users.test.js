import { describe, expect, it } from "bun:test";

import {
  buildCurrentUserPatchPayload,
  buildPatchUserPayload,
  toAuthUser,
} from "./user-helpers";

describe("buildPatchUserPayload", () => {
  it("should reject when no field changed", () => {
    const result = buildPatchUserPayload(
      {
        username: "alice",
        display_name: "Alice",
        email: "alice@example.com",
        phone: "123",
        avatar_url: "https://example.com/a.png",
      },
      {
        username: " alice ",
        display_name: " Alice ",
        email: "alice@example.com",
        phone: "123",
        avatar_url: "https://example.com/a.png",
      },
    );

    expect(result.ok).toBeFalse();
  });

  it("should return changed fields only", () => {
    const result = buildPatchUserPayload(
      {
        username: "alice",
        display_name: "Alice",
        email: "alice@example.com",
        phone: null,
        avatar_url: null,
      },
      {
        username: "alice2",
        display_name: "Alice Chen",
        email: "alice2@example.com",
        phone: "",
        avatar_url: "",
      },
    );

    expect(result).toEqual({
      ok: true,
      payload: {
        username: "alice2",
        display_name: "Alice Chen",
        email: "alice2@example.com",
      },
    });
  });

  it("should reject invalid email", () => {
    const result = buildPatchUserPayload(
      {
        username: "alice",
        display_name: "Alice",
        email: "alice@example.com",
        phone: null,
        avatar_url: null,
      },
      {
        username: "alice",
        display_name: "Alice",
        email: "not-an-email",
        phone: "",
        avatar_url: "",
      },
    );

    expect(result.ok).toBeFalse();
  });

  it("should only return supported editable fields", () => {
    const result = buildPatchUserPayload(
      {
        username: "alice",
        display_name: "Alice",
        email: "alice@example.com",
        phone: null,
        avatar_url: null,
      },
      {
        username: "alice",
        display_name: "Alice",
        email: "alice@example.com",
        phone: "123",
        avatar_url: "",
      },
    );

    expect(result).toEqual({
      ok: true,
      payload: {
        phone: "123",
      },
    });
  });
});

describe("buildCurrentUserPatchPayload", () => {
  it("should reject when no field changed", () => {
    const result = buildCurrentUserPatchPayload(
      {
        display_name: "Alice",
        email: "alice@example.com",
        phone: "123",
        avatar_url: "https://example.com/a.png",
      },
      {
        display_name: " Alice ",
        email: "alice@example.com",
        phone: "123",
        avatar_url: "https://example.com/a.png",
      },
    );

    expect(result.ok).toBeFalse();
  });

  it("should only include changed current-user fields", () => {
    const result = buildCurrentUserPatchPayload(
      {
        display_name: "Alice",
        email: "alice@example.com",
        phone: null,
        avatar_url: null,
      },
      {
        display_name: "Alice Chen",
        email: "alice@example.com",
        phone: "",
        avatar_url: "https://example.com/new.png",
      },
    );

    expect(result).toEqual({
      ok: true,
      payload: {
        display_name: "Alice Chen",
        avatar_url: "https://example.com/new.png",
      },
    });
  });
});

describe("toAuthUser", () => {
  it("should map user fields to AuthUser", () => {
    const mapped = toAuthUser({
      id: "11111111-1111-1111-1111-111111111111",
      username: "alice",
      display_name: "Alice",
      email: "alice@example.com",
      phone: null,
      avatar_url: null,
      is_active: true,
      metadata: {},
      identities: [],
      created_at: "2026-02-06T09:00:00Z",
      updated_at: "2026-02-06T09:00:00Z",
    });

    expect(mapped).toEqual({
      sub: "11111111-1111-1111-1111-111111111111",
      displayName: "Alice",
      email: "alice@example.com",
    });
  });

  it("should return null when required fields are invalid", () => {
    const mapped = toAuthUser({
      id: "",
      username: "alice",
      display_name: "",
      email: "",
      phone: null,
      avatar_url: null,
      is_active: true,
      metadata: {},
      identities: [],
      created_at: "2026-02-06T09:00:00Z",
      updated_at: "2026-02-06T09:00:00Z",
    });

    expect(mapped).toBeNull();
  });
});
