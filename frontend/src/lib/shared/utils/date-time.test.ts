import { describe, expect, it } from "bun:test";

import { formatBrowserLocalDateTime } from "./date-time";

describe("formatBrowserLocalDateTime", () => {
  it("should format datetime with injected formatter", () => {
    const formatted = formatBrowserLocalDateTime("2026-02-11T10:00:00Z", {
      format(date) {
        return `formatted:${date.toISOString()}`;
      },
    });

    expect(formatted).toBe("formatted:2026-02-11T10:00:00.000Z");
  });

  it("should fallback to dash for invalid datetime", () => {
    expect(formatBrowserLocalDateTime("not-a-date")).toBe("-");
  });
});
