import { describe, expect, it } from "bun:test";

import { validatePasswordChangeForm } from "./password-change";

describe("validatePasswordChangeForm", () => {
  it("当新密码与确认密码仅首尾空格差异时不应报错", () => {
    const result = validatePasswordChangeForm({
      currentPassword: "old-password",
      newPassword: "  new-password-123  ",
      confirmPassword: "  new-password-123  ",
    });

    expect(result.errors.confirm_password).toBeUndefined();
    expect(result.payload).toEqual({
      current_password: "old-password",
      new_password: "new-password-123",
    });
  });
});
