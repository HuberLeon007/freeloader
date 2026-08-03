import { describe, expect, it } from "vitest";
import { proposeBatchName } from "../src/features/composer/batch-name";
import { parsePaste, validCandidates } from "../src/features/composer/paste-parser";

describe("paste parser", () => {
  it("deduplicates valid links and reports invalid lines", () => {
    const parsed = parsePaste("https://example.com/a\nnot-a-url\nhttps://example.com/a");
    expect(validCandidates(parsed.map((item) => item.original).join("\n"))).toHaveLength(1);
    expect(parsed[1]?.reason).toBe("invalid URL");
    expect(parsed[2]?.reason).toBe("duplicate");
  });
  it("proposes the dominant host", () => expect(proposeBatchName(parsePaste("https://example.com/a\nhttps://example.com/b"))).toBe("example.com downloads"));
});
