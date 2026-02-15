import { describe, expect, it } from "bun:test";

import { toPermissionOptions } from "./permission-node-catalog";

describe("toPermissionOptions", () => {
  it("应把权限字典映射为可配置选项", () => {
    const options = toPermissionOptions({
      version: "2026-02-15",
      items: [
        {
          code: "users:list",
          name: "List Users",
          description: "查看用户列表",
          module: "users",
        },
      ],
    });

    expect(options).toHaveLength(1);
    expect(options[0]?.value).toBe("users:list");
    expect(options[0]?.label).toContain("List Users");
    expect(options[0]?.module).toBe("users");
    expect(options[0]?.description).toBe("查看用户列表");
  });
});
