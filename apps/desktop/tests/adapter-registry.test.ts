import { describe, expect, it } from "vitest";
import { AdapterRegistry } from "../src/features/adapters/adapter-registry";

describe("adapter registry", () => {
  it("uses direct HTTP as the fallback", async () => expect((await new AdapterRegistry().resolve("https://example.com/file")).at(0)?.adapterId).toBe("direct-http"));
  it("records the fuckingfast adapter", async () => expect((await new AdapterRegistry().resolve("https://fuckingfast.co/file")).at(0)?.adapterId).toBe("fuckingfast"));
});
