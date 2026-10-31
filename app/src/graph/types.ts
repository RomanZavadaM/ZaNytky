// Copied from MD Notes (app/src); ZaNytky modules stay independent of it.
export interface GraphNode {
  path: string;
  title: string;
  noteType: string | null;
}

export interface GraphEdge {
  source: string;
  target: string;
}

export interface GraphRelation {
  source: string;
  target: string;
  kind: string;
  grade: string | null;
  origin: string | null;
  status: string | null;
  evidence: string | null;
}

export interface KnowledgeGraph {
  nodes: GraphNode[];
  edges: GraphEdge[];
  relations: GraphRelation[];
}
