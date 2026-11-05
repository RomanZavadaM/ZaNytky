import type { KnowledgeGraph } from "./types";

/** How many lines the graph draws: one per pair of nodes, whether it is a typed relation or a plain link. */
export function linkCount(graph: KnowledgeGraph): number {
  const pairs = new Set<string>();
  const add = (source: string, target: string) => {
    if (source === target) return;
    pairs.add(source < target ? `${source}\u0000${target}` : `${target}\u0000${source}`);
  };
  for (const edge of graph.edges) add(edge.source, edge.target);
  for (const relation of graph.relations ?? []) add(relation.source, relation.target);
  return pairs.size;
}
