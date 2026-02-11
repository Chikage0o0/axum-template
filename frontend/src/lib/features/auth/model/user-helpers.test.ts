import { describe, expect, it } from "bun:test";
import { buildUserPatchPayload } from "./user-helpers";

describe("buildUserPatchPayload", () => {
  it("should return error when no fields changed", () => {
    const result = buildUserPatchPayload({
      mode: "admin-edit",
      current: {
        username: "alice",
        display_name: "Alice",
        email: "alice@example.com",
        phone: "123",
        avatar_url: "https://example.com/a.png",
        is_active: true,
      },
      draft: {
        username: "alice",
        display_name: "Alice",
        email: "alice@example.com",
        phone: "123",
        avatar_url: "https://example.com/a.png",
        is_active: true,
      },
    });

    expect(result).toEqual({ ok: false, message: "没有可更新的字段" });
  });

  it("should normalize empty optional fields consistently", () => {
    const result = buildUserPatchPayload({
      mode: "self-edit",
      current: {
        display_name: "Alice",
        email: "alice@example.com",
        phone: "123",
        avatar_url: "https://example.com/a.png",
      },
      draft: {
        display_name: "Alice",
        email: "alice@example.com",
        phone: "",
        avatar_url: "",
      },
    });

    expect(result).toEqual({
      ok: true,
      payload: {
        phone: null,
        avatar_url: null,
      },
    });
  });

  it("should keep required field validation consistent", () => {
    const result = buildUserPatchPayload({
      mode: "admin-edit",
      current: {
        username: "alice",
        display_name: "Alice",
        email: "alice@example.com",
        phone: null,
        avatar_url: null,
        is_active: true,
      },
      draft: {
        username: "alice",
        display_name: "",
        email: "invalid-email",
        phone: "",
        avatar_url: "",
        is_active: true,
      },
    });

    expect(result).toEqual({ ok: false, message: "display_name 不能为空" });
  });
});
