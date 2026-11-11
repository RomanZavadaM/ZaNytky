import type { KnowledgeGraph } from "./graph/types";

/** The selected subjects and what is linked to them within `depth` steps. */
export function cutGraph(graph: KnowledgeGraph, ids: number[], depth: number): KnowledgeGraph {
  const reached = new Set(ids.map(String));
  let frontier = new Set(reached);
  for (let step = 0; step < Math.max(1, depth); step += 1) {
    const next = new Set<string>();
    for (const r of graph.relations) {
      for (const [from, to] of [
        [r.source, r.target],
        [r.target, r.source],
      ]) {
        if (frontier.has(from) && !reached.has(to)) {
          reached.add(to);
          next.add(to);
        }
      }
    }
    frontier = next;
  }
  return {
    nodes: graph.nodes.filter((n) => reached.has(n.path)),
    edges: graph.edges.filter((e) => reached.has(e.source) && reached.has(e.target)),
    relations: graph.relations.filter((r) => reached.has(r.source) && reached.has(r.target)),
  };
}
