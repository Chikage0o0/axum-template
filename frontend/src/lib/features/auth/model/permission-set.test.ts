import { describe, expect, it } from "bun:test";

import { createPermissionSet } from "./permission-set";

describe("createPermissionSet", () => {
  it("can('users:list') 应匹配精确权限", () => {
    const permissions = createPermissionSet(["users:list"]);

    expect(permissions.can("users:list")).toBeTrue();
    expect(permissions.can("users:update")).toBeFalse();
  });

  it("can('users:list') 应匹配命名空间通配符", () => {
    const permissions = createPermissionSet(["users:*"]);

    expect(permissions.can("users:list")).toBeTrue();
    expect(permissions.can("settings:view")).toBeFalse();
  });
});
