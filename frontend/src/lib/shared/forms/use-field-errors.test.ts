import { describe, expect, it } from "bun:test";
import { useFieldErrors } from "./use-field-errors.svelte";

(globalThis as { $state?: <T>(value: T) => T }).$state = <T>(value: T) => value;

describe("useFieldErrors", () => {
  it("should return invalid=false when no errors", () => {
    const f = useFieldErrors<"email" | "password">();
    expect(f.invalid("email")).toBeFalse();
  });

  it("should merge api details and local errors", () => {
    const f = useFieldErrors<"email" | "password">();
    f.setErrors({ email: ["格式错误"] });
    f.mergeApiDetails({ password: ["长度不足"] });

    expect(f.invalid("email", "password")).toBeTrue();
    expect(f.items("email", "password")).toEqual([
      { message: "格式错误" },
      { message: "长度不足" },
    ]);
  });
});
