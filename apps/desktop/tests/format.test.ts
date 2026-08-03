import { describe, expect, it } from "vitest";
import { formatBytes, formatEta, formatSpeed, isKnownNumeric } from "../src/lib/format";

describe("format helpers", () => {
  it("renders unknown eta as an em dash", () => expect(formatEta(null)).toBe("—"));
  it("formats bytes consistently", () => expect(formatBytes(1536)).toContain("1.5"));
  it("formats speed", () => expect(formatSpeed(1024)).toBe("1.00 KB/s"));
  it("recognizes tabular values", () => expect(isKnownNumeric("42")).toBe(true));
});
