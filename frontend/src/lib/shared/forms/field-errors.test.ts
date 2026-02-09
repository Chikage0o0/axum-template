import { describe, expect, it } from "bun:test";
import { z } from "zod";
import {
  detailsToFieldErrors,
  hasFieldError,
  mergeFieldErrors,
  toFieldErrorItems,
  zodErrorToFieldErrors,
} from "./field-errors";

describe("zodErrorToFieldErrors", () => {
  it("should map zod issue path to dotted field key", () => {
    const schema = z.object({
      profile: z.object({ email: z.string().email("邮箱格式错误") }),
    });

    const parsed = schema.safeParse({ profile: { email: "not-email" } });
    if (parsed.success) throw new Error("expected parse failure");

    const errors = zodErrorToFieldErrors(parsed.error);
    expect(errors["profile.email"]).toEqual(["邮箱格式错误"]);
  });

  it("should localize too_small default message", () => {
    const schema = z.object({ username: z.string().min(1) });
    const parsed = schema.safeParse({ username: "" });
    if (parsed.success) throw new Error("expected parse failure");

    const errors = zodErrorToFieldErrors(parsed.error);
    expect(errors.username).toEqual(["不能为空"]);
  });

  it("should localize too_big default message", () => {
    const schema = z.object({ username: z.string().max(2) });
    const parsed = schema.safeParse({ username: "abcd" });
    if (parsed.success) throw new Error("expected parse failure");

    const errors = zodErrorToFieldErrors(parsed.error);
    expect(errors.username).toEqual(["长度不能超过 2 个字符"]);
  });

  it("should localize uuid format message", () => {
    const schema = z.object({ id: z.string().uuid() });
    const parsed = schema.safeParse({ id: "x" });
    if (parsed.success) throw new Error("expected parse failure");

    const errors = zodErrorToFieldErrors(parsed.error);
    expect(errors.id).toEqual(["UUID 格式不正确"]);
  });
});

describe("detailsToFieldErrors", () => {
  it("should map backend details object to field errors", () => {
    const details = {
      display_name: ["display_name 不能为空"],
      email: ["email 格式不合法"],
    };

    const errors = detailsToFieldErrors(details);
    expect(errors.display_name).toEqual(["display_name 不能为空"]);
    expect(errors.email).toEqual(["email 格式不合法"]);
  });
});

describe("toFieldErrorItems", () => {
  it("should merge alias keys in order", () => {
    const errors = {
      "app.check_interval_secs": ["不能小于 10"],
      check_interval_secs: ["必须是数字"],
    };

    const items = toFieldErrorItems(errors, "check_interval_secs", "app.check_interval_secs");
    expect(items).toEqual([{ message: "必须是数字" }, { message: "不能小于 10" }]);
  });
});

describe("hasFieldError", () => {
  it("should report true when any key has errors", () => {
    const errors = { username: ["必填"] };
    expect(hasFieldError(errors, "password", "username")).toBeTrue();
  });
});

describe("mergeFieldErrors", () => {
  it("should merge all error maps", () => {
    const merged = mergeFieldErrors(
      { username: ["必填"] },
      { username: ["过短"], password: ["必填"] },
    );

    expect(merged).toEqual({
      username: ["必填", "过短"],
      password: ["必填"],
    });
  });
});
