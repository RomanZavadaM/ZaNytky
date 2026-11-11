import { describe, expect, it } from "vitest";
import { cutGraph } from "./selectionCut";

const rel = (source: string, target: string) => ({
  source,
  target,
  kind: "k",
  grade: null,
  origin: null,
  status: null,
  evidence: null,
});

describe("cutGraph", () => {
  const graph = {
    nodes: ["1", "2", "3", "4"].map((path) => ({ path, title: path, noteType: "person" })),
    edges: [],
    relations: [rel("1", "2"), rel("2", "3"), rel("3", "4")],
  };

  it("keeps the selected subjects and one step around them", () => {
    const cut = cutGraph(graph, [1], 1);
    expect(cut.nodes.map((n) => n.path)).toEqual(["1", "2"]);
    expect(cut.relations).toHaveLength(1);
  });

  it("goes deeper on request", () => {
    expect(cutGraph(graph, [1], 3).nodes).toHaveLength(4);
  });
});
