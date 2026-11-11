import { describe, expect, it } from "vitest";
import { graphReloadKey } from "./graphReload";

describe("graphReloadKey", () => {
  it("changes when names are switched on", () => {
    expect(graphReloadKey(0, undefined)).not.toBe(graphReloadKey(0, "key.txt"));
  });
  it("changes when the key file changes", () => {
    expect(graphReloadKey(0, "a.txt")).not.toBe(graphReloadKey(0, "b.txt"));
  });
  it("changes on refresh and is stable otherwise", () => {
    expect(graphReloadKey(1, "a.txt")).not.toBe(graphReloadKey(0, "a.txt"));
    expect(graphReloadKey(2, "a.txt")).toBe(graphReloadKey(2, "a.txt"));
  });
});
