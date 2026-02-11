import { describe, expect, it } from "bun:test";

import { buildUserDetailRows, toUserStatusLabel } from "./user-detail";

describe("toUserStatusLabel", () => {
  it("active user should be enabled label", () => {
    expect(toUserStatusLabel(true)).toBe("启用");
  });

  it("inactive user should be disabled label", () => {
    expect(toUserStatusLabel(false)).toBe("禁用");
  });
});

describe("buildUserDetailRows", () => {
  it("should map user fields and keep order", () => {
    const rows = buildUserDetailRows({
      id: "user-1",
      username: "alice",
      email: "alice@example.com",
      phone: "13800000000",
      created_at: "2026-02-11T10:00:00Z",
      updated_at: "2026-02-11T11:00:00Z",
      formatDateTime: (value) => `local:${value}`,
    });

    expect(rows).toEqual([
      { label: "用户ID", value: "user-1" },
      { label: "用户名", value: "alice" },
      { label: "邮箱", value: "alice@example.com" },
      { label: "手机号", value: "13800000000" },
      { label: "创建时间", value: "local:2026-02-11T10:00:00Z" },
      { label: "更新时间", value: "local:2026-02-11T11:00:00Z" },
    ]);
  });

  it("should fallback optional fields with dash", () => {
    const rows = buildUserDetailRows({
      id: "user-2",
      username: null,
      email: "bob@example.com",
      phone: "   ",
      created_at: "2026-02-11T10:00:00Z",
      updated_at: "2026-02-11T11:00:00Z",
    });

    expect(rows[1]).toEqual({ label: "用户名", value: "-" });
    expect(rows[3]).toEqual({ label: "手机号", value: "-" });
  });
});
