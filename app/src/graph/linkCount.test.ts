import { describe, expect, it } from "vitest";
import { linkCount } from "./linkCount";

const relation = (source: string, target: string) => ({
  source,
  target,
  kind: "owns",
  grade: null,
  origin: null,
  status: null,
  evidence: null,
});

describe("linkCount", () => {
  it("counts a typed relation as a link, once per pair", () => {
    const graph = {
      nodes: [],
      edges: [],
      relations: [relation("1", "2"), relation("2", "1"), relation("2", "3")],
    };
    expect(linkCount(graph)).toBe(2);
  });

  it("does not count a plain link twice when a relation joins the same pair", () => {
    const graph = { nodes: [], edges: [{ source: "1", target: "2" }], relations: [relation("1", "2")] };
    expect(linkCount(graph)).toBe(1);
  });

  it("is zero for an empty graph", () => {
    expect(linkCount({ nodes: [], edges: [], relations: [] })).toBe(0);
  });
});
