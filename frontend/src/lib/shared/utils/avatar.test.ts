import { describe, expect, it } from "bun:test";
import { buildAvatarSeed, normalizeAvatarBeamSize } from "./avatar";

describe("buildAvatarSeed", () => {
  it("优先使用 email 作为 seed", () => {
    expect(
      buildAvatarSeed({
        email: "  alice@example.com  ",
        displayName: "Alice",
        id: "user-1",
      }),
    ).toBe("alice@example.com");
  });

  it("email 为空时回退 displayName", () => {
    expect(
      buildAvatarSeed({
        email: "  ",
        displayName: " Alice ",
        id: "user-2",
      }),
    ).toBe("Alice");
  });

  it("email 与 displayName 都为空时回退 id", () => {
    expect(
      buildAvatarSeed({
        email: "",
        displayName: "",
        id: "user-3",
      }),
    ).toBe("user-3");
  });

  it("全部为空时使用默认值", () => {
    expect(
      buildAvatarSeed({
        email: "",
        displayName: "",
        id: "",
      }),
    ).toBe("unknown-user");
  });
});

describe("normalizeAvatarBeamSize", () => {
  it("为空时使用默认值 32", () => {
    expect(normalizeAvatarBeamSize()).toBe(32);
  });

  it("向下取整并限制最小值", () => {
    expect(normalizeAvatarBeamSize(15.9)).toBe(16);
  });

  it("限制最大值", () => {
    expect(normalizeAvatarBeamSize(300)).toBe(128);
  });
});
