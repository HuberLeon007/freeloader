// SPDX-License-Identifier: GPL-3.0-or-later
import { describe, expect, it } from "vitest";

describe("test harness", () => {
  it("runs in jsdom", () => {
    expect(document.body).toBeDefined();
  });
});
