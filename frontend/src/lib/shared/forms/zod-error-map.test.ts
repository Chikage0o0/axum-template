import { describe, expect, it } from "bun:test";
import { z } from "zod";
import { setupZodErrorMap } from "./zod-error-map";

describe("setupZodErrorMap", () => {
  it("should localize default zod message globally", () => {
    setupZodErrorMap();

    const parsed = z.object({ username: z.string().min(1) }).safeParse({ username: "" });
    if (parsed.success) throw new Error("expected parse failure");

    expect(parsed.error.issues[0]?.message).toBe("不能为空");
  });
});
