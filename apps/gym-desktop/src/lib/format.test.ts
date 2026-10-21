import { describe, expect, it } from "vitest";

import { formatBytes, shortenChecksum } from "./format";

describe("formatBytes", () => {
  it("formats small and large values", () => {
    expect(formatBytes(512)).toBe("512 B");
    expect(formatBytes(2048)).toBe("2.0 KB");
    expect(formatBytes(5 * 1024 * 1024)).toBe("5.0 MB");
  });

  it("rejects invalid values", () => {
    expect(formatBytes(Number.NaN)).toBe("—");
    expect(formatBytes(-1)).toBe("—");
  });
});

describe("shortenChecksum", () => {
  it("keeps short values intact", () => {
    expect(shortenChecksum("abc")).toBe("abc");
  });

  it("truncates long values", () => {
    expect(shortenChecksum("0123456789abcdef", 8)).toBe("01234567…");
  });
});
