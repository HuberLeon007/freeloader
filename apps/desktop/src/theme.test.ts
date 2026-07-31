// SPDX-License-Identifier: GPL-3.0-or-later
import { describe, expect, it } from "vitest";
import { resolveTheme } from "./theme";

describe("theme resolution", () => {
  it("keeps an explicit light choice", () => expect(resolveTheme("light")).toBe("light"));
  it("keeps an explicit dark choice", () => expect(resolveTheme("dark")).toBe("dark"));
});
