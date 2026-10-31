// Copied from MD Notes (app/src); ZaNytky modules stay independent of it.
import { useEffect, useMemo, useRef, useState } from "react";
import Graph from "graphology";
import Sigma from "sigma";
import type { KnowledgeGraph } from "./types";
import { DEFAULT_FORCES, LiveLayout, type Forces } from "./liveLayout";
import { attachOverlay, type EdgeHit } from "./overlay";
import { chipColors, edgeStyle, gradeDigit, glyphFor, isDarkBackground, nodeSize, stateColor, stateOf, typeColor, type NodeState } from "./palette";
import { useI18n } from "./lang";
import { GRAPH_STRINGS } from "./graphStrings";
import { RelationCard, type RelationCardData, type RelationItem } from "./RelationCard";
import "./KnowledgeGraphPanel.css";

interface Props {
  mode: "local" | "global";
  notePath: string;
  refreshKey: number;
  load: () => Promise<KnowledgeGraph>;
  onOpen: (path: string) => void;
}

// v2: the defaults changed (more room between nodes), so older saved values are not reused.
const FORCES_KEY = "mdnotes.graph.forces.v2";
const FLOAT_KEY = "mdnotes.graph.float";
const SHAPES_KEY = "mdnotes.graph.shapes";
/** Energy that keeps the graph in gentle motion, and the one used while a node is dragged. */
const FLOAT_TARGET = 0.08;
const FLOAT_DRIFT = 0.08;
const DRAG_TARGET = 0.3;
/** Above this many nodes the graph settles by default instead of drifting. */
const FLOAT_AUTO_LIMIT = 400;

function readForces(): Forces {
  try {
    const raw = localStorage.getItem(FORCES_KEY);
    if (raw) return { ...DEFAULT_FORCES, ...(JSON.parse(raw) as Partial<Forces>) };
  } catch {
    // storage unavailable: defaults
  }
  return { ...DEFAULT_FORCES };
}

function readShapes(): boolean {
  try {
    return localStorage.getItem(SHAPES_KEY) === "1";
  } catch {
    return false;
  }
}

/** Type glyph on a round token, for the legend. */
function GlyphIcon({ type, color }: { type: string; color: string }) {
  const path = glyphFor(type);
  return (
    <svg className="glyph-icon" viewBox="-12 -12 24 24" width="16" height="16" aria-hidden="true">
      <circle r="11" fill={color} />
      {path && <path d={path} fill="#FFFFFF" transform="scale(0.62)" />}
    </svg>
  );
}

function readFloat(): boolean | null {
  try {
    const raw = localStorage.getItem(FLOAT_KEY);
    return raw === null ? null : raw === "1";
  } catch {
    return null;
  }
}

interface Hover {
  title: string;
  type: string;
  degree: number;
  x: number;
  y: number;
}

interface ViewState {
  hovered: string | null;
  neighbours: Set<string>;
  search: string;
  hidden: Set<string>;
  trace: { nodes: Set<string>; edges: Set<string> } | null;
  shapes: boolean;
  maxDigit: number | null;
}

interface TraceNode {
  path: string;
  title: string;
  type: string;
  grade?: string;
  kinds: string;
  evidence?: string;
  repeated: boolean;
  children: TraceNode[];
}

/** Upstream chain of `root`: who owns it, who owns them, plus the materials around the chain. */
function buildTrace(graph: Graph, root: string) {
  const nodes = new Set<string>([root]);
  const edges = new Set<string>();
  const materials = new Set<string>();
  const expanded = new Set<string>();
  const visit = (node: string, depth: number): TraceNode => {
    const owners: TraceNode[] = [];
    const repeated = expanded.has(node);
    expanded.add(node);
    if (!repeated && depth < 8) {
      graph.forEachInEdge(node, (edge, attributes, source) => {
        if (attributes.hasItems !== true) return;
        edges.add(edge);
        nodes.add(source);
        const items = attributes.items as RelationItem[];
        const child = visit(source, depth + 1);
        child.grade = attributes.grade as string | undefined;
        child.kinds = items.map((item) => item.kind).filter(Boolean).join('; ');
        child.evidence = items.find((item) => item.evidence)?.evidence;
        owners.push(child);
      });
    }
    graph.forEachEdge(node, (edge, attributes, source, target) => {
      const other = source === node ? target : source;
      if (graph.getNodeAttribute(other, 'typeKey') === 'material' && attributes.hasItems !== true) {
        materials.add(other);
        edges.add(edge);
      }
    });
    return {
      path: node,
      title: String(graph.getNodeAttribute(node, 'fullTitle')),
      type: String(graph.getNodeAttribute(node, 'typeKey')),
      kinds: '',
      repeated,
      children: owners,
    };
  };
  const tree = visit(root, 0);
  materials.forEach((material) => nodes.add(material));
  return { tree, nodes, edges, materials: [...materials] };
}

export function KnowledgeGraphPanel({ mode, notePath, refreshKey, load, onOpen }: Props) {
  const { language } = useI18n();
  const strings = GRAPH_STRINGS[language];
  const containerRef = useRef<HTMLDivElement | null>(null);
  const rendererRef = useRef<Sigma | null>(null);
  const viewRef = useRef<ViewState>({ hovered: null, neighbours: new Set(), search: "", hidden: new Set(), trace: null, shapes: false, maxDigit: null });
  const graphRef = useRef<Graph | null>(null);
  const traceModeRef = useRef(false);
  const simRef = useRef<LiveLayout | null>(null);
  const wakeRef = useRef<() => void>(() => undefined);
  const positionsRef = useRef<Map<string, { x: number; y: number }>>(new Map());
  const [forces, setForces] = useState<Forces>(readForces);
  const [floating, setFloating] = useState<boolean | null>(readFloat);
  const [paused, setPaused] = useState(false);
  const [shapes, setShapes] = useState<boolean>(readShapes);
  const [maxDigit, setMaxDigit] = useState<number | null>(null);
  const forcesRef = useRef(forces);
  const floatRef = useRef(false);
  const pausedRef = useRef(false);
  const [snapshot, setSnapshot] = useState<KnowledgeGraph | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [search, setSearch] = useState("");
  const [hiddenTypes, setHiddenTypes] = useState<Set<string>>(new Set());
  const [hover, setHover] = useState<Hover | null>(null);
  const [edgeCard, setEdgeCard] = useState<RelationCardData | null>(null);
  const [expanded, setExpanded] = useState(false);
  const [traceMode, setTraceMode] = useState(false);
  const [traceRoot, setTraceRoot] = useState<string | null>(null);
  const [traceResult, setTraceResult] = useState<ReturnType<typeof buildTrace> | null>(null);

  useEffect(() => {
    let cancelled = false;
    setLoading(true);
    setError(null);
    load()
      .then((result) => {
        if (!cancelled) setSnapshot(result);
      })
      .catch((reason) => {
        if (!cancelled) {
          setSnapshot(null);
          setError(String(reason));
        }
      })
      .finally(() => {
        if (!cancelled) setLoading(false);
      });
    return () => {
      cancelled = true;
    };
  }, [mode, notePath, refreshKey]);

  // Types present in the graph with counts, for the legend and its filter.
  const legend = useMemo(() => {
    const counts = new Map<string, number>();
    snapshot?.nodes.forEach((node) => {
      const key = node.noteType ?? "";
      counts.set(key, (counts.get(key) ?? 0) + 1);
    });
    return [...counts.entries()].sort((a, b) => b[1] - a[1] || a[0].localeCompare(b[0]));
  }, [snapshot]);

  useEffect(() => {
    const container = containerRef.current;
    if (!container || !snapshot || snapshot.nodes.length === 0) return;

    const dark = isDarkBackground();
    // Nodes are drawn in pixels, so a small pane needs smaller nodes to stay readable.
    const fit = Math.max(0.45, Math.min(1, Math.min(container.clientWidth, container.clientHeight) / 640));
    const labelColor = dark ? "#E5E7EB" : "#1D2430";
    const edgeColor = dark ? "#5B6475" : "#A3ABB9";
    const dimNode = dark ? "#2B313D" : "#DDE1E8";

    const graph = new Graph({ type: "directed", multi: false, allowSelfLoops: false });
    const indexOf = new Map<string, number>();
    const degree = new Map<string, number>();
    snapshot.nodes.forEach((node, index) => {
      indexOf.set(node.path, index);
      degree.set(node.path, 0);
    });

    // One line per pair of notes: typed relations (styled by grade, with an
    // arrow and a card) win over a plain [[link]] between the same notes.
    const titleOf = new Map(snapshot.nodes.map((node) => [node.path, node.title]));
    interface PairEdge {
      source: string;
      target: string;
      grade?: string;
      items: RelationItem[];
      digit?: number;
    }
    const pairEdges = new Map<string, PairEdge>();
    const pairKey = (a: number, b: number) => (a < b ? `${a}:${b}` : `${b}:${a}`);
    const evidence = new Map<string, number>();
    (snapshot.relations ?? []).forEach((relation) => {
      const a = indexOf.get(relation.source);
      const b = indexOf.get(relation.target);
      if (a === undefined || b === undefined || a === b) return;
      const key = pairKey(a, b);
      const digit = gradeDigit(relation.grade);
      const item: RelationItem = {
        kind: relation.kind,
        grade: relation.grade ?? undefined,
        origin: relation.origin ?? undefined,
        status: relation.status ?? undefined,
        evidence: relation.evidence ?? undefined,
        from: titleOf.get(relation.source) ?? relation.source,
        to: titleOf.get(relation.target) ?? relation.target,
      };
      const current = pairEdges.get(key);
      if (!current) {
        pairEdges.set(key, {
          source: relation.source,
          target: relation.target,
          grade: relation.grade ?? undefined,
          items: [item],
          digit,
        });
      } else {
        current.items.push(item);
        if (digit !== undefined && (current.digit === undefined || digit < current.digit)) {
          current.digit = digit;
          current.grade = relation.grade ?? undefined;
          current.source = relation.source;
          current.target = relation.target;
        }
      }
      if (digit !== undefined) {
        for (const path of [relation.source, relation.target]) {
          const best = evidence.get(path);
          if (best === undefined || digit < best) evidence.set(path, digit);
        }
      }
    });
    snapshot.edges.forEach((edge) => {
      const a = indexOf.get(edge.source);
      const b = indexOf.get(edge.target);
      if (a === undefined || b === undefined || a === b) return;
      const key = pairKey(a, b);
      if (!pairEdges.has(key)) pairEdges.set(key, { source: edge.source, target: edge.target, items: [] });
    });
    const pairs: Array<[number, number]> = [];
    pairEdges.forEach((edge) => {
      degree.set(edge.source, (degree.get(edge.source) ?? 0) + 1);
      degree.set(edge.target, (degree.get(edge.target) ?? 0) + 1);
      pairs.push([indexOf.get(edge.source) ?? 0, indexOf.get(edge.target) ?? 0]);
    });

    const sim = new LiveLayout(snapshot.nodes.length, pairs, forcesRef.current);
    // Keep the positions of nodes that were already on the screen (a refresh must not scramble the picture).
    let known = 0;
    snapshot.nodes.forEach((node, index) => {
      const previous = positionsRef.current.get(node.path);
      if (previous) {
        sim.place(index, previous.x, previous.y);
        known += 1;
      }
    });
    if (known > snapshot.nodes.length / 2) sim.setEnergy(0.3);
    simRef.current = sim;
    floatRef.current = floating ?? snapshot.nodes.length <= FLOAT_AUTO_LIMIT;
    sim.setTarget(floatRef.current ? FLOAT_TARGET : 0);
    sim.setDrift(floatRef.current ? FLOAT_DRIFT : 0);
    snapshot.nodes.forEach((node, index) => {
      const isFocus = mode === "local" && node.path === notePath;
      const type = node.noteType ?? "";
      const size = nodeSize(degree.get(node.path) ?? 0) * fit;
      graph.addNode(node.path, {
        label: node.title,
        x: sim.x[index],
        y: sim.y[index],
        size: isFocus ? size + 4 * fit : size,
        color: typeColor(type),
        typeKey: type,
        fullTitle: node.title,
        isFocus,
        evidence: evidence.get(node.path),
        glyph: glyphFor(type) ?? undefined,
      });
    });
    pairEdges.forEach((edge, key) => {
      graph.addDirectedEdgeWithKey(`pair:${key}`, edge.source, edge.target, {
        size: 1,
        grade: edge.grade,
        digit: edge.digit,
        arrow: edge.items.length > 0,
        hasItems: edge.items.length > 0,
        items: edge.items,
      });
    });

    const view = viewRef.current;
    view.hovered = null;
    view.neighbours = new Set();

    const renderer = new Sigma(graph, container, {
      renderEdgeLabels: false,
      // Fewer, larger labels: on a dense graph the names of neighbours no longer overprint each other.
      labelDensity: 0.6,
      labelGridCellSize: 130,
      labelRenderedSizeThreshold: 8,
      labelColor: { color: labelColor },
      labelSize: 12,
      defaultEdgeColor: edgeColor,
      zIndex: true,
      nodeReducer: (node, data) => {
        const result = { ...data };
        if (view.shapes) result.color = stateColor(stateOf(data.evidence as number | undefined), dark);
        if (view.hidden.has(String(data.typeKey))) {
          result.hidden = true;
          return result;
        }
        const searching = view.search.length > 0;
        const matches = searching && String(data.fullTitle).toLowerCase().includes(view.search);
        const focus = view.hovered;
        const related = focus !== null && (node === focus || view.neighbours.has(node));
        if (view.trace !== null) {
          if (view.trace.nodes.has(node)) {
            result.zIndex = 2;
            result.forceLabel = true;
          } else {
            result.color = dimNode;
            result.label = "";
            result.zIndex = 0;
          }
        } else if (focus !== null && !related) {
          result.color = dimNode;
          result.label = "";
          result.zIndex = 0;
        } else if (searching && !matches) {
          result.color = dimNode;
          result.label = "";
          result.zIndex = 0;
        } else {
          result.zIndex = 1;
          if (matches || node === focus || data.isFocus === true) {
            result.forceLabel = true;
            result.zIndex = 2;
          }
        }
        return result;
      },
      // Lines are drawn by the overlay (dashes by grade); hide Sigma's own.
      edgeReducer: (_edge, data) => ({ ...data, hidden: true }),
    });
    rendererRef.current = renderer;
    graphRef.current = graph;
    const detachOverlay = attachOverlay(container, renderer, graph, view, dark, (hit: EdgeHit | null) => {
      if (!hit || !graph.hasEdge(hit.edge)) {
        setEdgeCard(null);
        return;
      }
      const [source, target] = graph.extremities(hit.edge);
      const items = graph.getEdgeAttribute(hit.edge, "items") as RelationItem[];
      setEdgeCard({
        x: hit.x,
        y: hit.y,
        from: String(graph.getNodeAttribute(source, "fullTitle")),
        to: String(graph.getNodeAttribute(target, "fullTitle")),
        items,
      });
    });

    // Hover: highlight the node with its neighbours, show a small card.
    renderer.on("enterNode", ({ node }) => {
      view.hovered = node;
      view.neighbours = new Set(graph.neighbors(node));
      const attributes = graph.getNodeAttributes(node);
      const point = renderer.graphToViewport({ x: attributes.x, y: attributes.y });
      setHover({
        title: String(attributes.fullTitle),
        type: String(attributes.typeKey),
        degree: graph.degree(node),
        x: point.x,
        y: point.y,
      });
      renderer.refresh();
    });
    renderer.on("leaveNode", () => {
      view.hovered = null;
      view.neighbours = new Set();
      setHover(null);
      renderer.refresh();
    });

    // Live simulation: runs while the graph is warm, sleeps when it has settled, wakes on interaction.
    let frame = 0;
    const tick = () => {
      frame = 0;
      if (!pausedRef.current) {
        const steps = sim.energy > 0.5 && snapshot.nodes.length > 300 ? 2 : 1;
        for (let i = 0; i < steps; i += 1) sim.tick();
        graph.updateEachNodeAttributes((node, attributes) => {
          const index = indexOf.get(node) ?? 0;
          return { ...attributes, x: sim.x[index], y: sim.y[index] };
        });
        snapshot.nodes.forEach((node, index) => {
          positionsRef.current.set(node.path, { x: sim.x[index], y: sim.y[index] });
        });
      }
      if (!sim.settled && !pausedRef.current) frame = requestAnimationFrame(tick);
    };
    const wake = () => {
      if (frame === 0) frame = requestAnimationFrame(tick);
    };
    wakeRef.current = wake;
    wake();

    // Drag a node to move it; a click without movement opens the note.
    let dragged: string | null = null;
    let moved = false;
    renderer.on("downNode", (event) => {
      dragged = event.node;
      moved = false;
    });
    renderer.getMouseCaptor().on("mousedown", () => {
      if (!renderer.getCustomBBox()) renderer.setCustomBBox(renderer.getBBox());
    });
    renderer.getMouseCaptor().on("mousemovebody", (event) => {
      if (dragged === null) return;
      moved = true;
      const position = renderer.viewportToGraph(event);
      const draggedIndex = indexOf.get(dragged) ?? 0;
      sim.pin(draggedIndex, position.x, position.y);
      sim.setTarget(DRAG_TARGET);
      sim.reheat(0.3);
      graph.setNodeAttribute(dragged, "x", position.x);
      graph.setNodeAttribute(dragged, "y", position.y);
      wake();
      setHover(null);
      event.preventSigmaDefault();
      event.original.preventDefault();
      event.original.stopPropagation();
    });
    renderer.getMouseCaptor().on("mouseup", () => {
      if (dragged !== null) {
        sim.unpin(indexOf.get(dragged) ?? 0);
        sim.setTarget(floatRef.current ? FLOAT_TARGET : 0);
        wake();
      }
      dragged = null;
    });
    renderer.on("clickNode", ({ node }) => {
      if (!moved) {
        if (traceModeRef.current) setTraceRoot(node);
        else onOpen(node);
      }
      moved = false;
    });

    return () => {
      cancelAnimationFrame(frame);
      simRef.current = null;
      wakeRef.current = () => undefined;
      detachOverlay();
      renderer.kill();
      rendererRef.current = null;
      graphRef.current = null;
    };
  }, [snapshot, mode, notePath, onOpen, expanded]);

  // Force sliders, drift and pause act on the running simulation without rebuilding the graph.
  useEffect(() => {
    forcesRef.current = forces;
    simRef.current?.setForces(forces);
    wakeRef.current();
    try {
      localStorage.setItem(FORCES_KEY, JSON.stringify(forces));
    } catch {
      // storage unavailable: the choice just is not remembered
    }
  }, [forces]);

  useEffect(() => {
    const sim = simRef.current;
    const count = snapshot?.nodes.length ?? 0;
    floatRef.current = floating ?? count <= FLOAT_AUTO_LIMIT;
    sim?.setTarget(floatRef.current ? FLOAT_TARGET : 0);
    sim?.setDrift(floatRef.current ? FLOAT_DRIFT : 0);
    if (floatRef.current) sim?.reheat(0.1);
    wakeRef.current();
  }, [floating, snapshot]);

  useEffect(() => {
    pausedRef.current = paused;
    wakeRef.current();
  }, [paused]);

  // The glyph mode only changes what the reducers and the overlay draw.
  useEffect(() => {
    viewRef.current.shapes = shapes;
    rendererRef.current?.refresh();
    try {
      localStorage.setItem(SHAPES_KEY, shapes ? "1" : "0");
    } catch {
      // storage unavailable: the choice just is not remembered
    }
  }, [shapes, snapshot]);

  // The reliability filter only changes which lines the overlay draws.
  useEffect(() => {
    viewRef.current.maxDigit = maxDigit;
    rendererRef.current?.refresh();
  }, [maxDigit, snapshot]);

  // Search and type filters only change what the reducers return.
  useEffect(() => {
    viewRef.current.search = search.trim().toLowerCase();
    viewRef.current.hidden = hiddenTypes;
    rendererRef.current?.refresh();
  }, [search, hiddenTypes, snapshot]);

  // Provenance trace: recompute when the root changes, keep the graph repainted.
  useEffect(() => {
    traceModeRef.current = traceMode;
    const graph = graphRef.current;
    if (!traceMode || traceRoot === null || !graph || !graph.hasNode(traceRoot)) {
      viewRef.current.trace = null;
      setTraceResult(null);
    } else {
      const result = buildTrace(graph, traceRoot);
      viewRef.current.trace = { nodes: result.nodes, edges: result.edges };
      setTraceResult(result);
    }
    rendererRef.current?.refresh();
  }, [traceMode, traceRoot, snapshot]);

  const toggleType = (type: string) => {
    setHiddenTypes((current) => {
      const next = new Set(current);
      if (next.has(type)) next.delete(type);
      else next.add(type);
      return next;
    });
  };

  const hasGraph = !!snapshot && snapshot.nodes.length > 0;
  const floatOn = floating ?? (snapshot?.nodes.length ?? 0) <= FLOAT_AUTO_LIMIT;

  return (
    <section
      className={`knowledge-graph${expanded ? " expanded" : ""}`}
      aria-label={mode === "local" ? strings.localTitle : strings.globalTitle}
    >
      <div className="knowledge-graph-meta">
        <div className="knowledge-graph-title">
          <strong>{mode === "local" ? strings.localTitle : strings.globalTitle}</strong>
          {hasGraph && (
            <button type="button" className="knowledge-graph-expand" onClick={() => setExpanded((value) => !value)}>
              {expanded ? strings.collapse : strings.expand}
            </button>
          )}
        </div>
        {snapshot && (
          <span>
            {strings.nodes(snapshot.nodes.length)} · {strings.edges(snapshot.edges.length)}
          </span>
        )}
      </div>
      {hasGraph && (
        <div className="knowledge-graph-tools">
          <input
            type="search"
            value={search}
            placeholder={strings.searchPlaceholder}
            onChange={(event) => setSearch(event.target.value)}
          />
          <button
            type="button"
            className={`knowledge-graph-trace-toggle${traceMode ? " on" : ""}`}
            aria-pressed={traceMode}
            onClick={() => {
              setTraceMode((value) => !value);
              setTraceRoot(null);
            }}
          >
            {strings.trace}
          </button>
          {traceMode && traceRoot === null && <small className="knowledge-graph-hint">{strings.traceHint}</small>}
          <div className="knowledge-graph-physics">
            <button
              type="button"
              aria-pressed={floatOn}
              className={floatOn ? "on" : ""}
              onClick={() => setFloating(!floatOn)}
            >
              {strings.float}
            </button>
            <button type="button" aria-pressed={shapes} className={shapes ? "on" : ""} title={strings.shapesHint} onClick={() => setShapes((value) => !value)}>
              {strings.shapes}
            </button>
            <label className="knowledge-graph-grade-filter" title={strings.gradeFilterHint}>
              {strings.gradeFilter}{" "}
              <select
                value={maxDigit === null ? "" : String(maxDigit)}
                onChange={(event) => setMaxDigit(event.target.value === "" ? null : Number(event.target.value))}
              >
                <option value="">{strings.gradeAny}</option>
                <option value="2">{strings.gradeStrong}</option>
                <option value="4">{strings.gradeMedium}</option>
              </select>
            </label>
            <button type="button" aria-pressed={paused} className={paused ? "on" : ""} onClick={() => setPaused((value) => !value)}>
              {paused ? strings.resume : strings.pause}
            </button>
            <details>
              <summary>{strings.forces}</summary>
              {(
                [
                  ["center", strings.forceCenter, 0, 0.3, 0.01],
                  ["repel", strings.forceRepel, 0, 3, 0.05],
                  ["link", strings.forceLink, 0, 1, 0.02],
                  ["distance", strings.forceDistance, 0.3, 3, 0.05],
                ] as const
              ).map(([key, label, min, max, step]) => (
                <label key={key} className="knowledge-graph-slider">
                  {label}
                  <input
                    type="range"
                    min={min}
                    max={max}
                    step={step}
                    value={forces[key]}
                    onChange={(event) => setForces({ ...forces, [key]: Number(event.target.value) })}
                  />
                </label>
              ))}
              <button type="button" onClick={() => setForces({ ...DEFAULT_FORCES })}>
                {strings.forcesReset}
              </button>
            </details>
          </div>
          <details className="knowledge-graph-key">
            <summary>{strings.legend}</summary>
            <p>
              <strong>{strings.legendTypes}</strong>
              {["person", "company", "institution", "case", "material", "group"].map((type) => (
                <span key={type} className="knowledge-graph-key-item">
                  <GlyphIcon type={type} color={stateColor("none", isDarkBackground())} />
                  {type}
                </span>
              ))}
            </p>
            <p>
              <strong>{strings.legendState}</strong>
              {(["strong", "medium", "weak", "none"] as NodeState[]).map((state) => (
                <span key={state} className="knowledge-graph-key-item">
                  <span className="dot" style={{ background: stateColor(state, isDarkBackground()) }} />
                  {state === "strong" ? strings.stateStrong : state === "medium" ? strings.stateMedium : state === "weak" ? strings.stateWeak : strings.stateNone}
                </span>
              ))}
            </p>
            <p>
              <strong>{strings.legendLines}</strong>
              {(["A1", "C3", "F6", undefined] as Array<string | undefined>).map((grade) => {
                const style = edgeStyle(grade, isDarkBackground());
                return (
                  <span key={grade ?? "none"} className="knowledge-graph-key-item">
                    <svg width="34" height="8" aria-hidden="true">
                      <line x1="1" y1="4" x2="33" y2="4" stroke={style.color} strokeWidth={style.width} strokeDasharray={style.dash.join(" ")} />
                    </svg>
                    {grade ?? strings.stateNone}
                  </span>
                );
              })}
            </p>
            <p>{strings.legendRumour}</p>
          </details>
          <ul className="knowledge-graph-legend" aria-label={strings.typesHeading}>
            {legend.map(([type, count]) => (
              <li key={type || "_none"}>
                <button
                  type="button"
                  className={hiddenTypes.has(type) ? "off" : ""}
                  aria-pressed={!hiddenTypes.has(type)}
                  onClick={() => toggleType(type)}
                >
                  {shapes ? <GlyphIcon type={type} color={stateColor("none", isDarkBackground())} /> : <span className="dot" style={{ background: typeColor(type) }} />}
                  {type || strings.noType} <small>{count}</small>
                </button>
              </li>
            ))}
          </ul>
          {shapes && (
            <ul className="knowledge-graph-states" aria-label={strings.shapesHint}>
              {(["strong", "medium", "weak", "none"] as NodeState[]).map((state) => (
                <li key={state}>
                  <span className="dot" style={{ background: stateColor(state, isDarkBackground()) }} />
                  {state === "strong" ? strings.stateStrong : state === "medium" ? strings.stateMedium : state === "weak" ? strings.stateWeak : strings.stateNone}
                </li>
              ))}
            </ul>
          )}
        </div>
      )}
      {traceMode && traceResult && (
        <div className="knowledge-graph-trace">
          <header>
            <strong>
              {strings.traceFor} {traceResult.tree.title}
            </strong>
            <button type="button" onClick={() => setTraceRoot(null)}>
              {strings.traceClear}
            </button>
          </header>
          {traceResult.tree.children.length === 0 ? (
            <small>{strings.traceNone}</small>
          ) : (
            <TraceList nodes={traceResult.tree.children} onOpen={onOpen} />
          )}
          {traceResult.materials.length > 0 && (
            <div>
              <strong>{strings.traceMaterials}</strong>
              <ul>
                {traceResult.materials.map((path) => (
                  <li key={path}>
                    <button type="button" className="link" onClick={() => onOpen(path)}>
                      {String(graphRef.current?.getNodeAttribute(path, "fullTitle") ?? path)}
                    </button>
                  </li>
                ))}
              </ul>
            </div>
          )}
        </div>
      )}
      {loading && <p className="panel-note">{strings.loading}</p>}
      {error && <p className="panel-note warn">{error}</p>}
      {!loading && !error && snapshot?.nodes.length === 0 && <p className="panel-note">{strings.empty}</p>}
      <div className="knowledge-graph-stage">
        <div ref={containerRef} className={`knowledge-graph-canvas ${hasGraph ? "has-graph" : ""}`} />
        {edgeCard && !hover && <RelationCard card={edgeCard} />}
        {hover && (
          <div className="knowledge-graph-card" style={{ left: hover.x + 14, top: hover.y + 14 }}>
            <strong>{hover.title}</strong>
            <span>
              <i className="dot" style={{ background: typeColor(hover.type) }} />
              {hover.type || strings.noType} · {strings.connections(hover.degree)}
            </span>
          </div>
        )}
      </div>
      {hasGraph && <p className="knowledge-graph-hint">{strings.openHint}</p>}
    </section>
  );
}

function TraceList({ nodes, onOpen }: { nodes: TraceNode[]; onOpen: (path: string) => void }) {
  return (
    <ul>
      {nodes.map((node) => {
        const colors = chipColors(node.grade);
        return (
          <li key={node.path}>
            <div className="trace-line">
              <span className="relation-chip" style={{ background: colors.background, color: colors.color }}>
                {node.grade ?? "—"}
              </span>
              <button type="button" className="link" onClick={() => onOpen(node.path)}>
                {node.title}
              </button>
              {node.kinds && <small>{node.kinds}</small>}
              {node.repeated && <small>↺</small>}
            </div>
            {node.evidence && <small>{node.evidence}</small>}
            {node.children.length > 0 && <TraceList nodes={node.children} onOpen={onOpen} />}
          </li>
        );
      })}
    </ul>
  );
}
